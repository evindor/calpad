mod editor;
mod highlight;
mod notes;
mod theme;

use std::collections::HashMap;
use std::io;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use ratatui::Frame;
use ratatui::crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::layout::{Constraint, Layout, Position, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, List, ListItem, ListState, Paragraph};

use calpad_core::types::Value;

use editor::Editor;
use notes::{Config, NoteStore};
use theme::THEMES;

const SIDEBAR_WIDTH: u16 = 24;

const DEFAULT_CONTENT: &str = "\
# Welcome to Calpad

// Arithmetic
2 + 2
sqrt 144
10!

// Units
1 km in miles
5 kg in pounds
72 fahrenheit in celsius

// Currency
$100 in EUR

// Variables
rate = 0.15
price = $49.99
Tax: price * rate
Total: price + tax

// Percentages
$200 - 15%
50 as a % of 200
";

#[derive(PartialEq, Clone, Copy)]
enum Focus {
    Editor,
    Sidebar,
}

struct App {
    editor: Editor,
    notes: NoteStore,
    config: Config,
    results: Vec<calpad_core::types::LineResult>,
    rates: HashMap<String, f64>,
    theme_index: usize,
    sidebar_visible: bool,
    focus: Focus,
    should_quit: bool,
    sidebar_selected: usize,
    delete_confirm: bool,
    rates_loaded: bool,
}

impl App {
    fn new() -> Self {
        let notes = NoteStore::load();
        let config = Config::load();
        let theme_index = THEMES
            .iter()
            .position(|t| t.name == config.theme)
            .unwrap_or(0);

        let mut app = App {
            editor: Editor::new(),
            notes,
            sidebar_visible: config.sidebar_visible,
            config,
            results: Vec::new(),
            rates: HashMap::new(),
            theme_index,
            focus: Focus::Editor,
            should_quit: false,
            sidebar_selected: 0,
            delete_confirm: false,
            rates_loaded: false,
        };

        // Load active note or create default
        let active_id = app.config.active_note_id.clone();
        if app.notes.get(&active_id).is_some() {
            let content = app.notes.get(&active_id).unwrap().content.clone();
            app.editor.set_content(&content);
            app.sidebar_selected = app.notes.selected_index(&active_id).unwrap_or(0);
        } else if let Some(note) = app.notes.notes.first() {
            let id = note.id.clone();
            let content = note.content.clone();
            app.config.active_note_id = id;
            app.editor.set_content(&content);
        } else {
            let id = app.notes.create(DEFAULT_CONTENT.to_string());
            app.config.active_note_id = id;
            app.editor.set_content(DEFAULT_CONTENT);
        }

        app.evaluate();
        app
    }

    fn theme(&self) -> &theme::Theme {
        &THEMES[self.theme_index]
    }

