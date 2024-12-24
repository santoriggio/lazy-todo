mod entities;

use crossterm::event::KeyEvent;
use entities::Todo;
use serde::{Deserialize, Serialize};

use std::{
    fs::File,
    io::{Read, Write},
    thread::sleep,
    time::Duration,
    usize,
};

use ratatui::{
    crossterm::event::{self, Event, KeyCode, KeyEventKind},
    layout::{Alignment, Constraint, Layout, Margin, Position, Rect},
    style::{self, Color, Modifier, Style, Stylize},
    text::{Span, Text},
    widgets::{
        Block, BorderType, Cell, HighlightSpacing, List, ListItem, ListState, Paragraph, Row,
        Scrollbar, ScrollbarOrientation, ScrollbarState, Table, TableState, Wrap,
    },
    DefaultTerminal, Frame,
};

fn main() -> Result<(), Err> {
    let terminal = ratatui::init();
    let app_result = App::new().run(terminal);
    ratatui::restore();
    app_result
}

#[derive(Debug)]
struct Err;

impl Err {}

struct NewTodoState {
    visible: bool,
}

impl NewTodoState {
    fn new() -> Self {
        Self { visible: false }
    }
}

struct App {
    todos: Vec<entities::Todo>,
    todos_list_state: ListState,
    new_todo_state: NewTodoState,
}

impl App {
    fn new() -> Self {
        Self {
            todos: Vec::new(),
            todos_list_state: ListState::default().with_selected(Some(0)),
            new_todo_state: NewTodoState::new(),
        }
    }

    fn run(mut self, mut terminal: DefaultTerminal) -> Result<(), Err> {
        self.load_all_todos();
        loop {
            let _ = terminal.draw(|frame| self.draw(frame));

            if self.new_todo_state.visible {
                let _ = self.handle_new_todo_keys();
            } else {
                if let Ok(Event::Key(key)) = event::read() {
                    match key.code {
                        KeyCode::Char('q') => {
                            return Ok(());
                        }
                        KeyCode::Char('n') => self.new_todo_state.visible = true,
                        KeyCode::Char('j') => self.next_row(),
                        KeyCode::Char('k') => self.prev_row(),
                        KeyCode::Char('d') => self.delete_todo(),
                        KeyCode::Char(' ') => match self.todos_list_state.selected() {
                            Some(selected) => {
                                let todo = self.todos.get_mut(selected).unwrap();
                                todo.done = !todo.done;
                            }
                            None => {}
                        },
                        _ => {}
                    }
                }
            }
        }
    }

    fn handle_new_todo_keys(&mut self) -> Result<(), Err> {
        if let Ok(Event::Key(key)) = event::read() {
            match key.code {
                KeyCode::Esc => self.new_todo_state.visible = false,
                KeyCode::Enter => self.create_todo(),
                _ => {}
            }
        }
        Ok(())
    }

    fn next_row(&mut self) {
        match self.todos_list_state.selected() {
            Some(selected) => {
                if selected == self.todos.len() - 1 {
                    self.todos_list_state.select_first();
                } else {
                    self.todos_list_state.select_next();
                }
            }
            None => self.todos_list_state.select_first(),
        }
    }

    fn prev_row(&mut self) {
        match self.todos_list_state.selected() {
            Some(selected) => {
                if selected == 0 {
                    self.todos_list_state.select_last();
                } else {
                    self.todos_list_state.select_previous();
                }
            }
            None => self.todos_list_state.select_first(),
        }
    }

    fn render_todos_list(&mut self, frame: &mut Frame) {
        let block = Block::bordered()
            .title(" Todos ")
            .border_type(BorderType::Rounded);

        let todos = self.todos.iter().map(|todo| {
            let title = todo.title.clone();

            let mut text = if todo.done == true {
                String::from("[X] ")
            } else {
                String::from("[ ] ")
            };

            text.push_str(&title);

            Text::from(text)
        });

        let todos = List::new(todos)
            .block(block)
            .highlight_style(Style::default().add_modifier(Modifier::REVERSED));

        frame.render_stateful_widget(todos, frame.area(), &mut self.todos_list_state);
    }

    fn delete_todo(&mut self) {
        match self.todos_list_state.selected() {
            Some(selected) => {
                self.todos.remove(selected);
                self.save_todos();
            }
            None => {}
        }
    }

    fn draw(&mut self, frame: &mut Frame) {
        self.render_todos_list(frame);
    }

    fn load_all_todos(&mut self) {
        let mut file = File::open(".lazytodo/todos").unwrap();
        let mut buffer = Vec::new();
        let _ = file.read_to_end(&mut buffer);
        let data: Vec<Todo> = bincode::deserialize(&buffer).unwrap_or(vec![]);

        self.todos = data;
    }

    fn create_todo(&mut self) {
        let todo = Todo::new(
            self.get_next_todo_id(),
            String::from("Test Title"),
            String::from("Content"),
        );

        self.todos.push(todo);

        self.save_todos();

        self.new_todo_state.visible = false;
    }

    fn get_next_todo_id(&self) -> usize {
        if self.todos.len() == 0 {
            return 0;
        }

        let mut max_id = 0;

        self.todos.iter().for_each(|todo| {
            if todo.id > max_id {
                max_id = todo.id;
            }
        });

        max_id + 1
    }

    fn get_current_todos(&self) -> Vec<Todo> {
        return vec![];
    }

    fn save_todos(&self) {
        let encoded = bincode::serialize(&self.todos).unwrap();
        let mut file = File::create(".lazytodo/todos").unwrap();
        file.write_all(&encoded).unwrap();
    }
}

// fn get_list() -> Vec<Data> {
//     match load_from_file(".lazytodo/todos") {
//         Ok(list) => list,
//         Err(err) => {
//             eprintln!("Error on load todos file: {:?}", err);
//             Vec::new()
//         }
//     }
// }
//
// fn save_to_file(data: &[Data], filename: &str) -> std::io::Result<()> {
// }
//
// fn load_from_file(filename: &str) -> std::io::Result<Vec<Data>> {
// }
//
// fn get_workspaces() -> Vec<Workspace> {
//     match load_workspaces() {
//         Ok(list) => list,
//         Err(err) => {
//             eprintln!("Error on load workspaces file: {:?}", err);
//             Vec::new()
//         }
//     }
// }
//
// fn load_workspaces() -> std::io::Result<Vec<Workspace>> {
//     let mut file = File::open(".lazytodo/workspaces")?; // Apri il file
//     let mut buffer = Vec::new();
//     file.read_to_end(&mut buffer)?; // Leggi tutto il file in un buffer
//     let data: Vec<Workspace> = bincode::deserialize(&buffer).unwrap(); // Deserializza il contenuto
//     Ok(data)
// }
