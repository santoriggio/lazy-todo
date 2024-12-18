mod entities;

use std::{
    fs::File,
    io::{Read, Write},
    usize,
};

use entities::Workspace;

use chrono::{DateTime, Local, TimeZone, Utc};

use crossterm::event::KeyEvent;
use serde::{Deserialize, Serialize};

use color_eyre::{owo_colors::OwoColorize, Result};
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
use style::palette::tailwind;

const PALETTES: [tailwind::Palette; 4] = [
    tailwind::BLUE,
    tailwind::EMERALD,
    tailwind::INDIGO,
    tailwind::RED,
];
const INFO_TEXT: &str = "Add: a | Delete: d | Done: <space>";

const ITEM_HEIGHT: usize = 2;

fn main() -> Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::init();
    let app_result = App::new().run(terminal);
    ratatui::restore();
    app_result
}
struct TableColors {
    buffer_bg: Color,
    header_bg: Color,
    header_fg: Color,
    row_fg: Color,
    selected_row_style_fg: Color,
    selected_column_style_fg: Color,
    selected_cell_style_fg: Color,
    normal_row_color: Color,
    alt_row_color: Color,
    footer_border_color: Color,
}

impl TableColors {
    const fn new(color: &tailwind::Palette) -> Self {
        Self {
            buffer_bg: tailwind::SLATE.c950,
            header_bg: color.c900,
            header_fg: tailwind::SLATE.c200,
            row_fg: tailwind::SLATE.c200,
            selected_row_style_fg: color.c400,
            selected_column_style_fg: color.c400,
            selected_cell_style_fg: color.c600,
            normal_row_color: tailwind::SLATE.c950,
            alt_row_color: tailwind::SLATE.c900,
            footer_border_color: color.c400,
        }
    }
}

#[derive(PartialEq)]
enum AppTabs {
    Status,
    Inbox,
    Tags,
    Todos,
}

#[derive(Serialize, Deserialize)]
struct Data {
    done: bool,
    text: String,
    created_at: i64,
}

impl Data {
    const fn ref_array(&self) -> (&bool, &String, &i64) {
        (&self.done, &self.text, &self.created_at)
    }
}

struct InboxListItem {
    id: u8,
    text: String,
}

impl InboxListItem {
    fn new(id: u8, text: String) -> Self {
        Self { id, text }
    }
}

struct Inbox {
    list: Vec<InboxListItem>,
    state: ListState,
}

impl Inbox {
    fn new() -> Self {
        Self {
            list: vec![
                InboxListItem::new(0, String::from("Inbox")),
                InboxListItem::new(1, String::from("Today")),
                InboxListItem::new(2, String::from("Tomorrow")),
                InboxListItem::new(3, String::from("This week")),
            ],
            state: ListState::default().with_selected(Some(0)),
        }
    }

    fn on_key_pressed(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('j') => self.scroll_down(),
            KeyCode::Char('k') => self.scroll_up(),
            _ => {}
        }
    }

    fn scroll_down(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.list.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i))
        // self.state = self.state.se(i * ITEM_HEIGHT);
    }

    fn scroll_up(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.list.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i))
    }
}

struct Workspaces {
    current_tab: u8,
    list: Vec<Workspace>,
    input_visible: bool,
    input: String,
    character_index: usize,
    state: ListState,
}
impl Workspaces {
    fn new() -> Self {
        let list = get_workspaces();
        Self {
            list,
            current_tab: 0,
            input_visible: false,
            input: String::new(),
            character_index: 0,
            state: ListState::default().with_selected(Some(0)),
        }
    }

