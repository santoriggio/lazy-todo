use std::{
    fs::File,
    io::{Read, Write},
};

use serde::{Deserialize, Serialize};

use color_eyre::{eyre::ContextCompat, owo_colors::OwoColorize, Result};
use ratatui::{
    crossterm::event::{self, Event, KeyCode, KeyEventKind},
    layout::{Constraint, Layout, Margin, Position, Rect},
    style::{self, Color, Modifier, Style, Styled, Stylize},
    text::Text,
    widgets::{
        Block, BorderType, Borders, Cell, HighlightSpacing, Paragraph, Row, Scrollbar,
        ScrollbarOrientation, ScrollbarState, Table, TableState, Wrap,
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
const INFO_TEXT: [&str; 2] = [
    "Quit: q | Move up: k | Move down: j",
    "Add: a | Delete: d | Done: <space>",
];

const ITEM_HEIGHT: usize = 1;

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

#[derive(Serialize, Deserialize)]
struct Data {
    done: bool,
    text: String,
}

impl Data {
    const fn ref_array(&self) -> (&bool, &String) {
        (&self.done, &self.text)
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
                            KeyCode::Char('a') => self.toggle_input(),
                            KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                            KeyCode::Char('j') | KeyCode::Down => self.next_row(),
                            KeyCode::Char('k') | KeyCode::Up => self.previous_row(),
                            KeyCode::Char('d') => {
                                if let Some(idx) = self.state.selected() {
                                    self.delete_todo(idx);
                                }
                            }
                            KeyCode::Char(' ') => {
                                if let Some(idx) = self.state.selected() {
                                    self.toggle_todo(idx);
                                }
                            }
                            KeyCode::Char('l') | KeyCode::Right => self.next_column(),
                            KeyCode::Char('h') | KeyCode::Left => self.previous_column(),
                            _ => {}
                        },
                    }
                }
            }
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
            Layout::vertical([Constraint::Percentage(90), Constraint::Min(4)]).split(frame.area());

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
            .title("[1] Status")
            .bg(self.colors.buffer_bg)
            .fg(self.colors.header_bg);
        let inbox_block = Block::bordered()
            .border_type(BorderType::Rounded)
            .title("[2] Inbox")
            .bg(self.colors.buffer_bg)
            .fg(self.colors.header_bg);
        let tags_block = Block::bordered()
            .border_type(BorderType::Rounded)
            .title("[3] Tags")
            .bg(self.colors.buffer_bg)
            .fg(self.colors.header_bg);

        frame.render_widget(status_block, vertical_layout[0]);
        frame.render_widget(inbox_block, vertical_layout[1]);
        frame.render_widget(tags_block, vertical_layout[2]);
    }

    fn render_table(&mut self, frame: &mut Frame, area: Rect) {
        let block = Block::bordered()
            .border_type(BorderType::Rounded)
            .style(Style::default().fg(self.colors.header_bg))
            .title("[4] Todos");
        let selected_row_style = Style::default()
            .add_modifier(Modifier::REVERSED)
            .fg(self.colors.selected_row_style_fg);
        let selected_col_style = Style::default().fg(self.colors.selected_column_style_fg);
        let selected_cell_style = Style::default()
            .add_modifier(Modifier::REVERSED)
            .fg(self.colors.selected_cell_style_fg);

        let rows = self.items.iter().enumerate().map(|(i, data)| {
            let item = data.ref_array();
            let done_text = if *item.0 == true { "[x]" } else { "[ ]" };
            let text = format!("{}", item.1);
            Row::new(vec![Cell::from(done_text), Cell::from(text)])
                .fg(match *item.0 {
                    true => Color::Blue,
                    false => Color::default(),
                })
                .height(1)
        });
        let t = Table::new(rows, [Constraint::Length(4), Constraint::Min(1)])
            .block(block)
            .row_highlight_style(selected_row_style)
            .column_highlight_style(selected_col_style)
            .cell_highlight_style(selected_cell_style)
            .bg(self.colors.buffer_bg)
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
        let info_footer = Paragraph::new(Text::from_iter(INFO_TEXT))
            .style(
                Style::new()
                    .fg(self.colors.row_fg)
                    .bg(self.colors.buffer_bg),
            )
            .centered()
            .block(
                Block::bordered()
                    .border_type(BorderType::Double)
                    .border_style(Style::new().fg(self.colors.footer_border_color)),
            );
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
                    .fg(Color::Blue)
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