    fn evaluate(&mut self) {
        let content = self.editor.content();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs_f64())
            .ok();
        self.results = calpad_core::evaluate_document_full(&content, &self.rates, now);
    }

    fn save_current_note(&mut self) {
        let content = self.editor.content();
        self.notes.update(&self.config.active_note_id, &content);
        self.notes.save();
    }

    fn handle_key(&mut self, key: KeyEvent) {
        // Global shortcuts
        if key.modifiers.contains(KeyModifiers::CONTROL) {
            match key.code {
                KeyCode::Char('q') | KeyCode::Char('c') => {
                    self.should_quit = true;
                    return;
                }
                KeyCode::Char('b') => {
                    self.toggle_sidebar();
                    return;
                }
                KeyCode::Char('n') => {
                    self.new_note();
                    return;
                }
                KeyCode::Char('t') => {
                    self.cycle_theme();
                    return;
                }
                _ => {}
            }
        }

        match self.focus {
            Focus::Editor => self.handle_editor_key(key),
            Focus::Sidebar => self.handle_sidebar_key(key),
        }
    }

    fn handle_editor_key(&mut self, key: KeyEvent) {
        let modified = match key.code {
            KeyCode::Char(c)
                if key.modifiers.is_empty() || key.modifiers == KeyModifiers::SHIFT =>
            {
                self.editor.insert_char(c);
                true
            }
            KeyCode::Backspace => {
                self.editor.backspace();
                true
            }
            KeyCode::Delete => {
                self.editor.delete();
                true
            }
            KeyCode::Enter => {
                self.editor.newline();
                true
            }
            KeyCode::Tab => {
                self.editor.insert_char(' ');
                self.editor.insert_char(' ');
                true
            }
            KeyCode::Left => {
                self.editor.move_left();
                false
            }
            KeyCode::Right => {
                self.editor.move_right();
                false
            }
            KeyCode::Up => {
                self.editor.move_up();
                false
            }
            KeyCode::Down => {
                self.editor.move_down();
                false
            }
            KeyCode::Home => {
                self.editor.move_home();
                false
            }
            KeyCode::End => {
                self.editor.move_end();
                false
            }
            KeyCode::PageUp => {
                self.editor.page_up();
                false
            }
            KeyCode::PageDown => {
                self.editor.page_down();
                false
            }
            KeyCode::Esc => {
                if self.sidebar_visible {
                    self.focus = Focus::Sidebar;
                }
                false
            }
            _ => false,
        };

        if modified {
            self.evaluate();
            self.save_current_note();
        }
    }

    fn handle_sidebar_key(&mut self, key: KeyEvent) {
        // Cancel delete confirmation on any key except 'd'
        if self.delete_confirm && key.code != KeyCode::Char('d') {
            self.delete_confirm = false;
        }

        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                if !self.notes.notes.is_empty() {
                    self.sidebar_selected = self.sidebar_selected.saturating_sub(1);
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if !self.notes.notes.is_empty() {
                    self.sidebar_selected =
                        (self.sidebar_selected + 1).min(self.notes.notes.len() - 1);
                }
            }
            KeyCode::Enter => {
                self.select_note(self.sidebar_selected);
                self.focus = Focus::Editor;
            }
            KeyCode::Char('d') => {
                if self.delete_confirm {
                    self.delete_selected_note();
                    self.delete_confirm = false;
                } else {
                    self.delete_confirm = true;
                }
            }
            KeyCode::Esc | KeyCode::Right => {
                self.focus = Focus::Editor;
            }
            _ => {}
        }
    }

    fn select_note(&mut self, index: usize) {
        if index >= self.notes.notes.len() {
            return;
        }
        self.save_current_note();
        let note = &self.notes.notes[index];
        self.config.active_note_id = note.id.clone();
        let content = note.content.clone();
        self.editor.set_content(&content);
        self.sidebar_selected = index;
        self.evaluate();
    }

    fn new_note(&mut self) {
        self.save_current_note();
        let id = self.notes.create(String::new());
        self.config.active_note_id = id;
        self.editor.set_content("");
        self.sidebar_selected = 0;
        self.results.clear();
        self.focus = Focus::Editor;
    }

    fn delete_selected_note(&mut self) {
        if self.notes.notes.is_empty() {
            return;
        }
        let id = self.notes.notes[self.sidebar_selected].id.clone();
        self.notes.delete(&id);

        if self.notes.notes.is_empty() {
            let new_id = self.notes.create(DEFAULT_CONTENT.to_string());
            self.config.active_note_id = new_id;
            self.editor.set_content(DEFAULT_CONTENT);
            self.sidebar_selected = 0;
        } else {
            self.sidebar_selected = self.sidebar_selected.min(self.notes.notes.len() - 1);
            let note = &self.notes.notes[self.sidebar_selected];
            self.config.active_note_id = note.id.clone();
            self.editor.set_content(&note.content);
        }
        self.evaluate();
    }

    fn toggle_sidebar(&mut self) {
        self.sidebar_visible = !self.sidebar_visible;
        if !self.sidebar_visible {
            self.focus = Focus::Editor;
        }
        self.config.sidebar_visible = self.sidebar_visible;
    }

    fn cycle_theme(&mut self) {
        self.theme_index = (self.theme_index + 1) % THEMES.len();
        self.config.theme = self.theme().name.to_string();
    }

    fn set_rates(&mut self, rates: HashMap<String, f64>) {
        self.rates = rates;
        self.rates_loaded = true;
        self.evaluate();
    }

    fn save_all(&mut self) {
        self.save_current_note();
        self.config.save();
    }
}

