use ratatui::{prelude::*, widgets::*};
use ratatui::{
    prelude::{Alignment, Frame},
    style::{Color, Style},
    widgets::{Block, BorderType, Borders, Paragraph},
};

use crate::app::App;
use crate::filesystem::FileNode;

const VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn render(app: &mut App, frame: &mut Frame) {
    let area = frame.size();
    let middle_h = area.height - 3 - 3;

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![
            Constraint::Max(3),
            Constraint::Min(middle_h),
            Constraint::Max(3),
        ])
        .split(area);

    render_info_panel(app, frame, layout[0]);
    render_dir_tree(app, frame, layout[1]);
    render_filter_panel(app, frame, layout[2]);
}

fn render_info_panel(_app: &mut App, frame: &mut Frame, area: Rect) {
    let p_text = "`/` to filter. `F5` to refresh. `F6` to sort. `Enter` to confirm. `Esc` to exit.";
    let widget = Paragraph::new(p_text)
        .wrap(Wrap { trim: true })
        .block(
            Block::default()
                .title(format!("fpick {}", VERSION))
                .title_alignment(Alignment::Center)
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        )
        .style(Style::default().fg(Color::Gray))
        .alignment(Alignment::Center);

    frame.render_widget(widget, area);
}

fn render_dir_tree(app: &mut App, frame: &mut Frame, area: Rect) {
    let list_items: Vec<ListItem> = app
        .current_child_nodes
        .iter()
        .map(|it: &FileNode| ListItem::new(it.display_name()))
        .collect();

    let title = app.get_current_string_path();

    let widget = List::new(list_items)
        .block(Block::default().title(title).borders(Borders::ALL))
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::new().add_modifier(Modifier::REVERSED))
        .highlight_symbol(">> ");

    frame.render_stateful_widget(widget, area, &mut app.file_tree_state);
}

fn render_filter_panel(app: &mut App, frame: &mut Frame, area: Rect) {
    let p_text = format!("{}\u{2588}", app.filter_text);
    let panel_color = Color::LightYellow;
    let mut title = Block::default().title("Filter");
    title = title.title_style(Style::new().bold());

    let widget = Paragraph::new(p_text)
        .block(
            title
                .title_alignment(Alignment::Center)
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        )
        .style(Style::default().fg(panel_color))
        .alignment(Alignment::Left);

    frame.render_widget(widget, area);
}
