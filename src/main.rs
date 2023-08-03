use chrono::prelude::*;
use crossterm::{
    event::{self, Event as CEvent, KeyCode},
    terminal::{enable_raw_mode, disable_raw_mode}
};
use tui::{
    backend::CrosstermBackend, 
    Terminal, 
    layout::{self, Layout, Direction, Constraint, Alignment},
    widgets::{Paragraph, Block, Borders, BorderType}, 
    style::{Color, Style}
};
use serde::{Serialize, Deserialize};
use thiserror::Error;
use std::{
    time::{Duration, Instant}, 
    thread, slice::Chunks
};
use std::io;
use std::sync::mpsc;


const DB_PATH : &str = "../data/db.json";

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
enum Menu{
    Home,
    Task,
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

    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;


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
        });

    }

    Ok(())
}
