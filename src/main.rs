use chrono::{prelude::*, format::Parsed};
use crossterm::{
    event::{self, Event as CEvent, KeyCode, DisableMouseCapture, EnableMouseCapture},
    terminal::{enable_raw_mode, disable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    execute
};
use tui::{
    backend::CrosstermBackend, 
    Terminal, 
    layout::{self, Layout, Direction, Constraint, Alignment},
    widgets::{Paragraph, Block, Borders, BorderType, Tabs, ListState, List, Table, ListItem, Row, Cell}, 
    style::{Color, Style, self, Modifier}, 
    text::{Spans, Span}, 
};
use serde::{Serialize, Deserialize};
use thiserror::Error;
use std::{
    time::{Duration, Instant}, 
    thread, slice::Chunks, fs, collections::vec_deque
};
use std::io;
use std::sync::mpsc;


const DB_PATH : &str = "./data/db.json";

//Data structure
#[derive(Serialize, Deserialize, Clone)]
struct Task{
    id: usize,
    task: String,
    category: String,
    created_at: DateTime<Utc>,

}

//I/O DB error handlers
#[derive(Error, Debug)]
pub enum Error{
    #[error("error reading the DB file: {0}")]
    ReadDBError(#[from] io::Error),
    #[error("error parsing the DB file: {0}")]
    ParseDBError(#[from] serde_json::Error),
}

//Tick for response
enum Event<I>{
    Input(I),
    Tick,
}

//Principal menu
#[derive(Copy, Clone, Debug)]
enum Menu{
    Home,
    Task,
}

impl From<Menu> for usize {
    // add code here
    fn from(input: Menu) -> usize{
        match input {
            Menu::Home => 0,
            Menu::Task => 1,
        }
    }
}


fn main() -> Result<(), Box<dyn std::error::Error>>{

    enable_raw_mode().expect("teminal in raw mode now");



    let (tx, rx) = mpsc::channel();

    let tick_rate = Duration::from_millis(200);

    thread::spawn(move ||{
        let mut last_tick = Instant::now();
        loop {
            let timeout = tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or_else(|| Duration ::from_secs(0));

            if(event::poll(timeout).expect("poll works here")){
                if let CEvent::Key(key) = event::read().expect("can read events"){
                    tx.send(Event::Input(key)).expect("can send events");
                }
            }

            if last_tick.elapsed() >= tick_rate{
                if let Ok(_) = tx.send(Event::Tick){
                    last_tick = Instant::now();
                }
            }
        }
    });

    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;


    let menu_titles = vec!["HOME", "TASK", "ADD", "DELETE", "QUIT"];
    let mut active_menu_item = Menu::Home;
    let mut task_list_state = ListState::default();
    task_list_state.select(Some(0));


    //render loop
    loop {
        terminal.draw(|rect| {
            let size = rect.size();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(2)
                .constraints(
                    [
                        Constraint::Length(3),
                        Constraint::Min(2),
                        Constraint::Length(3)
                    ]
                    .as_ref(),
                ).split(size);

            let footer = Paragraph::new("This is the footer of the App")
                .style(Style::default().fg(Color::LightGreen))
                .alignment(Alignment::Center)
                .block(
                    Block::default()
                    .borders(Borders::ALL)
                    .style(Style::default().fg(Color::White))
                    .title("FOOTER")
                    .border_type(BorderType::Plain),
                );
            rect.render_widget(footer, chunks[2]);

            let menu: Vec<_> = menu_titles
                .iter()
                .map(|t| {
                    let (first, rest) = t.split_at(1);
                    Spans::from(vec![
                                Span::styled(
                                    first,
                                    Style::default()
                                    .fg(Color::Yellow)
                                    .add_modifier(Modifier::UNDERLINED),
                                    ),
                                    Span::styled(rest, Style::default().fg(Color::White)),
                    ])
                })
            .collect();

            let tabs = Tabs::new(menu)
                .select(active_menu_item.into())
                .block(Block::default().title("Menu").borders(Borders::ALL))
                .style(Style::default().fg(Color::White))
                .highlight_style(Style::default().fg(Color::Yellow))
                .divider(Span::raw("|"));

            rect.render_widget(tabs, chunks[0]);

            match active_menu_item {
                Menu::Home => rect.render_widget(render_home(), chunks[1]),
                Menu::Task => {
                    let task_chunks = Layout::default()
                        .direction(Direction::Horizontal)
                        .constraints(
                            [Constraint::Percentage(20), Constraint::Percentage(80)].as_ref(),
                        )
                        .split(chunks[1]);
                    let (left, right) = render_task(&task_list_state);
                    rect.render_stateful_widget(left, task_chunks[0], &mut task_list_state);
                    rect.render_widget(right, task_chunks[1]);
                }
                _ => {}
            }
         });


        match rx.recv()?{
            Event::Input(event) => match event.code{
                KeyCode::Char('q') => {
                    disable_raw_mode()?;
                    terminal.show_cursor()?;
                    break;
                }
                KeyCode::Char('h') => active_menu_item = Menu::Home,
                KeyCode::Char('t') => active_menu_item = Menu::Task,
                _ => {}
            },
            Event::Tick => {}
        }

    }
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;


    Ok(())
}

fn read_db() -> Result<Vec<Task>, Error>{
    let db_content = fs::read_to_string(DB_PATH)?;
    let parsed: Vec<Task> = serde_json::from_str(&db_content)?;

    Ok(parsed)
}

fn render_home<'a>() -> Paragraph<'a>{
    let home = Paragraph::new(vec![
        Spans::from(vec![Span::raw("")]),
        Spans::from(vec![Span::raw("Welcome")]),
        Spans::from(vec![Span::raw("")]),
        Spans::from(vec![Span::raw("to")]),
        Spans::from(vec![Span::raw("")]),
        Spans::from(vec![Span::styled(
                "TODO_LIST", 
                Style::default().fg(Color::LightBlue)
                )]),
        Spans::from(vec![Span::raw("")]),
        Spans::from(vec![Span::raw("Press t to access tasks, 'a' to add a random new task and 'd' to delete the currently selected task")]),
    ])
    .alignment(Alignment::Center)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::White))
            .title("Home")
            .border_type(BorderType::Rounded)
        );
    home
}

