// Contains code related to TUI layout

use cookbook::{
    Error, Result,
    cook::tui::{get_clicked_tab_index, kill_everything, render_build_log},
    recipe::CookRecipe,
    staged_pkg,
};
use pkg::PackageName;
use ratatui::{
    Terminal,
    backend::TermionBackend,
    layout::{Constraint, Direction, Layout, Position, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Tabs, Wrap},
};
use std::collections::BTreeSet;
use std::time::{Duration, Instant};
use std::{
    io::{self},
    thread,
};
use std::{ops::ControlFlow, path::Path};
use termion::{
    event::{Event, Key, MouseButton, MouseEvent},
    input::{MouseTerminal, TermRead},
    raw::IntoRawMode,
    screen::IntoAlternateScreen,
};

use std::sync::mpsc;

pub mod app;
pub use app::*;

pub mod jobs;
pub use jobs::*;

fn main() -> Result<()> {
    let mut app = App::new();
    let (status_tx, status_rx) = mpsc::channel::<StatusUpdate>();
    let stdout = io::stdout().into_raw_mode().unwrap();
    let stdout = MouseTerminal::from(stdout);
    let stdout = stdout.into_alternate_screen().unwrap();
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend).unwrap();

    let stdin = io::stdin();
    let events = stdin.events();

    let (input_tx, input_rx) = mpsc::channel::<Event>();
    let _input_handle = thread::spawn(move || {
        for evt in events {
            if let Ok(evt) = evt {
                if input_tx.send(evt).is_err() {
                    return;
                }
            }
        }
    });

    loop {
        let r = terminal.draw(|f| {
            let main_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Length(30), Constraint::Min(0)])
                .split(f.area());
            render_left_recipes(&mut app, f, &main_chunks);

            let right_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(3), Constraint::Min(0)])
                .split(main_chunks[1]);
            render_right_tabs(&mut app, f, &right_chunks);

            match app.right_tab_active {
                RightPanelTab::MainActions => {
                    let right_style =
                        app.box_style(" Main Actions ", "[Ctrl+C] Exit", Focus::MainPanel);
                    let right_inner = right_style.inner(right_chunks[1]);
                    f.render_widget(right_style, right_chunks[1]);
                    let right_chunks_2 = Layout::default()
                        .direction(Direction::Horizontal)
                        .constraints([Constraint::Length(25), Constraint::Min(0)])
                        .split(right_inner);
                    render_right_actions(&mut app, f, right_chunks_2[0]);
                }
                RightPanelTab::Terminal(_) => {
                    render_right_terminal(f, &mut app, right_chunks[1]);
                }
            }
        });

        r.map_err(|e| Error::from_io_error(e, "Drawing to terminal pty"))?;

        while let Ok(event) = input_rx.try_recv() {
            if let ControlFlow::Break(_) = handle_input(&mut app, &mut terminal, event, &status_tx)
            {
                kill_everything(None);
                return Ok(());
            }
        }

        while let Ok(update) = status_rx.try_recv() {
            app.exec.handle_status_update(update);
        }

        std::thread::sleep(Duration::from_millis(16));
    }
}

fn render_left_recipes(
    app: &mut App,
    f: &mut ratatui::prelude::Frame<'_>,
    main_chunks: &std::rc::Rc<[Rect]>,
) {
    let left_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(2),
            Constraint::Min(0),
        ])
        .split(main_chunks[0]);

    let search_title = if app.pinned_recipes.len() == 0 {
        format!(" Search ")
    } else {
        format!(" Search [Selected {}] ", app.pinned_recipes.len())
    };
    let search_style = app.box_style(&search_title, "", Focus::SearchAndList);
    let search_box = Paragraph::new(app.search_query.as_str()).block(search_style);
    f.render_widget(search_box, left_chunks[0]);

    app.filter_tabs_area = left_chunks[1];
    {
        let split = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(1), Constraint::Min(0)])
            .split(left_chunks[1]);
        let labels = FilterSource::labels();
        let tabs = Tabs::new(vec![labels[0], labels[1]])
            .select(app.filter_source.index())
            .style(Style::default().fg(Color::Gray))
            .highlight_style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            );
        f.render_widget(tabs, split[0]);
        let tabs = Tabs::new(vec![labels[2], labels[3]])
            .select(app.filter_source.index() - 2)
            .style(Style::default().fg(Color::Gray))
            .highlight_style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            );
        f.render_widget(tabs, split[1]);
    }

    let list_style = app.box_style(" Recipes ", "[Ctrl+A] Select All", Focus::SearchAndList);

    let items: Vec<ListItem> = app
        .filtered_recipes
        .iter()
        .map(|name| {
            let is_pinned = app.pinned_recipes.contains(name);
            let prefix = if is_pinned { "* " } else { "  " };
            let style = if is_pinned {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            ListItem::new(Line::from(vec![
                Span::styled(prefix, Style::default().fg(Color::Green)),
                Span::styled(name.as_str(), style),
            ]))
        })
        .collect();

    let list = List::new(items)
        .block(list_style)
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">");
    f.render_stateful_widget(list, left_chunks[2], &mut app.list_state);
}

