// Contains TUI specific code

use crate::*;
use cookbook::cook::fetch::FetchResult;
use cookbook::cook::pty::{UnixSlavePty, flush_pty, setup_pty, write_to_pty};
use cookbook::cook::tui::{drain_buffer_to_lines, join_logs, kill_everything, render_build_log};
use cookbook::recipe::CookRecipe;
use cookbook::{Error, Result, staged_pkg};
use pkg::PackageName;
use ratatui::Terminal;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::prelude::TermionBackend;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap};
use std::borrow::Cow;
use std::collections::HashMap;
use std::io::{Read, Write, stdin, stdout};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Arc, mpsc};
use std::thread;
use std::time::{Duration, Instant};
use termion::event::{Event, Key};
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::screen::IntoAlternateScreen;

#[derive(Debug, Clone, PartialEq)]
pub enum RecipeStatus {
    Pending,
    Fetching,
    Fetched,
    Cooking,
    Cached,
    Done,
    Failed(String),
}

impl RecipeStatus {
    pub fn fetch_is_part_of(&self) -> bool {
        matches!(*self, RecipeStatus::Pending | RecipeStatus::Fetching)
    }
    pub fn fetch_style(&self) -> Style {
        match *self {
            RecipeStatus::Fetching => Style::default().fg(Color::Yellow),
            _ => Style::default(),
        }
    }
    pub fn fetch_icon(&self, spin: char) -> char {
        match *self {
            RecipeStatus::Pending => ' ',
            RecipeStatus::Fetching => spin,
            _ => '?',
        }
    }
    pub fn cook_is_part_of(&self) -> bool {
        matches!(
            *self,
            RecipeStatus::Fetched
                | RecipeStatus::Cooking
                | RecipeStatus::Done
                | RecipeStatus::Cached
                | RecipeStatus::Failed(_)
        )
    }
    pub fn cook_style(&self) -> Style {
        match *self {
            RecipeStatus::Fetched => Style::default(),
            RecipeStatus::Cooking => Style::default().fg(Color::Yellow),
            RecipeStatus::Done => Style::default().fg(Color::Green),
            RecipeStatus::Cached => Style::default().fg(Color::Cyan),
            RecipeStatus::Failed(_) => Style::default().fg(Color::Red),
            _ => Style::default(),
        }
    }
    pub fn cook_icon(&self, spin: char) -> char {
        match *self {
            RecipeStatus::Fetched => ' ',
            RecipeStatus::Cooking => spin,
            RecipeStatus::Done => '+',
            RecipeStatus::Cached => ' ',
            RecipeStatus::Failed(_) => 'X',
            _ => '?',
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum StatusUpdate {
    StartFetch(PackageName),
    Fetched(CookRecipe),
    FailFetch(CookRecipe, String),
    StartCook(PackageName),
    Cooked(CookRecipe, bool),
    FailCook(CookRecipe, String),
    PushLog(PackageName, Vec<u8>),
    FlushLog(PackageName, PathBuf),
    FetchThreadFinished,
    CookThreadFinished,
}

#[derive(PartialEq)]
pub enum JobType {
    Fetch,
    Cook,
}

impl ToString for JobType {
    fn to_string(&self) -> String {
        match self {
            JobType::Fetch => "Fetch",
            JobType::Cook => "Cook",
        }
        .to_string()
    }
}

const PROMPT_WAIT: Duration = Duration::from_millis(101);

pub struct TuiApp {
    pub recipes: Vec<(CookRecipe, RecipeStatus)>,
    active_fetch: Option<PackageName>,
    active_cook: Option<PackageName>,
    logs: HashMap<PackageName, Vec<String>>,
    log_byte_buffer: HashMap<PackageName, Vec<u8>>,
    log_scroll: usize,
    log_view_job: JobType,
    auto_scroll: bool,
    cook_scroll: usize,
    cook_list_state: ListState,
    fetch_complete: bool,
    cook_complete: bool,
    prompt: Option<FailurePrompt>,
    pub dump_logs_anyway: bool,
    pub dump_logs_on_exit: Option<(PackageName, String)>,
    is_inspecting: bool,
    search_query: String,
    search_results: Option<Vec<usize>>,
    search_idx: usize,
}

impl TuiApp {
    pub fn new(recipes: Vec<CookRecipe>) -> Self {
        Self {
            recipes: recipes
                .iter()
                .cloned()
                .map(|r| (r, RecipeStatus::Pending))
                .collect(),
            active_fetch: None,
            active_cook: None,
            logs: HashMap::new(),
            log_byte_buffer: HashMap::new(),
            log_scroll: 0,
            auto_scroll: true,
            log_view_job: JobType::Fetch,
            cook_scroll: 0,
            cook_list_state: ListState::default(),
            fetch_complete: false,
            cook_complete: false,
            prompt: None,
            dump_logs_anyway: false,
            dump_logs_on_exit: None,
            is_inspecting: false,
            search_query: String::new(),
            search_results: None,
            search_idx: 0,
        }
    }

    pub fn get_active_name(&self) -> Option<PackageName> {
        if self.log_view_job == JobType::Cook {
            self.active_cook.clone()
        } else {
            self.active_fetch.clone()
        }
    }

    pub fn get_active_log(
        &self,
    ) -> (
        Option<PackageName>,
        Option<&Vec<String>>,
        Option<Cow<'_, str>>,
    ) {
        let active_name = self.get_active_name();
        let (log_text, log_line) = if let Some(active_name) = active_name.as_ref() {
            self.get_recipe_log(active_name)
        } else {
            (None, None)
        };

        (active_name, log_text, log_line)
    }

    pub fn get_recipe_log(
        &self,
        recipe_name: &PackageName,
    ) -> (Option<&Vec<String>>, Option<Cow<'_, str>>) {
        let log_text = self.logs.get(recipe_name);
        let log_line = if let Some(b) = self.log_byte_buffer.get(recipe_name) {
            Some(String::from_utf8_lossy(b))
        } else {
            None
        };
        (log_text, log_line)
    }

    pub fn write_log(&self, recipe_name: &PackageName, log_path: &PathBuf) -> Result<()> {
        let (Some(logs), line) = self.get_recipe_log(recipe_name) else {
            return Ok(());
        };
        let str = strip_ansi_escapes::strip_str(join_logs(logs, line));
        if !str.trim_end().is_empty() {
            std::fs::write(log_path, str).map_err(|e| Error::from_io_error(e, "Writing log"))?;
        }
        return Ok(());
    }

    // Update the state based on a message from a worker thread
    pub fn update_status(&mut self, update: StatusUpdate) {
        let (name, new_status) = match update {
            StatusUpdate::StartFetch(name) => {
                self.active_fetch = Some(name.clone());
                self.logs.insert(name.clone(), Vec::new());
                self.log_byte_buffer.insert(name.clone(), Vec::new());
                self.log_scroll = 0;
                self.auto_scroll = true;
                (name.clone(), RecipeStatus::Fetching)
            }
            StatusUpdate::Fetched(recipe) => (recipe.name.clone(), RecipeStatus::Fetched),
            StatusUpdate::FailFetch(recipe, err) => {
                self.prompt = Some(FailurePrompt::new(recipe.clone(), err.clone()));
                (recipe.name.clone(), RecipeStatus::Failed(err))
            }
            StatusUpdate::StartCook(name) => {
                self.active_cook = Some(name.clone());
                self.logs.insert(name.clone(), Vec::new());
                self.log_byte_buffer.insert(name.clone(), Vec::new());
                (name.clone(), RecipeStatus::Cooking)
            }
            StatusUpdate::PushLog(name, chunk) => {
                let buffer = self.log_byte_buffer.entry(name.clone()).or_default();
                buffer.extend_from_slice(&chunk);
                if self.dump_logs_anyway {
                    let _ = std::io::stdout().write_all(&chunk);
                }
                let log_list = self.logs.entry(name.clone()).or_default();
                drain_buffer_to_lines(buffer, log_list);
                return;
            }
            StatusUpdate::FlushLog(name, path) => {
                // TODO: This blocks the TUI, maybe open separate thread?
                // FIXME: handle error here?
                let _ = self.write_log(&name, &path);
                return;
            }
            StatusUpdate::Cooked(recipe, cached) => {
                if self.active_cook.as_ref() == Some(&recipe.name) {
                    self.active_cook = None;
                }
                self.auto_scroll = true;
                (
                    recipe.name.clone(),
                    if cached {
                        RecipeStatus::Cached
                    } else {
                        RecipeStatus::Done
                    },
                )
            }
            StatusUpdate::FailCook(recipe, err) => {
                self.prompt = Some(FailurePrompt::new(recipe.clone(), err.clone()));
                (recipe.name.clone(), RecipeStatus::Failed(err))
            }
            StatusUpdate::FetchThreadFinished => {
                self.fetch_complete = true;
                self.log_view_job = JobType::Cook;
                return;
            }
            StatusUpdate::CookThreadFinished => {
                self.cook_complete = true;
                return;
            }
        };

        if let Some((_, status)) = self.recipes.iter_mut().find(|(r, _)| r.name == name) {
            *status = new_status;
        }
    }
}

pub fn run_tui_cook(config: CliConfig, recipes: Vec<CookRecipe>) -> Result<TuiApp> {
    let (work_tx, work_rx) = mpsc::channel::<(CookRecipe, FetchResult)>();
    let (status_tx, status_rx) = mpsc::channel::<StatusUpdate>();

    let running = Arc::new(AtomicBool::new(true));
    let prompting = Arc::new(AtomicU32::new(0));
    const TICK_RATE: Duration = Duration::from_millis(100);

    // ---- Cooker Thread ----
    let cooker_config = config.clone();
    let cooker_status_tx = status_tx.clone();
    let cooker_prompting = prompting.clone();
    let cooker_handle = thread::spawn(move || {
        'done: for (mut recipe, fetch_result) in work_rx {
            let name = recipe.name.clone();
            let (mut stdout_writer, mut stderr_writer) = setup_logger(&cooker_status_tx, &name);
            let mut logger = Some((&mut stdout_writer, &mut stderr_writer));
            'again: loop {
                cooker_status_tx
                    .send(StatusUpdate::StartCook(name.clone()))
                    .unwrap();
                let _ = recipe.reload_recipe(); // reread recipe.toml in case we're retrying
                let handler = handle_cook(
                    &recipe,
                    &cooker_config,
                    fetch_result.source_dir.clone(),
                    &logger,
                );
                if let Some(log_path) = cooker_config.logs_dir.as_ref()
                    // prefer to retain full build logs
                    && !matches!(handler, Ok(true))
                {
                    if let Err(err_ctx) = &handler {
                        write_to_pty(&logger, &format!("\n{err_ctx}"));
                    }
                    flush_pty(&mut logger);
                    let log_path = log_path.join(format!("{}/{}.log", recipe.target, name.name()));
                    cooker_status_tx
                        .send(StatusUpdate::FlushLog(name.clone(), log_path))
                        .unwrap_or_default();
                }
                match handler {
                    Ok(cached) => {
                        cooker_status_tx
                            .send(StatusUpdate::Cooked(recipe, cached))
                            .unwrap_or_default();
                        if cooker_config.cook.nonstop
                            && cooker_prompting.load(Ordering::SeqCst) == 4
                        {
                            break 'done;
                        }
                        break;
                    }
                    Err(e) => {
                        cooker_status_tx
                            .send(StatusUpdate::FailCook(recipe.clone(), e.to_string()))
                            .unwrap_or_default();
                        if cooker_config.cook.nonstop {
                            if cooker_prompting.load(Ordering::SeqCst) == 4 {
                                break 'done;
                            }
                            // TODO: where to report error?
                            let _ = handle_nonstop_fail(&recipe);
                            break;
                        }
                        while cooker_prompting.load(Ordering::SeqCst) != 0 {
                            thread::sleep(PROMPT_WAIT); // wait other prompt
                        }
                        cooker_prompting.swap(1, Ordering::SeqCst);
                        'wait: loop {
                            match cooker_prompting.load(Ordering::SeqCst) {
                                0 => break 'again,
                                1 => thread::sleep(PROMPT_WAIT),
                                2 => {
                                    cooker_prompting.swap(0, Ordering::SeqCst);
                                    break 'wait;
                                } // retry
                                3 => {
                                    cooker_prompting.swap(0, Ordering::SeqCst);
                                    let _ = handle_nonstop_fail(&recipe);
                                    break 'again;
                                } // skip
                                4 => {
                                    cooker_prompting.swap(0, Ordering::SeqCst);
                                    break 'done;
                                } // done
                                _ => unreachable!(),
                            }
                        }
                    }
                }
            }
        }
        cooker_status_tx
            .send(StatusUpdate::CookThreadFinished)
            .unwrap_or_default();
    });