    fn scroll_down(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.list.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i))
        // self.state = self.state.se(i * ITEM_HEIGHT);
    }

    fn scroll_up(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.list.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i))
    }

    fn on_key_pressed(&mut self, key: KeyEvent) {
        match self.input_visible {
            true => match key.code {
                KeyCode::Esc => {
                    self.input_visible = false;
                    self.input.clear();
                    self.character_index = 0;
                }
                KeyCode::Enter => self.submit_input(),
                KeyCode::Char(to_insert) => self.enter_char(to_insert),
                KeyCode::Backspace => self.delete_char(),
                KeyCode::Left => self.move_cursor_left(),
                KeyCode::Right => self.move_cursor_right(),
                _ => {}
            },
            false => match key.code {
                KeyCode::Char('n') => self.input_visible = true,
                KeyCode::Char('d') => self.delete_current_workspace(),
                KeyCode::Char('j') => self.scroll_down(),
                KeyCode::Char('k') => self.scroll_up(),
                KeyCode::Char(']') => {
                    if self.current_tab == 1 {
                        self.current_tab = 0
                    } else {
                        self.current_tab = 1
                    }
                }
                KeyCode::Char('[') => {
                    if self.current_tab == 0 {
                        self.current_tab = 1
                    } else {
                        self.current_tab = 0
                    }
                }
                _ => {}
            },
        }
    }

    fn save_workspaces(&self) -> std::io::Result<()> {
        let encoded = bincode::serialize(&self.list).unwrap(); // Serializza i dati in binario
        let mut file = File::create(".lazytodo/workspaces")?; // Crea o sovrascrivi il file
        file.write_all(&encoded)?; // Scrivi il contenuto serializzato
        Ok(())
    }

    fn delete_current_workspace(&mut self) {
        match self.state.selected() {
            Some(idx) => {
                let _ = self.list.remove(idx);
            }
            None => eprintln!("Not found"),
        };

        let _ = self.save_workspaces();
    }

    fn submit_input(&mut self) {
        self.list.push(Workspace::new(self.input.clone()));

        self.input_visible = false;
        self.input.clear();
        self.reset_cursor();

        let _ = self.save_workspaces();
    }
    fn reset_cursor(&mut self) {
        self.character_index = 0;
    }

    fn move_cursor_left(&mut self) {
        let cursor_moved_left = self.character_index.saturating_sub(1);
        self.character_index = self.clamp_cursor(cursor_moved_left);
    }

    fn move_cursor_right(&mut self) {
        let cursor_moved_right = self.character_index.saturating_add(1);
        self.character_index = self.clamp_cursor(cursor_moved_right);
    }

    fn enter_char(&mut self, new_char: char) {
        let index = self.byte_index();
        self.input.insert(index, new_char);
        self.move_cursor_right();
    }

    fn byte_index(&self) -> usize {
        self.input
            .char_indices()
            .map(|(i, _)| i)
            .nth(self.character_index)
            .unwrap_or(self.input.len())
    }
    fn clamp_cursor(&self, new_cursor_pos: usize) -> usize {
        new_cursor_pos.clamp(0, self.input.chars().count())
    }
    fn delete_char(&mut self) {
        let is_not_cursor_leftmost = self.character_index != 0;
        if is_not_cursor_leftmost {
            // Method "remove" is not used on the saved text for deleting the selected char.
            // Reason: Using remove on String works on bytes instead of the chars.
            // Using remove would require special care because of char boundaries.

            let current_index = self.character_index;
            let from_left_to_current_index = current_index - 1;

            // Getting all characters before the selected character.
            let before_char_to_delete = self.input.chars().take(from_left_to_current_index);
            // Getting all characters after selected character.
            let after_char_to_delete = self.input.chars().skip(current_index);

            // Put all characters together except the selected one.
            // By leaving the selected one out, it is forgotten and therefore deleted.
            self.input = before_char_to_delete.chain(after_char_to_delete).collect();
            self.move_cursor_left();
        }
    }
}
struct App {
    state: TableState,
    items: Vec<Data>,
    scroll_state: ScrollbarState,
    colors: TableColors,
    color_index: usize,
    input_visible: bool,
    input: String,
    character_index: usize,
    current_tab: AppTabs,
    inbox: Inbox,
    workspaces: Workspaces,
}

impl App {
    fn new() -> Self {
        let data_vec = get_list();
        Self {
            state: TableState::default().with_selected(0),
            scroll_state: ScrollbarState::new(0),
            colors: TableColors::new(&PALETTES[0]),
            color_index: 0,
            items: data_vec,
            input_visible: false,
            input: String::new(),
            character_index: 0,
            current_tab: AppTabs::Inbox,
            inbox: Inbox::new(),
            workspaces: Workspaces::new(),
        }
    }
    pub fn next_row(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
        self.scroll_state = self.scroll_state.position(i * ITEM_HEIGHT);
    }

    pub fn previous_row(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
        self.scroll_state = self.scroll_state.position(i * ITEM_HEIGHT);
    }

    pub fn next_column(&mut self) {
        self.state.select_next_column();
    }

    pub fn previous_column(&mut self) {
        self.state.select_previous_column();
    }

    pub fn set_colors(&mut self) {
        self.colors = TableColors::new(&PALETTES[self.color_index]);
    }

    fn submit_message(&mut self) {
        self.items.push(Data {
            done: false,
            text: self.input.clone(),
            created_at: Local::now().timestamp_millis(),
        });
        self.input.clear();
        self.reset_cursor();

        self.toggle_input();

        let _ = save_to_file(&self.items, ".lazytodo/todos");

        // if self.items.len() == 1 {
        //     self.scroll_state = self.scroll_state.position(ITEM_HEIGHT);
        // }
    }