fn render_right_tabs(
    app: &mut App,
    f: &mut ratatui::prelude::Frame<'_>,
    right_chunks: &std::rc::Rc<[Rect]>,
) {
    let mut right_tab_titles = vec![" [F6] Build ".to_string()];
    let mut fbtn = 6;
    for job in &app.exec.active_job_order {
        let Some(job) = app.exec.jobs.get(&job) else {
            continue;
        };
        let fstr = if fbtn > 12 {
            "".into()
        } else {
            fbtn += 1;
            format!("[F{}] ", fbtn)
        };
        right_tab_titles.push(format!("{}{}", fstr, job));
    }
    let selected_tab_idx = match &app.right_tab_active {
        RightPanelTab::MainActions => 0,
        RightPanelTab::Terminal(i) => *i + 1,
    };
    app.right_tabs_area = right_chunks[0];
    let top_right_tabs = Tabs::new(right_tab_titles.clone())
        .select(selected_tab_idx)
        .style(Style::default().fg(Color::Gray))
        .block(app.box_style("", "", Focus::MainPanel))
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        );
    app.right_tabs_str = right_tab_titles
        .into_iter()
        .map(|s| s.to_string())
        .collect();
    f.render_widget(top_right_tabs, right_chunks[0]);
}

fn render_right_actions(app: &mut App, f: &mut ratatui::prelude::Frame<'_>, right_layout: Rect) {
    let active_targets = app.selected_recipes();
    if !active_targets.is_empty() {
        let btn_count = app.buttons.len();
        let info_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Length((btn_count * 3) as _),
            ])
            .split(right_layout);

        let target_str = if active_targets.len() == 1 {
            active_targets[0].to_string()
        } else {
            format!("{} recipes", active_targets.len())
        };

        let info_text = vec![Line::from(vec![
            Span::raw("Selected: "),
            Span::styled(
                target_str,
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
        ])];
        f.render_widget(Paragraph::new(info_text), info_chunks[0]);

        let btn_constraints = vec![Constraint::Percentage(100 / btn_count as u16); btn_count];
        let btn_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(btn_constraints)
            .split(info_chunks[1]);

        for (idx, btn) in app.buttons.iter_mut().enumerate() {
            btn.area = btn_chunks[idx];
            let btn_widget = Paragraph::new(btn.label)
                .alignment(ratatui::layout::Alignment::Center)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(Color::Gray)),
                );
            f.render_widget(btn_widget, btn.area);
        }
    } else {
        let no_select = Paragraph::new("No recipe selected");
        f.render_widget(no_select, right_layout);
    }
}

fn render_right_terminal(f: &mut ratatui::Frame, app: &mut App, area: Rect) {
    let Some(id) = app.current_job_id() else {
        return;
    };
    let is_focus = app.focus == Focus::MainPanel;
    let Some(job) = &mut app.exec.jobs.get_mut(&id) else {
        return;
    };

    let title = format!(" Job #{}: repo {}", job.id, job);
    let hint = if job.exit_code.is_some() {
        "[X] Close [Ctrl+C] Exit"
    } else {
        "[Ctrl+C] Stop"
    };

    let auto_scroll = job.auto_scroll;
    let lines = render_build_log(
        area.height as _,
        Some(&job.logs),
        Some(String::from_utf8_lossy(&job.buffer)),
        true,
        &mut job.auto_scroll,
        &mut job.scroll,
        None,
    );

    let line_block = {
        // TODO: Copied from app.box_style (borrowing issue)
        let style = if is_focus {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default()
        };
        Block::default()
            .borders(Borders::ALL)
            .title(title)
            .title_bottom(hint)
            .border_style(style)
    };
    let mut paragraph = Paragraph::new(lines).block(line_block);

    if !auto_scroll {
        paragraph = paragraph.wrap(Wrap { trim: false });
    }

    f.render_widget(paragraph, area);
}