// ── Rendering ──────────────────────────────────────────────────────────

fn ui(frame: &mut Frame, app: &App) {
    let theme = app.theme();

    // Clear with bg color
    let bg_block = Block::default().style(Style::default().bg(theme.bg));
    frame.render_widget(bg_block, frame.area());

    // Split: main area + status bar
    let chunks = Layout::vertical([Constraint::Min(0), Constraint::Length(1)]).split(frame.area());
    let main_area = chunks[0];
    let status_area = chunks[1];

    if app.sidebar_visible {
        let cols = Layout::horizontal([Constraint::Length(SIDEBAR_WIDTH), Constraint::Min(0)])
            .split(main_area);
        render_sidebar(frame, cols[0], app);
        render_editor(frame, cols[1], app);
    } else {
        render_editor(frame, main_area, app);
    }

    render_status(frame, status_area, app);
}

fn render_sidebar(frame: &mut Frame, area: Rect, app: &App) {
    let theme = app.theme();
    let max_title_len = area.width.saturating_sub(4) as usize;

    let items: Vec<ListItem> = app
        .notes
        .notes
        .iter()
        .enumerate()
        .map(|(i, note)| {
            let title = if note.title.chars().count() > max_title_len {
                let t: String = note.title.chars().take(max_title_len - 1).collect();
                format!(" {t}\u{2026}")
            } else {
                format!(" {}", note.title)
            };

            // Show delete confirmation
            if app.delete_confirm && i == app.sidebar_selected {
                ListItem::new(Line::from(vec![Span::styled(
                    " press d to delete",
                    Style::default().fg(theme.error),
                )]))
            } else {
                ListItem::new(title)
            }
        })
        .collect();

    let list = List::new(items)
        .block(Block::default())
        .style(Style::default().bg(theme.bg_sidebar).fg(theme.fg_muted))
        .highlight_style(Style::default().bg(theme.bg_active).fg(theme.fg));

    let mut state = ListState::default();
    state.select(Some(app.sidebar_selected));
    frame.render_stateful_widget(list, area, &mut state);
}

fn render_editor(frame: &mut Frame, area: Rect, app: &App) {
    let theme = app.theme();

    if area.width < 4 || area.height < 2 {
        return;
    }

    // 1-cell left padding
    let padded = Rect {
        x: area.x + 1,
        y: area.y,
        width: area.width.saturating_sub(2),
        height: area.height,
    };

    let editor_width = padded.width as usize;

    let display_lines: Vec<Line> = app
        .editor
        .lines
        .iter()
        .enumerate()
        .map(|(i, line_text)| {
            let result = app.results.iter().find(|r| r.line_index == i);
            let mut spans = highlight::highlight_line_spans(line_text, theme);

            if let Some(r) = result {
                if !r.display.is_empty() {
                    let input_width = line_text.chars().count();
                    let result_width = r.display.chars().count();
                    let padding = editor_width
                        .saturating_sub(input_width + result_width)
                        .max(2);

                    spans.push(Span::raw(" ".repeat(padding)));

                    let result_style = if matches!(r.value, Value::Error(_)) {
                        Style::default().fg(theme.error)
                    } else {
                        Style::default().fg(theme.result)
                    };
                    spans.push(Span::styled(r.display.clone(), result_style));
                }
            }

            Line::from(spans)
        })
        .collect();

    let paragraph = Paragraph::new(display_lines)
        .style(Style::default().bg(theme.bg).fg(theme.fg))
        .scroll((app.editor.scroll_offset as u16, 0));

    frame.render_widget(paragraph, padded);

    // Cursor
    if app.focus == Focus::Editor {
        let cursor_x = padded.x + app.editor.cursor_col as u16;
        let cursor_y = padded.y
            + app
                .editor
                .cursor_row
                .saturating_sub(app.editor.scroll_offset) as u16;
        if cursor_y < padded.y + padded.height && cursor_x < padded.x + padded.width {
            frame.set_cursor_position(Position::new(cursor_x, cursor_y));
        }
    }
}