    let mstdin = stdin();
    let mstdout = stdout()
        .into_raw_mode()
        .unwrap()
        .into_alternate_screen()
        .unwrap();

    // ----- Input Thread -----
    let (input_tx, input_rx) = mpsc::channel::<Event>();
    let _input_handle = thread::spawn(move || {
        for evt in mstdin.events() {
            if let Ok(evt) = evt {
                if input_tx.send(evt).is_err() {
                    return;
                }
            }
        }
    });

    // ---- Fetcher Thread ----
    let fetcher_recipes = recipes.clone();
    let fetcher_status_tx = status_tx.clone();
    let fetcher_config = config.clone();
    let fetcher_prompting = prompting.clone();
    let fetcher_handle = thread::spawn(move || {
        'done: for mut recipe in fetcher_recipes {
            let name = recipe.name.clone();
            let (mut stdout_writer, mut stderr_writer) = setup_logger(&fetcher_status_tx, &name);
            let mut logger = Some((&mut stdout_writer, &mut stderr_writer));
            'again: loop {
                fetcher_status_tx
                    .send(StatusUpdate::StartFetch(name.clone()))
                    .unwrap();
                let _ = recipe.reload_recipe(); // reread recipe.toml in case we're retrying
                let handler = handle_fetch(&recipe, &fetcher_config, true, &logger);
                if let Some(log_path) = fetcher_config.logs_dir.as_ref()
                    // prefer to retain full build logs
                    && !matches!(handler, Ok(FetchResult { cached: true, .. }))
                {
                    if let Err(err_ctx) = &handler {
                        write_to_pty(&logger, &format!("\n{err_ctx}"));
                    }
                    flush_pty(&mut logger);
                    let log_path = log_path.join(format!("{}/{}.log", recipe.target, name.name()));
                    fetcher_status_tx
                        .send(StatusUpdate::FlushLog(name.clone(), log_path))
                        .unwrap_or_default();
                }
                match handler {
                    Ok(fetch) => {
                        fetcher_status_tx
                            .send(StatusUpdate::Fetched(recipe.clone()))
                            .unwrap();
                        if work_tx.send((recipe.clone(), fetch)).is_err() {
                            // Cooker thread died
                            break 'done;
                        }
                        if fetcher_config.cook.nonstop
                            && fetcher_prompting.load(Ordering::SeqCst) == 4
                        {
                            break 'done;
                        }
                        break;
                    }
                    Err(e) => {
                        fetcher_status_tx
                            .send(StatusUpdate::FailFetch(recipe.clone(), e.to_string()))
                            .unwrap_or_default();
                        if fetcher_config.cook.nonstop {
                            if fetcher_prompting.load(Ordering::SeqCst) == 4 {
                                break 'done;
                            }
                            let _ = handle_nonstop_fail(&recipe);
                            break;
                        }
                        while fetcher_prompting.load(Ordering::SeqCst) != 0 {
                            thread::sleep(PROMPT_WAIT); // wait other prompt
                        }
                        fetcher_prompting.swap(1, Ordering::SeqCst);
                        'wait: loop {
                            match fetcher_prompting.load(Ordering::SeqCst) {
                                0 => break 'again,
                                1 => thread::sleep(PROMPT_WAIT),
                                2 => {
                                    fetcher_prompting.swap(0, Ordering::SeqCst);
                                    break 'wait;
                                } // retry
                                3 => {
                                    fetcher_prompting.swap(0, Ordering::SeqCst);
                                    let _ = handle_nonstop_fail(&recipe);
                                    break 'again;
                                } // skip
                                4 => {
                                    fetcher_prompting.swap(0, Ordering::SeqCst);
                                    break 'done;
                                } // done
                                _ => unreachable!(),
                            }
                        }
                    }
                }
            }
        }
        status_tx
            .send(StatusUpdate::FetchThreadFinished)
            .unwrap_or_default();
    });

    let mut terminal = Terminal::new(TermionBackend::new(stdout()))
        .map_err(|e| Error::from_io_error(e, "Reading terminal pty"))?;
    terminal
        .clear()
        .map_err(|e| Error::from_io_error(e, "Clearing terminal pty"))?;
    let mut app = TuiApp::new(recipes);

    let spinner = ['-', '\\', '|', '/'];
    let mut spinner_i = 0;

    while running.load(Ordering::SeqCst) {
        let frame_start = Instant::now();
        let r = terminal.draw(|f| {
            spinner_i = (spinner_i + 1) % spinner.len();
            let spin = spinner[spinner_i];

            let mut constraints = Vec::new();
            if app.is_inspecting {
                constraints.push(Constraint::Percentage(100));
            } else {
                if !app.fetch_complete {
                    constraints.push(Constraint::Length(22));
                }
                constraints.push(Constraint::Length(22));
                constraints.push(Constraint::Min(20));
            }

            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(constraints)
                .split(f.area());
            let panel_height = chunks[0].height.saturating_sub(2) as usize;

            if !app.is_inspecting {
                // Left Pane
                let fetch_items: Vec<ListItem> = app
                    .recipes
                    .iter()
                    .filter(|(_, s)| s.fetch_is_part_of())
                    .map(|(r, s)| {
                        let icon = s.fetch_icon(spin);
                        ListItem::new(format!("{icon} {}", r.name)).style(s.fetch_style())
                    })
                    .collect();
                let fetch_list = List::new(fetch_items).block(
                    Block::default()
                        .title("Fetch Queue [1]")
                        .borders(Borders::ALL),
                );
                f.render_widget(fetch_list, chunks[0]);

                // Right Pane
                let cook_items: Vec<ListItem> = app
                    .recipes
                    .iter()
                    .filter(|(_, s)| s.cook_is_part_of())
                    .map(|(r, s)| {
                        let icon = s.cook_icon(spin);
                        ListItem::new(format!("{icon} {}", r.name)).style(s.cook_style())
                    })
                    .collect();
                {
                    let cooking_index = app
                        .recipes
                        .iter()
                        .filter(|(_, s)| s.cook_is_part_of())
                        .position(|(_r, s)| *s == RecipeStatus::Cooking);

                    if let Some(index) = cooking_index {
                        app.cook_list_state.select(Some(index));
                        let index_u16 = index;
                        let center_offset = panel_height / 2;
                        let new_offset = index_u16.saturating_sub(center_offset) as usize;

                        *app.cook_list_state.offset_mut() = new_offset;
                    }
                }
                let cook_items: Vec<ListItem> = cook_items[app.cook_scroll..].into();
                let cook_chunk = chunks[if app.fetch_complete { 0 } else { 1 }];
                let cook_list = List::new(cook_items).block(
                    Block::default()
                        .title("Cook Queue [2]")
                        .borders(Borders::ALL),
                );
                f.render_stateful_widget(cook_list, cook_chunk, &mut app.cook_list_state);
            }

            let log_area = if app.is_inspecting {
                chunks[0]
            } else {
                chunks[if app.fetch_complete { 1 } else { 2 }]
            };

            let (active_name, log_text, log_line) = app.get_active_log();
            let log_title = if let Some(active_name) = active_name {
                format!(
                    " {} Log: {} ",
                    app.log_view_job.to_string(),
                    if app.is_inspecting {
                        staged_pkg::find(active_name.as_str())
                            .map(|s| s.to_str())
                            .flatten()
                            .unwrap_or(active_name.as_str())
                    } else {
                        active_name.as_str()
                    }
                )
            } else {
                format!(" {} Log ", app.log_view_job.to_string())
            };

            let mut enable_auto_scroll = app.auto_scroll;
            let mut intended_scroll_pos = app.log_scroll;

            let log_lines = render_build_log(
                panel_height,
                log_text,
                log_line,
                app.prompt.is_none() || config.cook.nonstop,
                &mut enable_auto_scroll,
                &mut intended_scroll_pos,
                if let Some(search_results) = app.search_results.as_ref()
                    && app.is_inspecting
                {
                    Some((app.search_idx, search_results))
                } else {
                    None
                },
            );

            let instruct = if app.is_inspecting {
                let line_info = if let Some(search_results) = app.search_results.as_ref() {
                    format!(
                        "[line {}; {} of {}]",
                        search_results[app.search_idx],
                        app.search_idx + 1,
                        search_results.len()
                    )
                } else {
                    format!("[line {}]", app.log_scroll + 1)
                };

                format!(
                    " Search: {:?} {} {} ",
                    app.search_query,
                    line_info,
                    if app.search_results.is_some() {
                        "[Down/Up] Next/previous search [Shift+Down/Up] Scroll [Esc] Exit search"
                    } else {
                        "[Enter] Begin search [Esc] Exit inspect"
                    }
                )
            } else {
                format!(
                    " Keys: [c] Stop [PageUp/Down] Scroll{}{} ",
                    match app.auto_scroll {
                        true => "",
                        false => " [End] Follow log trails",
                    },
                    match (&app.log_view_job, app.fetch_complete) {
                        (JobType::Fetch, _) => " [2] View Cook Log",
                        (JobType::Cook, false) => " [1] View Fetch Log",
                        (JobType::Cook, true) => "",
                    }
                )
            };

            let log_borders = if app.is_inspecting {
                Borders::TOP | Borders::BOTTOM
            } else {
                Borders::ALL
            };

            let mut log_paragraph = Paragraph::new(log_lines).block(
                Block::default()
                    .title(log_title)
                    .title_bottom(instruct)
                    .borders(log_borders),
            );

            if !app.auto_scroll {
                log_paragraph = log_paragraph.wrap(Wrap { trim: false });
            }

            f.render_widget(log_paragraph, log_area);
            if let Some(prompt) = &mut app.prompt {
                if config.cook.nonstop && prompt.selected == PromptOption::Retry {
                    prompt.selected = PromptOption::Skip;
                }
                if !app.is_inspecting {
                    draw_prompt(f, prompt, config.cook.nonstop);
                }
            }
            if !app.auto_scroll && enable_auto_scroll {
                app.auto_scroll = true;
            }
            if intended_scroll_pos != app.log_scroll {
                app.log_scroll = intended_scroll_pos;
            }

            while let Ok(event) = input_rx.try_recv() {
                if app.is_inspecting {
                    if handle_inspect_event(&event, &mut app) {
                        app.is_inspecting = false;
                    }
                    continue;
                }
                if let Some((app, res)) = handle_prompt_input(&event, &mut app) {
                    prompting.swap(res as u32, Ordering::SeqCst);
                    if res == PromptOption::Exit {
                        // TODO: This can be a different log with what prompted on nonstop mode
                        let (name, log, line) = app.get_active_log();
                        if let Some(name) = name
                            && let Some(log) = log
                        {
                            app.dump_logs_on_exit = Some((name.to_owned(), join_logs(log, line)));
                        }
                        running.store(false, Ordering::SeqCst);
                    }
                    app.prompt = None;
                } else {
                    handle_main_event(&mut app, &event);
                }
            }
        });

        r.map_err(|e| Error::from_io_error(e, "Drawing to terminal pty"))?;

        while let Ok(update) = status_rx.try_recv() {
            app.update_status(update);
        }

        if app.cook_complete {
            running.swap(false, Ordering::SeqCst);
        }

        if let Some(sleep_duration) = TICK_RATE.checked_sub(frame_start.elapsed()) {
            thread::sleep(sleep_duration);
        }
    }

    drop(mstdout);
    let _ = stdout().flush();

    if config.cook.nonstop && app.dump_logs_on_exit.is_some() {
        kill_everything(None);
    }

    let _ = fetcher_handle.join();
    let _ = cooker_handle.join();

    Ok(app)
}

