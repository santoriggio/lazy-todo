use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Workspace {
    pub id: usize,
    pub title: String,
}

impl Workspace {
    pub fn new(title: String) -> Self {
        Self { id: 10, title }
    }
}