fn render_status(frame: &mut Frame, area: Rect, app: &App) {
    let theme = app.theme();

    let mut spans = vec![
        Span::styled(" ^B", Style::default().fg(theme.fg)),
        Span::styled(" sidebar  ", Style::default().fg(theme.fg_muted)),
        Span::styled("^N", Style::default().fg(theme.fg)),
        Span::styled(" new  ", Style::default().fg(theme.fg_muted)),
        Span::styled("^T", Style::default().fg(theme.fg)),
        Span::styled(
            format!(" {}  ", app.theme().name),
            Style::default().fg(theme.fg_muted),
        ),
        Span::styled("Esc", Style::default().fg(theme.fg)),
        Span::styled(" focus  ", Style::default().fg(theme.fg_muted)),
        Span::styled("^Q", Style::default().fg(theme.fg)),
        Span::styled(" quit", Style::default().fg(theme.fg_muted)),
    ];

    // Right-align rates status
    let status_text = if app.rates_loaded {
        " \u{2713}"
    } else {
        " ..."
    };
    let left_len: usize = spans.iter().map(|s| s.content.chars().count()).sum();
    let padding = (area.width as usize).saturating_sub(left_len + status_text.len());
    spans.push(Span::raw(" ".repeat(padding)));
    spans.push(Span::styled(
        status_text,
        Style::default().fg(theme.fg_muted),
    ));

    let line = Line::from(spans);
    let paragraph = Paragraph::new(line).style(Style::default().bg(theme.bg_sidebar));
    frame.render_widget(paragraph, area);
}

// ── Rate fetching ──────────────────────────────────────────────────────

fn spawn_rate_fetcher(tx: mpsc::Sender<HashMap<String, f64>>) {
    // Send cached rates immediately
    if let Some(cached) = notes::load_cached_rates() {
        let _ = tx.send(cached);
    }

    // Fetch fresh rates in background
    thread::spawn(move || {
        if let Ok(rates) = fetch_rates() {
            notes::cache_rates(&rates);
            let _ = tx.send(rates);
        }
    });
}

fn fetch_rates() -> Result<HashMap<String, f64>, Box<dyn std::error::Error + Send + Sync>> {
    let body: String = ureq::get("https://api.frankfurter.app/latest?from=USD")
        .call()?
        .into_string()?;
    let json: serde_json::Value = serde_json::from_str(&body)?;

    let mut rates = HashMap::new();
    rates.insert("USD".to_string(), 1.0);
    if let Some(obj) = json.get("rates").and_then(|r| r.as_object()) {
        for (key, val) in obj {
            if let Some(n) = val.as_f64() {
                rates.insert(key.clone(), n);
            }
        }
    }
    Ok(rates)
}

// ── Entry point ────────────────────────────────────────────────────────

pub fn run() -> io::Result<()> {
    let mut terminal = ratatui::init();
    let result = run_app(&mut terminal);
    ratatui::restore();
    result
}

fn run_app(terminal: &mut ratatui::DefaultTerminal) -> io::Result<()> {
    let mut app = App::new();

    // Start rate fetcher
    let (rate_tx, rate_rx) = mpsc::channel();
    spawn_rate_fetcher(rate_tx);

    // Set initial viewport height
    if let Ok(size) = terminal.size() {
        app.editor.viewport_height = size.height.saturating_sub(2) as usize;
    }

    loop {
        terminal.draw(|frame| ui(frame, &app))?;

        // Check for rate updates (non-blocking)
        if let Ok(rates) = rate_rx.try_recv() {
            app.set_rates(rates);
        }

        if event::poll(Duration::from_millis(50))? {
            match event::read()? {
                Event::Key(key) if key.kind == KeyEventKind::Press => {
                    app.handle_key(key);
                    if app.should_quit {
                        break;
                    }
                }
                Event::Resize(_, h) => {
                    app.editor.viewport_height = h.saturating_sub(2) as usize;
                }
                _ => {}
            }
        }
    }

    app.save_all();
    Ok(())
}