fn handle_main_event(app: &mut TuiApp, event: &Event) {
    match event {
        Event::Key(key) => match key {
            Key::Char('1') => {
                app.log_view_job = JobType::Fetch;
            }
            Key::Char('2') => {
                app.log_view_job = JobType::Cook;
            }
            Key::Char('c') => {
                // as compilers still running, we use this way to stop it
                kill_everything(None);
            }
            Key::Up => {
                app.auto_scroll = false;
                app.log_scroll = app.log_scroll.saturating_sub(1);
            }
            Key::Down => {
                app.auto_scroll = false;
                app.log_scroll = app.log_scroll.saturating_add(1);
            }
            Key::PageUp => {
                app.auto_scroll = false;
                app.log_scroll = app.log_scroll.saturating_sub(20);
            }
            Key::PageDown => {
                app.auto_scroll = false;
                app.log_scroll = app.log_scroll.saturating_add(20);
            }
            Key::End => {
                app.auto_scroll = true;
            }
            Key::Home => {
                app.auto_scroll = false;
                app.log_scroll = 0;
            }
            _ => {}
        },
        _ => {}
    }
}

fn perform_search(app: &mut TuiApp) {
    app.search_idx = 0;
    if app.search_query.is_empty() {
        return;
    }

    let (_, log_text, _) = app.get_active_log();
    let mut search_results = Vec::new();
    let mut first_index = None;
    if let Some(logs) = log_text {
        for (i, line) in logs.iter().enumerate() {
            let stripped = strip_ansi_escapes::strip_str(line);
            if stripped
                .to_lowercase()
                .contains(&app.search_query.to_lowercase())
            {
                search_results.push(i);
                if first_index.is_none() && i >= app.log_scroll {
                    first_index = Some((i, search_results.len() - 1));
                }
            }
        }
    }
    first_index = match first_index {
        Some(i) => Some(i),
        None => search_results.first().cloned().map(|s| (s, 0)),
    };
    app.search_results = Some(search_results);
    app.auto_scroll = false;
    if let Some((first_index, search_i)) = first_index {
        app.log_scroll = first_index.saturating_sub(10);
        app.search_idx = search_i;
    }
}