    fn reset_cursor(&mut self) {
        self.character_index = 0;
    }

    fn move_cursor_left(&mut self) {
        let cursor_moved_left = self.character_index.saturating_sub(1);
        self.character_index = self.clamp_cursor(cursor_moved_left);
    }

    fn move_cursor_right(&mut self) {
        let cursor_moved_right = self.character_index.saturating_add(1);
        self.character_index = self.clamp_cursor(cursor_moved_right);
    }

    fn enter_char(&mut self, new_char: char) {
        let index = self.byte_index();
        self.input.insert(index, new_char);
        self.move_cursor_right();
    }

    fn byte_index(&self) -> usize {
        self.input
            .char_indices()
            .map(|(i, _)| i)
            .nth(self.character_index)
            .unwrap_or(self.input.len())
    }
    fn clamp_cursor(&self, new_cursor_pos: usize) -> usize {
        new_cursor_pos.clamp(0, self.input.chars().count())
    }
    fn delete_char(&mut self) {
        let is_not_cursor_leftmost = self.character_index != 0;
        if is_not_cursor_leftmost {
            // Method "remove" is not used on the saved text for deleting the selected char.
            // Reason: Using remove on String works on bytes instead of the chars.
            // Using remove would require special care because of char boundaries.

            let current_index = self.character_index;
            let from_left_to_current_index = current_index - 1;

            // Getting all characters before the selected character.
            let before_char_to_delete = self.input.chars().take(from_left_to_current_index);
            // Getting all characters after selected character.
            let after_char_to_delete = self.input.chars().skip(current_index);

            // Put all characters together except the selected one.
            // By leaving the selected one out, it is forgotten and therefore deleted.
            self.input = before_char_to_delete.chain(after_char_to_delete).collect();
            self.move_cursor_left();
        }
    }

    fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        loop {
            terminal.draw(|frame| self.draw(frame))?;

            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    // let shift_pressed = key.modifiers.contains(KeyModifiers::SHIFT);
                    match self.input_visible {
                        true => match key.code {
                            KeyCode::Esc => self.toggle_input(),
                            KeyCode::Enter => self.submit_message(),
                            KeyCode::Char(to_insert) => self.enter_char(to_insert),
                            KeyCode::Backspace => self.delete_char(),
                            KeyCode::Left => self.move_cursor_left(),
                            KeyCode::Right => self.move_cursor_right(),
                            _ => {}
                        },
                        false => match key.code {
                            KeyCode::Char('q') => {
                                return Ok(());
                            }
                            KeyCode::Tab => self.toggle_next_tab(),
                            KeyCode::Char('1') => self.current_tab = AppTabs::Status,
                            KeyCode::Char('2') => self.current_tab = AppTabs::Inbox,
                            KeyCode::Char('3') => self.current_tab = AppTabs::Tags,
                            KeyCode::Char('4') => self.current_tab = AppTabs::Todos,
                            _ => match self.current_tab {
                                AppTabs::Inbox => self.inbox.on_key_pressed(key),
                                AppTabs::Tags => self.workspaces.on_key_pressed(key),
                                _ => {}
                            }, // KeyCode::Tab => self.toggle_next_tab(),
                               // KeyCode::Char('a') => self.toggle_input(),
                               // KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                               // KeyCode::Char('j') | KeyCode::Down => self.next_row(),
                               // KeyCode::Char('k') | KeyCode::Up => self.previous_row(),
                               // KeyCode::Char('d') => {
                               //     if let Some(idx) = self.state.selected() {
                               //         self.delete_todo(idx);
                               //     }
                               // }
                               // KeyCode::Char(' ') => {
                               //     if let Some(idx) = self.state.selected() {
                               //         self.toggle_todo(idx);
                               //     }
                               // }
                               // KeyCode::Char('l') | KeyCode::Right => self.next_column(),
                               // KeyCode::Char('h') | KeyCode::Left => self.previous_column(),
                               // _ => {}
                        },
                    }
                }
            }
        }
    }

    fn toggle_next_tab(&mut self) {
        match self.current_tab {
            AppTabs::Status => self.current_tab = AppTabs::Inbox,
            AppTabs::Inbox => self.current_tab = AppTabs::Tags,
            AppTabs::Tags => self.current_tab = AppTabs::Todos,
            AppTabs::Todos => self.current_tab = AppTabs::Status,
        }
    }

    fn toggle_todo(&mut self, idx: usize) {
        if let Some(item) = self.items.get_mut(idx) {
            item.done = !item.done;
            let _ = save_to_file(&self.items, ".lazytodo/todos");
        }
    }

    fn delete_todo(&mut self, idx: usize) {
        self.items.remove(idx);
        let _ = save_to_file(&self.items, ".lazytodo/todos");
    }

    fn toggle_input(&mut self) {
        self.input_visible = !self.input_visible
    }

    fn draw(&mut self, frame: &mut Frame) {
        let main_vertical =
            Layout::vertical([Constraint::Percentage(100), Constraint::Min(3)]).split(frame.area());

        let horizontal_layout =
            Layout::horizontal([Constraint::Ratio(1, 3), Constraint::Ratio(2, 3)])
                .split(main_vertical[0]);

        self.set_colors();

        self.render_drawer(frame, horizontal_layout[0]);
        self.render_table(frame, horizontal_layout[1]);
        self.render_scrollbar(frame, horizontal_layout[1]);
        self.render_footer(frame, main_vertical[1]);

        if self.input_visible {
            self.render_input(frame);
        }

        if self.workspaces.input_visible {
            self.render_workspaces_input(frame);
        }
    }

    fn render_drawer(&mut self, frame: &mut Frame, area: Rect) {
        let vertical_layout = Layout::vertical([
            Constraint::Min(4),
            Constraint::Percentage(40),
            Constraint::Percentage(40),
        ])
        .split(area);

        let status_block = Block::bordered()
            .border_type(BorderType::Rounded)
            .title("[1] Status ")
            .fg(match self.current_tab {
                AppTabs::Status => Color::Green,
                _ => Color::default(),
            });
        let inbox_block = Block::bordered()
            .border_type(BorderType::Rounded)
            .title("[2] Inbox ")
            .fg(match self.current_tab {
                AppTabs::Inbox => Color::Green,
                _ => Color::default(),
            });

        let is_active_tab = if self.current_tab == AppTabs::Tags {
            true
        } else {
            false
        };

        let workspaces_block = Block::bordered()
            .border_type(BorderType::Rounded)
            .fg(match self.current_tab {
                AppTabs::Tags => Color::Green,
                _ => Color::default(),
            })
            .title(vec![
                Span::from("[3] "),
                Span::styled(
                    "Workspaces",
                    if is_active_tab && self.workspaces.current_tab == 0 {
                        Style::default().fg(Color::Green).bold()
                    } else {
                        Style::default().fg(Color::default())
                    },
                ),
                Span::styled(" - ", Style::default().fg(Color::default())),
                Span::styled(
                    "Tags",
                    if is_active_tab && self.workspaces.current_tab == 1 {
                        Style::default().fg(Color::Green).bold()
                    } else {
                        Style::default().fg(Color::default())
                    },
                ),
                Span::from(" "),
            ]);

        let workspaces_list = self
            .workspaces
            .list
            .iter()
            .map(|item| Text::from(item.title.clone()).fg(Color::default()));

        let workspaces_list = List::new(workspaces_list)
            .block(workspaces_block)
            .highlight_style(Style::new().add_modifier(Modifier::REVERSED));

        let inbox_list = self
            .inbox
            .list
            .iter()
            .map(|item| Text::from(item.text.clone()).fg(Color::default()));

        let inbox_list = List::new(inbox_list)
            .block(inbox_block)
            .highlight_style(Style::new().add_modifier(Modifier::REVERSED));

        frame.render_widget(status_block, vertical_layout[0]);
        frame.render_stateful_widget(inbox_list, vertical_layout[1], &mut self.inbox.state);
        frame.render_stateful_widget(
            workspaces_list,
            vertical_layout[2],
            &mut self.workspaces.state,
        );
    }

    fn render_table(&mut self, frame: &mut Frame, area: Rect) {
        let block = Block::bordered()
            .border_type(BorderType::Rounded)
            .title("[4] Todos ")
            .fg(match self.current_tab {
                AppTabs::Todos => Color::Green,
                _ => Color::default(),
            });
        let selected_row_style = Style::default().add_modifier(Modifier::REVERSED);
        let selected_col_style = Style::default();
        let selected_cell_style = Style::default().add_modifier(Modifier::REVERSED);

        let rows = self.items.iter().enumerate().map(|(i, data)| {
            let item = data.ref_array();
            let done_text = if *item.0 == true { "[x]" } else { "[ ]" };
            let text = format!("{}", item.1);

            let created_at = DateTime::from_timestamp_millis(*item.2);
            let created_at = match created_at {
                Some(d) => format!("{}", d.with_timezone(&Local).format("%Y-%m-%d %H:%M")),
                None => String::from(""),
            };
            Row::new(vec![
                Cell::from(done_text),
                Cell::from(text),
                Cell::from(created_at),
            ])
            .fg(Color::default())
            .height(ITEM_HEIGHT as u16)
        });

        let t = Table::new(
            rows,
            [
                Constraint::Min(3),
                Constraint::Percentage(100),
                Constraint::Min(17),
            ],
        )
        .block(block)
        .header(
            Row::new(vec![
                Cell::from(""),
                Cell::from(""),
                Cell::from(Text::from("Created At").centered().bold()),
            ])
            .fg(Color::default())
            .bottom_margin(1),
        )
        .column_spacing(1)
        .row_highlight_style(selected_row_style)
        .column_highlight_style(selected_col_style)
        .cell_highlight_style(selected_cell_style)
        .highlight_spacing(HighlightSpacing::Always);

        frame.render_stateful_widget(t, area, &mut self.state);
    }

    fn render_scrollbar(&mut self, frame: &mut Frame, area: Rect) {
        frame.render_stateful_widget(
            Scrollbar::default()
                .orientation(ScrollbarOrientation::VerticalRight)
                .begin_symbol(None)
                .end_symbol(None),
            area.inner(Margin {
                vertical: 1,
                horizontal: 1,
            }),
            &mut self.scroll_state,
        );
    }

    fn render_footer(&self, frame: &mut Frame, area: Rect) {
        let block = Block::bordered().border_type(BorderType::Double);
        let info_footer = Paragraph::new(INFO_TEXT).block(block).centered();
        frame.render_widget(info_footer, area);
    }

    fn render_input(&self, frame: &mut Frame) {
        let area = frame.area();
        let input_area = Rect {
            x: area.width / 4,
            y: area.height / 3,
            width: area.width / 2,
            height: area.height / 3,
        };

        let popup = Paragraph::new(Text::from(self.input.as_str()).fg(Color::Gray))
            .wrap(Wrap { trim: true })
            .block(
                Block::bordered()
                    .title("New Todo")
                    .border_type(BorderType::Rounded),
            );

        frame.render_widget(popup, input_area);
        frame.set_cursor_position(Position::new(
            // Draw the cursor at the current position in the input field.
            // This position is can be controlled via the left and right arrow key
            input_area.x + self.character_index as u16 + 1,
            // Move one line down, from the border to the input line
            input_area.y + 1,
        ));
    }
    fn render_workspaces_input(&self, frame: &mut Frame) {
        let area = frame.area();
        let input_area = Rect {
            x: area.width / 4,
            y: area.height / 3,
            width: area.width / 2,
            height: area.height / 3,
        };

        let popup = Paragraph::new(Text::from(self.workspaces.input.as_str()).fg(Color::Gray))
            .wrap(Wrap { trim: true })
            .block(
                Block::bordered()
                    .title("New Workspace")
                    .border_type(BorderType::Rounded),
            );

        frame.render_widget(popup, input_area);
        frame.set_cursor_position(Position::new(
            // Draw the cursor at the current position in the input field.
            // This position is can be controlled via the left and right arrow key
            input_area.x + self.workspaces.character_index as u16 + 1,
            // Move one line down, from the border to the input line
            input_area.y + 1,
        ));
    }
}

