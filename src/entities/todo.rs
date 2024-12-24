use chrono::{DateTime, Local, TimeZone};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]

pub struct Todo {
    pub id: usize,
    pub title: String,
    pub content: String,
    pub done: bool,
    created_at: i64,
    updated_at: i64,
}

impl Todo {
    pub fn new(id: usize, title: String, content: String) -> Self {
        Self {
            id,
            title,
            content,
            done: false,
            created_at: Local::now().timestamp_millis(),
            updated_at: Local::now().timestamp_millis(),
        }
    }
}