fn handle_input<B: ratatui::backend::Backend>(
    app: &mut App,
    terminal: &mut Terminal<B>,
    event: Event,
    status_tx: &mpsc::Sender<StatusUpdate>,
) -> ControlFlow<()> {
    match event {
        Event::Key(Key::Ctrl('c')) => {
            if let Some(job) = app.current_job()
                && job.pid.is_some()
            {
                kill_everything(job.pid);
            } else {
                return ControlFlow::Break(());
            }
        }

        Event::Key(Key::F(n)) => {
            if let Some(fi) = FilterSource::try_from_index((n - 2) as _) {
                if app.filter_source != fi {
                    app.filter_source = fi;
                    app.update_filter();
                }
                app.focus = Focus::SearchAndList;
            } else if n == 6 {
                app.right_tab_active = RightPanelTab::MainActions;
                app.focus = Focus::MainPanel;
            } else if n != 0 {
                app.right_tab_active = RightPanelTab::Terminal((n - 7) as _);
                app.focus = Focus::MainPanel;
            }
        }

        Event::Key(Key::Ctrl('a')) => {
            app.pin_all_filtered();
        }

        Event::Key(Key::Ctrl('r')) => {
            app.read_recipes();
        }

        Event::Key(Key::Char('x')) => {
            if let Some(id) = app.current_job_id() {
                app.exec.close_job(&id);
                app.right_tab_active = RightPanelTab::MainActions;
            }
        }

        Event::Key(key) => match app.focus {
            Focus::SearchAndList => match key {
                Key::Char('\t') => {
                    app.toggle_pinned_highlighted();
                }
                Key::Char('\n') => {
                    app.focus = Focus::MainPanel;
                }
                Key::Char(c) => {
                    app.search_query.push(c);
                    app.update_filter();
                }
                Key::Backspace => {
                    app.search_query.pop();
                    app.update_filter();
                }
                Key::Up => app.select_delta(-1, true),
                Key::Down => app.select_delta(1, true),
                _ => {}
            },
            Focus::MainPanel => match key {
                Key::Char('\n') | Key::Esc => {
                    app.focus = Focus::SearchAndList;
                }
                Key::Char('\t') => {
                    app.toggle_pinned_highlighted();
                }
                Key::Down => app.select_delta(1, true),
                Key::Char(c) => {
                    let matched_action = app.buttons.iter().find(|b| b.key == c).map(|b| b.cmd);
                    let targets = app.selected_recipes();
                    if let (Some(cmd), false, true) = (
                        matched_action,
                        targets.is_empty(),
                        app.current_job_id().is_none(),
                    ) {
                        app.exec.spawn_job(cmd, targets, status_tx.clone());
                        app.right_tab_active =
                            RightPanelTab::Terminal(app.exec.active_job_order.len() - 1);
                    }
                }
                key => {
                    if let Some(job) = app.current_job_mut() {
                        match key {
                            Key::Up => {
                                job.auto_scroll = false;
                                job.scroll = job.scroll.saturating_add_signed(-1);
                            }
                            Key::Down => {
                                job.auto_scroll = false;
                                job.scroll = job.scroll.saturating_add_signed(1);
                            }
                            Key::PageUp => {
                                job.auto_scroll = false;
                                job.scroll = job.scroll.saturating_add_signed(-20);
                            }
                            Key::PageDown => {
                                job.auto_scroll = false;
                                job.scroll = job.scroll.saturating_add_signed(20);
                            }
                            Key::Home => {
                                job.auto_scroll = false;
                                job.scroll = 0;
                            }
                            Key::End => {
                                job.auto_scroll = true;
                                job.scroll = usize::MAX;
                            }
                            _ => {}
                        }
                    }
                }
            },
        },

        Event::Mouse(mouse_event) => {
            let (x, y) = match mouse_event {
                MouseEvent::Press(_, x, y) => (x, y),
                MouseEvent::Hold(x, y) => (x, y),
                _ => (0, 0),
            };
            let mx = x.saturating_sub(1);
            let my = y.saturating_sub(1);
            let mp = Position::new(mx, my);

            let size = terminal.size().unwrap();
            let viewport = Rect::new(0, 0, size.width, size.height);

            let main_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Length(30), Constraint::Min(0)])
                .split(viewport);

            let left_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Length(2),
                    Constraint::Min(0),
                ])
                .split(main_chunks[0]);

            match mouse_event {
                MouseEvent::Press(
                    btn @ MouseButton::WheelUp | btn @ MouseButton::WheelDown,
                    _,
                    _,
                ) => {
                    let delta = if btn == MouseButton::WheelUp { -1 } else { 1 };
                    if main_chunks[0].contains(mp) {
                        app.focus = Focus::SearchAndList;
                        app.select_delta(delta, false);
                    } else {
                        app.focus = Focus::MainPanel;
                        if let Some(i) = app.current_job_mut() {
                            i.scroll = i.scroll.saturating_add_signed(delta);
                            i.auto_scroll = false;
                        }
                    }
                }
                MouseEvent::Press(MouseButton::Left, _, _) | MouseEvent::Hold(_, _) => {
                    if app.filter_tabs_area.contains(mp) {
                        let tabw = app.filter_tabs_area.width / 2;
                        let tabh = app.filter_tabs_area.height / 2;
                        let relative_x = mx.saturating_sub(app.filter_tabs_area.x);
                        let relative_y = my.saturating_sub(app.filter_tabs_area.y);
                        let index = (relative_x / tabw) + (relative_y / tabh * 2);
                        app.filter_source =
                            FilterSource::try_from_index(index as _).unwrap_or(FilterSource::All);
                        app.update_filter();
                    } else if app.right_tabs_area.contains(mp) {
                        app.focus = Focus::MainPanel;

                        if let Some(i) =
                            get_clicked_tab_index(app.right_tabs_area, mp, &app.right_tabs_str, 3)
                        {
                            if i == 0 {
                                app.right_tab_active = RightPanelTab::MainActions;
                            } else {
                                app.right_tab_active = RightPanelTab::Terminal(i - 1);
                            }
                        }
                    } else if left_chunks[2].contains(mp) {
                        app.focus = Focus::SearchAndList;

                        let list_top = left_chunks[2].y + 1;
                        if my >= list_top && my < left_chunks[2].y + left_chunks[2].height - 1 {
                            let clicked_visible_row = (my - list_top) as usize;
                            let offset = app.list_state.offset();
                            let target_idx = offset + clicked_visible_row;

                            if target_idx < app.filtered_recipes.len() {
                                app.list_state.select(Some(target_idx));
                                app.right_tab_active = RightPanelTab::MainActions;

                                let now = Instant::now();
                                if let (Some(last_time), Some(last_idx)) =
                                    (app.last_click_time, app.last_click_idx)
                                {
                                    if last_idx == target_idx
                                        && now.duration_since(last_time)
                                            < Duration::from_millis(300)
                                    {
                                        app.toggle_pinned_highlighted();
                                    }
                                }
                                app.last_click_time = Some(now);
                                app.last_click_idx = Some(target_idx);
                            }
                        }
                    } else if main_chunks[1].contains(mp) {
                        app.focus = Focus::MainPanel;

                        if app.current_job_id().is_none() {}
                        let mut clicked_command = None;
                        for btn in &app.buttons {
                            if btn.area.contains(mp) {
                                clicked_command = Some(btn.cmd);
                                break;
                            }
                        }
                        let targets = app.selected_recipes();
                        if let Some(cmd) = clicked_command
                            && !targets.is_empty()
                            && app.current_job_id().is_none()
                        {
                            app.exec.spawn_job(cmd, targets, status_tx.clone());
                            app.right_tab_active =
                                RightPanelTab::Terminal(app.exec.active_job_order.len() - 1);
                        }
                    }
                }
                _ => {}
            };
        }
        _ => {}
    }

    ControlFlow::Continue(())
}
