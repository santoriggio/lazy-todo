use chrono::{DateTime, Local, TimeZone};

pub struct Todo {
    id: usize,
    pub title: String,
    created_at: i64,
    updated_at: i64,
}

impl Todo {
    pub fn new(id: usize, title: String) -> Self {
        Self {
            id,
            title,
            created_at: Local::now().timestamp_millis(),
            updated_at: Local::now().timestamp_millis(),
        }
    }
}
