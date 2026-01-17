use crate::app::App;
use crate::domain::{MosaicSpec, TilesSource};
use crate::error::AppResult;
use crate::infra::{ImageIoImpl, TomlCatalogStore};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap};
use ratatui::{prelude::*, Terminal};
use std::io::{self, Stdout};
use std::path::PathBuf;

const ASCII_TITLE: &str = "
      _                _                     __  __               _      
     / \\   _ __   __| |_ __ ___  __ _       |  \\/  | ___  ___  __ _(_) ___ 
    / _ \\ | '_ \\ / _` | '__/ _ \\/ _` |      | |\\/| |/ _ \\/ __/ _` | |/ __|
   / ___ \\| | | | (_| | | |  __/ (_| |      | |  | | (_) \\__ \\ (_| | | (__ 
  /_/   \\_\\_| |_|\\__,_|_|  \\___|\\__,_|      |_|  |_|\\___/|___/\\__,_|_|\\___|
";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MenuItem {
    Generate,
    CatalogAdd,
    CatalogList,
    CatalogRemove,
    Exit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum InputMode {
    None,
    CatalogAdd,
    CatalogRemove,
    GenerateInput,
    GenerateOutput,
    GenerateTiles,
    GenerateTileSize,
}

struct GenerateForm {
    input: Option<PathBuf>,
    output: Option<PathBuf>,
    tiles: Option<String>,
    tile_size: Option<u32>,
}

impl GenerateForm {
    fn new() -> Self {
        Self {
            input: None,
            output: None,
            tiles: None,
            tile_size: None,
        }
    }
}

struct UiState {
    selected: usize,
    input_mode: InputMode,
    input_buffer: String,
    status: Vec<String>,
    generate_form: GenerateForm,
    default_tile_size: u32,
}

pub fn run_tui(app: App<TomlCatalogStore, ImageIoImpl>, default_tile_size: u32) -> AppResult<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut state = UiState {
        selected: 0,
        input_mode: InputMode::None,
        input_buffer: String::new(),
        status: vec!["Use arrows to navigate. Enter to select. Q to quit.".to_string()],
        generate_form: GenerateForm::new(),
        default_tile_size,
    };

    let menu_items = [
        MenuItem::Generate,
        MenuItem::CatalogAdd,
        MenuItem::CatalogList,
        MenuItem::CatalogRemove,
        MenuItem::Exit,
    ];

    let result = (|| -> AppResult<()> {
        loop {
            terminal.draw(|frame| render(frame, &state, &menu_items))?;

            match event::read()? {
                Event::Key(key) => {
                    if handle_key_event(key, &mut state, &menu_items, &app)? {
                        break;
                    }
                }
                _ => {}
            }
        }
        Ok(())
    })();

    let restore = restore_terminal(terminal);
    result.and(restore)
}

fn render(frame: &mut Frame, state: &UiState, menu_items: &[MenuItem]) {
    let root = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(9),
            Constraint::Min(5),
            Constraint::Length(5),
        ])
        .split(frame.size());

    let title = Paragraph::new(Text::from(ASCII_TITLE))
        .block(Block::default().borders(Borders::ALL).title("Andrea Mosaic"))
        .wrap(Wrap { trim: false });
    frame.render_widget(title, root[0]);

    let body = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(35), Constraint::Percentage(65)])
        .split(root[1]);

    let items: Vec<ListItem> = menu_items
        .iter()
        .map(|item| ListItem::new(menu_label(*item)))
        .collect();

    let mut list_state = ListState::default();
    list_state.select(Some(state.selected));

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Menu"))
        .highlight_style(Style::default().add_modifier(Modifier::BOLD))
        .highlight_symbol("-> ");

    frame.render_stateful_widget(list, body[0], &mut list_state);

    let right_block = Block::default().borders(Borders::ALL).title("Output");
    let mut text = Text::default();
    for line in &state.status {
        text.lines.push(Line::from(Span::raw(line.clone())));
    }
    let output = Paragraph::new(text).block(right_block).wrap(Wrap { trim: false });
    frame.render_widget(output, body[1]);

    let footer = render_footer(state);
    frame.render_widget(footer, root[2]);
}

