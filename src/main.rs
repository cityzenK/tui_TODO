use chrono::prelude::*;
use serde::{Serialize, Deserialize};
use thiserror::Error;
use std::io;


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

fn main() {
    println!("Hello, world!");
}
