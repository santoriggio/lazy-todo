mod entities;

use crossterm::event::KeyEvent;
use entities::Todo;

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

fn main() {
    let terminal = ratatui::init();
    let app_result = App::new().run(terminal);
    ratatui::restore();
    app_result
}

struct App {
    todos: Vec<entities::Todo>,
    todos_list_state: ListState,
}

impl App {
    fn new() -> Self {
        Self {
            todos: Vec::new(),
            todos_list_state: ListState::default(),
        }
    }

    fn run(mut self, mut terminal: DefaultTerminal) {
        loop {
            let _ = terminal.draw(|frame| self.draw(frame));

            if let Event::Key(key) = event::read() {
                match key.code {
                    KeyCode::Char('q') => {
                        return ();
                    }
                }
            }
        }
    }

    fn render_todos_list(&mut self, frame: &mut Frame) {
        let block = Block::bordered();

        let todos = self.todos.iter().map(|todo| Text::from(todo.title.clone()));

        let todos =
            List::new(todos).highlight_style(Style::default().add_modifier(Modifier::REVERSED));

        frame.render_stateful_widget(todos, frame.area(), &mut self.todos_list_state);
    }

    fn draw(&mut self, frame: &mut Frame) {
        self.render_todos_list(frame);
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
//     let encoded = bincode::serialize(data).unwrap(); // Serializza i dati in binario
//     let mut file = File::create(filename)?; // Crea o sovrascrivi il file
//     file.write_all(&encoded)?; // Scrivi il contenuto serializzato
//     Ok(())
// }
//
// fn load_from_file(filename: &str) -> std::io::Result<Vec<Data>> {
//     let mut file = File::open(filename)?; // Apri il file
//     let mut buffer = Vec::new();
//     file.read_to_end(&mut buffer)?; // Leggi tutto il file in un buffer
//     let data: Vec<Data> = bincode::deserialize(&buffer).unwrap(); // Deserializza il contenuto
//     Ok(data)
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
