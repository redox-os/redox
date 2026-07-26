// TUI utilities, shared between repo and repo_editor

use ansi_to_tui::IntoText;
use std::{borrow::Cow, cmp, process};

use ratatui::{
    layout::{Position, Rect},
    style::{Color, Style},
    text::{Line, Text},
};

use crate::cook::script::KILL_ALL_PID;

/// Drain `buffer` to `lines` when new lines occurred. Does `"\r"` trimming.
pub fn drain_buffer_to_lines(buffer: &mut Vec<u8>, lines: &mut Vec<String>) -> usize {
    let mut addition = 0;
    // TODO: multibyte-aware line split?
    while let Some(newline_pos) = buffer.iter().position(|&b| b == b'\n') {
        let line_bytes = buffer.drain(..=newline_pos);
        let line_str = String::from_utf8_lossy(&line_bytes.as_slice());
        let line_str_pos = line_str.trim_end();
        let line_str = line_str_pos.rsplit('\r').next().unwrap_or(&line_str_pos);
        lines.push(line_str.to_owned());
        addition += 1;
    }
    addition
}

/// Render build log lines, stateful
pub fn render_build_log<'a>(
    panel_height: usize,
    log_text: Option<&'a Vec<String>>,
    log_line: Option<Cow<'a, str>>,
    may_autoscroll: bool,
    enable_auto_scroll: &'a mut bool,
    intended_scroll_pos: &'a mut usize,
    search_result: Option<(usize, &'a Vec<usize>)>,
) -> Vec<Line<'a>> {
    let mut log_lines: Vec<Line> = if let Some(log_text) = log_text
        && !log_text.is_empty()
    {
        let total_log_lines = log_text.len() as usize;

        let start = if *enable_auto_scroll {
            if total_log_lines > panel_height {
                *intended_scroll_pos = total_log_lines - panel_height;
                total_log_lines - panel_height
            } else {
                0
            }
        } else {
            if total_log_lines > panel_height {
                let limit = 2; // arbitrary number
                if *intended_scroll_pos >= total_log_lines - limit {
                    if may_autoscroll {
                        *enable_auto_scroll = true;
                    }
                    *intended_scroll_pos = total_log_lines - limit;
                    total_log_lines - limit
                } else {
                    *intended_scroll_pos
                }
            } else {
                0
            }
        };

        let end = cmp::min(panel_height + start, total_log_lines - 1);

        log_text[start..end]
            .iter()
            .enumerate()
            .map(|(i, s)| {
                if let Some((search_idx, search_results)) = search_result {
                    let absolute_i = start + i;
                    if absolute_i == search_results[search_idx] {
                        let s = strip_ansi_escapes::strip_str(s);
                        return Line::from(s)
                            .style(Style::default().bg(Color::Yellow).fg(Color::Black));
                    } else if search_results.binary_search(&absolute_i).is_ok() {
                        let s = strip_ansi_escapes::strip_str(s);
                        return Line::from(s)
                            .style(Style::default().bg(Color::Black).fg(Color::White));
                    }
                }
                let text_with_colors = s
                    .into_text()
                    .unwrap_or_else(|_| Text::raw("--unrenderable line--"));
                text_with_colors
                    .lines
                    .into_iter()
                    .next()
                    .unwrap_or_else(|| Line::raw("--unrenderable line--"))
            })
            .collect()
    } else {
        vec![]
    };

    if let Some(buffer) = log_line
        && !buffer.is_empty()
    {
        let text_with_colors = handle_cr(&buffer)
            .into_text()
            .unwrap_or_else(|_| Text::raw("--unrenderable line--"));

        if let Some(line) = text_with_colors.lines.into_iter().next() {
            log_lines.push(line);
        }
    }

    if log_lines.is_empty() {
        log_lines.push(Line::from("No logs yet"));
    }
    log_lines
}

/// Join logs for logging
pub fn join_logs(log: &Vec<String>, line: Option<Cow<'_, str>>) -> String {
    let mut logs = log.join("\n");
    if let Some(line) = line {
        logs.push_str("\n");
        logs.push_str(handle_cr(&line));
    }
    logs
}

fn handle_cr<'a>(buffer: &'a Cow<'_, str>) -> &'a str {
    let st = buffer.trim_end();
    st.rsplit('\r').next().unwrap_or(&st)
}

/// Check if point is in rect
pub fn get_clicked_tab_index(
    area: Rect,
    mp: Position,
    titles: &Vec<String>,
    divider_len: u16,
) -> Option<usize> {
    if !area.contains(mp) {
        return None;
    }

    let mut current_x = area.x;
    let mx = mp.x;

    for (idx, title) in titles.iter().enumerate() {
        let title_width = title.chars().count() as u16;
        let tab_end_x = current_x + title_width;

        if mx >= current_x && mx < tab_end_x {
            return Some(idx);
        }

        current_x = tab_end_x + divider_len;

        if current_x >= area.x + area.width {
            break;
        }
    }

    None
}

/// Kill everything under given PID or this process PID.
pub fn kill_everything(pid: Option<u32>) {
    process::Command::new("bash")
        .arg("-c")
        .arg(KILL_ALL_PID.replace("$PID", &pid.unwrap_or_else(|| process::id()).to_string()))
        .stdout(process::Stdio::null())
        .stderr(process::Stdio::null())
        .spawn()
        .expect("unable to spawn kill");
}