fn handle_inspect_event(event: &Event, app: &mut TuiApp) -> bool {
    if let Event::Key(key) = event {
        match key {
            Key::Esc => {
                if app.search_results.is_some() {
                    app.search_results.take();
                } else {
                    app.search_query.clear();
                    return true;
                }
            }
            Key::Char('\n') => {
                if let Some(search_results) = app.search_results.as_mut() {
                    // same as keydown
                    app.search_idx = if app.search_idx + 1 < search_results.len() {
                        app.search_idx + 1
                    } else {
                        0
                    };
                    app.log_scroll = search_results[app.search_idx].saturating_sub(10);
                } else {
                    perform_search(app);
                }
            }
            Key::Backspace if app.search_results.is_some() => {
                app.search_query.pop();
                if app.search_query.len() == 0 {
                    app.search_results.take();
                } else {
                    perform_search(app);
                }
            }
            Key::Char(c) if app.search_results.is_some() => {
                app.search_query.push(*c);
                perform_search(app);
            }
            Key::Backspace => {
                app.search_query.pop();
            }
            Key::Char(c) => {
                app.search_query.push(*c);
            }
            Key::Up => {
                if let Some(search_results) = app.search_results.as_mut() {
                    app.search_idx = if app.search_idx > 0 {
                        app.search_idx - 1
                    } else {
                        search_results.len() - 1
                    };
                    app.log_scroll = search_results[app.search_idx].saturating_sub(10);
                } else {
                    app.auto_scroll = false;
                    app.log_scroll = app.log_scroll.saturating_sub(1);
                }
            }
            Key::ShiftUp => {
                app.auto_scroll = false;
                app.log_scroll = app.log_scroll.saturating_sub(1);
            }
            Key::Down => {
                if let Some(search_results) = app.search_results.as_mut() {
                    app.search_idx = if app.search_idx + 1 < search_results.len() {
                        app.search_idx + 1
                    } else {
                        0
                    };
                    app.log_scroll = search_results[app.search_idx].saturating_sub(10);
                } else {
                    app.auto_scroll = false;
                    app.log_scroll = app.log_scroll.saturating_add(1);
                }
            }
            Key::ShiftDown => {
                app.auto_scroll = false;
                app.log_scroll = app.log_scroll.saturating_add(1);
            }
            Key::PageUp => {
                app.auto_scroll = false;
                app.log_scroll = app.log_scroll.saturating_sub(20);
            }
            Key::PageDown => {
                app.auto_scroll = false;
                app.log_scroll = app.log_scroll.saturating_add(20);
            }
            _ => {}
        }
    }
    false
}