fn render_task<'a>(task_list_state: &ListState) -> (List<'a>, Table<'a>){
    let task = Block::default()
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::White))
        .title("Task")
        .border_type(BorderType::Plain);

    let task_list = read_db().expect("can fetch task list");

    let items: Vec<_> = task_list
        .iter()
        .map(|task|{
            ListItem::new(Spans::from(vec![Span::styled(
                        task.task.clone(), 
                        Style::default(),
                    )]))
        }).collect();

    let select_task = task_list
        .get(
            task_list_state
            .selected()
            .expect("There is always a task selected"),
        )
        .expect("exist")
        .clone();

    let list = List::new(items).block(task).highlight_style(
        Style::default()
        .bg(Color::Yellow)
        .fg(Color::Black)
        .add_modifier(Modifier::BOLD),
    );

    let task_detail = Table::new(vec![Row::new(vec![
        Cell::from(Span::raw(select_task.id.to_string())),
        Cell::from(Span::raw(select_task.task)),
        Cell::from(Span::raw(select_task.category)),
        Cell::from(Span::raw(select_task.created_at.to_string())),
    ])])
    .header(Row::new(vec![
        Cell::from(Span::styled(
                "ID", 
                Style::default().add_modifier(Modifier::BOLD),
        )),
        Cell::from(Span::styled(
                "TASK", 
                Style::default().add_modifier(Modifier::BOLD),
        )),
        Cell::from(Span::styled(
                "CATEGORY", 
                Style::default().add_modifier(Modifier::BOLD),
        )),
        Cell::from(Span::styled(
                "CREATED_AT", 
                Style::default().add_modifier(Modifier::BOLD),
        )),
    ]))
    .block(
        Block::default()
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::White))
        .title("Detail")
        .border_type(BorderType::Plain),
        )
    .widths(&[
        Constraint::Percentage(5),
        Constraint::Percentage(20),
        Constraint::Percentage(20),
        Constraint::Percentage(5),
        Constraint::Percentage(20),
    ]);

    (list, task_detail)

}