fn render_footer(state: &UiState) -> Paragraph<'static> {
    let prompt = match state.input_mode {
        InputMode::None => "",
        InputMode::CatalogAdd => "Path to images folder or file: ",
        InputMode::CatalogRemove => "Tile id to remove: ",
        InputMode::GenerateInput => "Input image path: ",
        InputMode::GenerateOutput => "Output image path: ",
        InputMode::GenerateTiles => "Tiles source (catalog or /path): ",
        InputMode::GenerateTileSize => "Tile size (blank=default): ",
    };

    let mut lines = Vec::new();
    lines.push(Line::from(Span::raw(prompt.to_string())));
    lines.push(Line::from(Span::raw(state.input_buffer.clone())));

    Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL).title("Input"))
        .wrap(Wrap { trim: false })
}

fn handle_key_event(
    key: KeyEvent,
    state: &mut UiState,
    menu_items: &[MenuItem],
    app: &App<TomlCatalogStore, ImageIoImpl>,
) -> AppResult<bool> {
    if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
        return Ok(true);
    }

    if key.code == KeyCode::Char('q') && state.input_mode == InputMode::None {
        return Ok(true);
    }

    if state.input_mode != InputMode::None {
        return handle_input_mode(key, state, app).map(|_| false);
    }

    match key.code {
        KeyCode::Up => {
            if state.selected > 0 {
                state.selected -= 1;
            }
        }
        KeyCode::Down => {
            if state.selected + 1 < menu_items.len() {
                state.selected += 1;
            }
        }
        KeyCode::Enter => {
            let menu = menu_items[state.selected];
            match menu {
                MenuItem::Generate => {
                    state.generate_form = GenerateForm::new();
                    state.input_buffer.clear();
                    state.input_mode = InputMode::GenerateInput;
                }
                MenuItem::CatalogAdd => {
                    state.input_buffer.clear();
                    state.input_mode = InputMode::CatalogAdd;
                }
                MenuItem::CatalogList => match app.catalog_list() {
                    Ok(catalog) => {
                        state.status.clear();
                        if catalog.tiles.is_empty() {
                            state.status.push("Catalog is empty.".to_string());
                        } else {
                            state.status.push(format!("Catalog tiles: {}", catalog.tiles.len()));
                            for tile in catalog.tiles.iter().take(10) {
                                state.status.push(format!("{}  {}", tile.id, tile.path.display()));
                            }
                            if catalog.tiles.len() > 10 {
                                state.status
                                    .push(format!("... and {} more", catalog.tiles.len() - 10));
                            }
                        }
                    }
                    Err(err) => {
                        state.status = vec![format!("Error: {err}")];
                    }
                },
                MenuItem::CatalogRemove => {
                    state.input_buffer.clear();
                    state.input_mode = InputMode::CatalogRemove;
                }
                MenuItem::Exit => return Ok(true),
            }
        }
        _ => {}
    }

    Ok(false)
}