fn handle_prompt_input<'a>(
    event: &Event,
    app: &'a mut TuiApp,
) -> Option<(&'a mut TuiApp, PromptOption)> {
    if let Some(prompt) = &mut app.prompt {
        match event {
            Event::Key(key) => match key {
                Key::Char('q') | Key::Ctrl('c') | Key::Esc => {
                    // Treat as "Exit"
                    return Some((app, PromptOption::Exit));
                }
                Key::Left | Key::BackTab => prompt.prev(),
                Key::Right | Key::Char('\t') => prompt.next(),
                Key::Char('\n') if prompt.selected == PromptOption::Inspect => {
                    app.is_inspecting = true;
                }
                Key::Char('\n') => {
                    let prompt = app.prompt.take().unwrap();
                    return Some((app, prompt.selected));
                }
                _ => {}
            },
            _ => {} // Ignore mouse events
        }
    }
    None
}

fn draw_prompt(f: &mut ratatui::Frame, prompt: &FailurePrompt, is_nonstop: bool) {
    let title = format!(
        " FAILURE in {} {}",
        prompt.recipe.name,
        if is_nonstop { "(skipped) " } else { "" }
    );
    let mut error_text = prompt.error.clone();
    if error_text.len() > 300 {
        error_text =
            error_text[0..150].to_string() + ".." + &error_text[(error_text.len() - 150)..];
    } else if error_text.len() > 150 {
        error_text = error_text[0..150].to_string() + "..";
    }

    let get_style = |opt: PromptOption| {
        if prompt.selected == opt {
            Style::default().bg(Color::White).fg(Color::Black)
        } else {
            Style::default()
        }
    };

    let buttons = if is_nonstop {
        vec![
            Span::styled(" [Skip] ", get_style(PromptOption::Skip)),
            Span::raw("   "),
            Span::styled(" [Exit] ", get_style(PromptOption::Exit)),
        ]
    } else {
        vec![
            Span::styled(" [Skip] ", get_style(PromptOption::Skip)),
            Span::raw(" "),
            Span::styled(" [Inspect] ", get_style(PromptOption::Inspect)),
            Span::raw(" "),
            Span::styled(" [Exit] ", get_style(PromptOption::Exit)),
            Span::raw(" "),
            Span::styled(" [Retry] ", get_style(PromptOption::Retry)),
        ]
    };

    let text = vec![
        Line::from(error_text).style(Style::default().fg(Color::Yellow)),
        Line::from(""),
        Line::from(buttons),
    ];

    let block = Block::default()
        .title(Span::styled(
            title,
            Style::default().fg(Color::White).bg(Color::Red),
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Red));

    let paragraph = Paragraph::new(text)
        .block(block)
        .alignment(ratatui::layout::Alignment::Center)
        .wrap(Wrap { trim: true });

    let area = f.area();
    let popup_area = Rect {
        x: area.width.saturating_sub(100) / 2, // Centered better for wider prompts
        y: area.height / 3,
        width: 100.min(area.width),
        height: 10,
    };

    f.render_widget(Clear, popup_area);
    f.render_widget(paragraph, popup_area);
}