fn get_list() -> Vec<Data> {
    match load_from_file(".lazytodo/todos") {
        Ok(list) => list,
        Err(err) => {
            eprintln!("Error on load todos file: {:?}", err);
            Vec::new()
        }
    }
}

fn save_to_file(data: &[Data], filename: &str) -> std::io::Result<()> {
    let encoded = bincode::serialize(data).unwrap(); // Serializza i dati in binario
    let mut file = File::create(filename)?; // Crea o sovrascrivi il file
    file.write_all(&encoded)?; // Scrivi il contenuto serializzato
    Ok(())
}

fn load_from_file(filename: &str) -> std::io::Result<Vec<Data>> {
    let mut file = File::open(filename)?; // Apri il file
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?; // Leggi tutto il file in un buffer
    let data: Vec<Data> = bincode::deserialize(&buffer).unwrap(); // Deserializza il contenuto
    Ok(data)
}

fn get_workspaces() -> Vec<Workspace> {
    match load_workspaces() {
        Ok(list) => list,
        Err(err) => {
            eprintln!("Error on load workspaces file: {:?}", err);
            Vec::new()
        }
    }
}

fn load_workspaces() -> std::io::Result<Vec<Workspace>> {
    let mut file = File::open(".lazytodo/workspaces")?; // Apri il file
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?; // Leggi tutto il file in un buffer
    let data: Vec<Workspace> = bincode::deserialize(&buffer).unwrap(); // Deserializza il contenuto
    Ok(data)
}