fn handle_input_mode(
    key: KeyEvent,
    state: &mut UiState,
    app: &App<TomlCatalogStore, ImageIoImpl>,
) -> AppResult<()> {
    match key.code {
        KeyCode::Esc => {
            state.input_mode = InputMode::None;
            state.input_buffer.clear();
            return Ok(());
        }
        KeyCode::Enter => {
            let input = state.input_buffer.trim().to_string();
            state.input_buffer.clear();
            match state.input_mode {
                InputMode::CatalogAdd => {
                    if input.is_empty() {
                        state.status = vec!["Path is required.".to_string()];
                    } else {
                        match app.catalog_add(&PathBuf::from(input)) {
                            Ok(added) => {
                                state.status.clear();
                                if added.is_empty() {
                                    state.status.push("No new tiles added.".to_string());
                                } else {
                                    state.status.push(format!("Added {} tile(s):", added.len()));
                                    for tile in added.iter().take(10) {
                                        state.status
                                            .push(format!("{}  {}", tile.id, tile.path.display()));
                                    }
                                    if added.len() > 10 {
                                        state.status
                                            .push(format!("... and {} more", added.len() - 10));
                                    }
                                }
                            }
                            Err(err) => {
                                state.status = vec![format!("Error: {err}")];
                            }
                        }
                    }
                    state.input_mode = InputMode::None;
                }
                InputMode::CatalogRemove => {
                    if input.is_empty() {
                        state.status = vec!["Tile id is required.".to_string()];
                    } else {
                        match app.catalog_remove(&input) {
                            Ok(removed) => {
                                state.status = vec![format!(
                                    "Removed tile {} ({})",
                                    removed.id,
                                    removed.path.display()
                                )];
                            }
                            Err(err) => {
                                state.status = vec![format!("Error: {err}")];
                            }
                        }
                    }
                    state.input_mode = InputMode::None;
                }
                InputMode::GenerateInput => {
                    if input.is_empty() {
                        state.status = vec!["Input image path is required.".to_string()];
                    } else {
                        state.generate_form.input = Some(PathBuf::from(input));
                        state.input_mode = InputMode::GenerateOutput;
                    }
                }
                InputMode::GenerateOutput => {
                    if input.is_empty() {
                        state.status = vec!["Output image path is required.".to_string()];
                    } else {
                        state.generate_form.output = Some(PathBuf::from(input));
                        state.input_mode = InputMode::GenerateTiles;
                    }
                }
                InputMode::GenerateTiles => {
                    let tiles_value = if input.is_empty() {
                        "catalog".to_string()
                    } else {
                        input
                    };
                    state.generate_form.tiles = Some(tiles_value);
                    state.input_mode = InputMode::GenerateTileSize;
                }
                InputMode::GenerateTileSize => {
                    if !input.is_empty() {
                        match input.parse::<u32>() {
                            Ok(value) => state.generate_form.tile_size = Some(value),
                            Err(_) => {
                                state.status = vec!["Tile size must be a number.".to_string()];
                                return Ok(());
                            }
                        }
                    }

                    match build_spec_from_form(state).and_then(|spec| app.generate_mosaic(&spec)) {
                        Ok(result) => {
                            state.status = vec![
                                format!("Mosaic generated at {}", result.output.display()),
                                format!("Grid: {} x {}", result.grid_width, result.grid_height),
                                format!("Tiles used: {}", result.tiles_used),
                            ];
                        }
                        Err(err) => {
                            state.status = vec![format!("Error: {err}")];
                        }
                    }
                    state.input_mode = InputMode::None;
                }
                InputMode::None => {}
            }
            return Ok(());
        }
        KeyCode::Backspace => {
            state.input_buffer.pop();
        }
        KeyCode::Char(ch) => {
            state.input_buffer.push(ch);
        }
        _ => {}
    }

    Ok(())
}

fn build_spec_from_form(state: &UiState) -> AppResult<MosaicSpec> {
    let input = state
        .generate_form
        .input
        .clone()
        .ok_or_else(|| crate::error::AppError::InvalidInput("input image is required".to_string()))?;

    let output = state
        .generate_form
        .output
        .clone()
        .ok_or_else(|| crate::error::AppError::InvalidInput("output path is required".to_string()))?;

    let tile_size = state.generate_form.tile_size.unwrap_or(state.default_tile_size);

    let tiles_value = state
        .generate_form
        .tiles
        .clone()
        .unwrap_or_else(|| "catalog".to_string());

    let tiles_source = if tiles_value.eq_ignore_ascii_case("catalog") {
        TilesSource::Catalog
    } else {
        TilesSource::Directory(PathBuf::from(tiles_value))
    };

    Ok(MosaicSpec {
        input,
        output,
        tile_size,
        tiles_source,
    })
}

fn menu_label(item: MenuItem) -> &'static str {
    match item {
        MenuItem::Generate => "Generate Mosaic",
        MenuItem::CatalogAdd => "Catalog Add",
        MenuItem::CatalogList => "Catalog List",
        MenuItem::CatalogRemove => "Catalog Remove",
        MenuItem::Exit => "Exit",
    }
}

fn restore_terminal(mut terminal: Terminal<CrosstermBackend<Stdout>>) -> AppResult<()> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}