fn spawn_log_reader<R>(
    mut reader: R,
    package_name: PackageName,
    status_tx: mpsc::Sender<StatusUpdate>,
) where
    R: Read + Send + 'static,
{
    thread::spawn(move || {
        let mut buffer = [0; 1024];
        loop {
            let buf = match reader.read(&mut buffer) {
                Ok(0) => break,
                Ok(n) => buffer[..n].to_vec(),
                Err(e) => format!("[IO Error] {}", e).into_bytes(),
            };
            if status_tx
                .send(StatusUpdate::PushLog(package_name.clone(), buf))
                .is_err()
            {
                // TUI thread hung up
                break;
            }
        }
    });
}

pub fn setup_logger(
    status_tx: &mpsc::Sender<StatusUpdate>,
    name: &PackageName,
) -> (UnixSlavePty, std::io::PipeWriter) {
    let (pty_reader, log_reader, pipes) = setup_pty();

    spawn_log_reader(pty_reader, name.clone(), status_tx.clone());
    spawn_log_reader(log_reader, name.clone(), status_tx.clone());
    pipes
}

#[derive(PartialEq, Clone, Copy)]
#[repr(u32)]
enum PromptOption {
    Retry = 2,
    Skip,
    Exit,
    Inspect,
}

struct FailurePrompt {
    recipe: CookRecipe,
    error: String,
    selected: PromptOption,
}

impl FailurePrompt {
    fn new(recipe: CookRecipe, error: String) -> Self {
        Self {
            recipe,
            error,
            selected: PromptOption::Exit,
        }
    }

    fn next(&mut self) {
        self.selected = match self.selected {
            PromptOption::Retry => PromptOption::Skip,
            PromptOption::Skip => PromptOption::Inspect,
            PromptOption::Inspect => PromptOption::Exit,
            PromptOption::Exit => PromptOption::Retry,
        }
    }

    fn prev(&mut self) {
        self.selected = match self.selected {
            PromptOption::Retry => PromptOption::Exit,
            PromptOption::Skip => PromptOption::Retry,
            PromptOption::Inspect => PromptOption::Skip,
            PromptOption::Exit => PromptOption::Inspect,
        }
    }
}
