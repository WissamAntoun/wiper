use crate::config::UIConfig;
use crate::fs::Folder;
use crate::fs::SortBy;
use crate::logger::Logger;
use crate::logger::MessageLevel;
use ratatui::{prelude::*, widgets::*};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::ui::constants::{
    NORMAL_ROW_COLOR, TABLE_HEADER_BG, TABLE_HEADER_FG, TABLE_SPACE_WIDTH, TEXT_COLOR,
    TEXT_PRE_DELETED_BG, TEXT_SELECTED_BG,
};
use crate::ui::utils::folder_to_rows;

const MAX_LOG_LEN: usize = 40;
#[derive(Debug)]
pub struct DebugData {
    pub time_taken: Option<u128>,
    pub fps: String,
    pub skipped_frames: String,
    pub folders: usize,
    pub spin_symbol: (char, char),
}

pub fn render_content(
    area: Rect,
    buf: &mut Buffer,
    maybe_folder: Option<&Folder>,
    config: &UIConfig,
    logger: &Logger,
    debug_data: &DebugData,
) {
    let horizontal_layout = Layout::horizontal(match config.debug_enabled {
        true => [Constraint::Min(1), Constraint::Min(1)],
        false => [Constraint::Min(1), Constraint::Max(0)],
    });

    let [content_col, debug_col] = horizontal_layout.areas(area);

    if let Some(folder) = maybe_folder {
        render_table(content_col, buf, folder, config, debug_data.spin_symbol.0);
    }

    if config.debug_enabled {
        render_debug_panel(debug_col, buf, logger, debug_data);
    }
}

pub fn render_table(
    area: Rect,
    buf: &mut Buffer,
    folder: &Folder,
    config: &UIConfig,
    loading_indicator: char,
) {
    let block = Block::default()
        .borders(Borders::ALL)
        .padding(Padding::horizontal(1))
        .border_set(symbols::border::PROPORTIONAL_TALL)
        .fg(TEXT_COLOR)
        .bg(NORMAL_ROW_COLOR);

    let header_style = Style::default().fg(TABLE_HEADER_FG).bg(TABLE_HEADER_BG);
    let selected_style = if config.confirming_deletion {
        Style::default().bg(TEXT_PRE_DELETED_BG)
    } else {
        Style::default().bg(TEXT_SELECTED_BG)
    };

    let header_titles = match config.sort_by {
        SortBy::Title => ["", "Name ↓", "Size", "Space"],
        SortBy::Size => ["", "Name", "Size ↓", "Space"],
    };

    let header = header_titles
        .into_iter()
        .map(Cell::from)
        .collect::<Row>()
        .style(header_style)
        .height(1);

    let rows = folder_to_rows(folder, config, loading_indicator);

    let table = Table::new(
        rows,
        [
            Constraint::Length(1),
            Constraint::Length(40),
            Constraint::Length(20),
            Constraint::Length(TABLE_SPACE_WIDTH as u16),
        ],
    )
    .block(block)
    .header(header)
    .highlight_symbol("> ")
    .highlight_style(selected_style)
    .highlight_spacing(HighlightSpacing::Always);

    StatefulWidget::render(
        table,
        area,
        buf,
        &mut TableState::default().with_selected(Some(folder.cursor_index)),
    );
}

pub fn render_debug_panel(area: Rect, buf: &mut Buffer, logger: &Logger, debug_data: &DebugData) {
    let [top, bottom] = Layout::vertical([Constraint::Max(5), Constraint::Fill(1)]).areas(area);

    // Stats
    let time_taken = debug_data
        .time_taken
        .map(|t| t.to_string())
        .unwrap_or_else(|| "...".to_string());

    let stats_text = Text::from(format!(
        "Folders: {}\nDone in: {}\nFPS: {} | Skipped: {}",
        debug_data.folders, time_taken, debug_data.fps, debug_data.skipped_frames
    ));

    let stats_block = Block::default()
        .padding(Padding::horizontal(1))
        .borders(Borders::ALL)
        .border_set(symbols::border::PROPORTIONAL_TALL)
        .title(" Stats ")
        .title_alignment(Alignment::Center);

    let stats = Paragraph::new(stats_text).left_aligned().block(stats_block);

    Widget::render(stats, top, buf);

    // Logs
    let logs_block = Block::default()
        .padding(Padding::horizontal(1))
        .borders(Borders::ALL)
        .border_set(symbols::border::PROPORTIONAL_TALL)
        .title(" Logs ")
        .title_alignment(Alignment::Center);

    let logs: Vec<ListItem> = logger
        .messages
        .iter()
        .enumerate()
        .map(|(_i, (timestamp, level, message))| {
            let mut message = message.clone();
            let current_timestamp_ms = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards")
                .as_millis();
            let elapsed_ms = current_timestamp_ms - timestamp;
            if message.len() > MAX_LOG_LEN {
                message = format!(
                    "{}..{}",
                    &message[..MAX_LOG_LEN / 4],
                    &message[message.len() - MAX_LOG_LEN / 4 * 3..]
                );
            }
            message = format!("[{:.1}] - {}", elapsed_ms as f64 / 1000.0, message);

            let style = Style::default();
            let style = match level {
                MessageLevel::Info => style.fg(TEXT_COLOR),
                MessageLevel::Error => style.fg(TEXT_PRE_DELETED_BG),
            };
            ListItem::from(message).style(style)
        })
        .collect();

    let items = List::new(logs).block(logs_block);
    Widget::render(items, bottom, buf);
}
