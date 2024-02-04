use crate::numbers::ClampNumExt;
use ratatui::{prelude::*, widgets::*};
use ratatui::{
    prelude::{Alignment, Frame},
    style::{Color, Style},
    widgets::{Block, BorderType, Borders, Paragraph},
};

use crate::app::App;
use crate::tree::TreeNode;

pub fn render(app: &mut App, frame: &mut Frame) {
    let area = frame.size();
    let middle_h = area.height - 3;

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![Constraint::Min(middle_h), Constraint::Max(3)])
        .split(area);

    render_dir_tree(app, frame, layout[0]);
    render_filter_panel(app, frame, layout[1]);
    if app.error_message.is_some() {
        render_error_popup(app, frame);
    }
}

fn render_dir_tree(app: &mut App, frame: &mut Frame, area: Rect) {
    let list_items: Vec<ListItem> = app
        .child_tree_nodes
        .iter()
        .map(|it: &TreeNode| it.render_list_item())
        .collect();

    let title_block = Block::default()
        .title(app.get_current_string_path())
        .title_style(Style::new().bold())
        .title_alignment(Alignment::Left)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded);
    let widget = List::new(list_items)
        .block(title_block)
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::new().add_modifier(Modifier::REVERSED))
        .highlight_symbol(">> ");

    frame.render_stateful_widget(widget, area, &mut app.file_tree_state);
}

fn render_filter_panel(app: &mut App, frame: &mut Frame, area: Rect) {
    let p_text = format!("{}\u{2588}", app.filter_text);
    let title = Block::default()
        .title("Search")
        .title_style(Style::new().bold())
        .title_alignment(Alignment::Left)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded);

    let widget = Paragraph::new(p_text)
        .block(title)
        .style(Style::default().fg(Color::LightYellow))
        .alignment(Alignment::Left);

    frame.render_widget(widget, area);
}

fn render_error_popup(app: &mut App, frame: &mut Frame) {
    if app.error_message.is_none() {
        return;
    }
    let error_message: String = app.error_message.clone().unwrap();

    let title = Block::default()
        .title("Error")
        .title_style(Style::new().bold())
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .bg(Color::Red)
        .border_type(BorderType::Rounded);

    let widget = Paragraph::new(error_message)
        .wrap(Wrap { trim: true })
        .block(title)
        .style(Style::default().fg(Color::White));

    let width: u16 = (frame.size().width as f32 * 0.75f32) as u16;
    let height: u16 = frame.size().height / 2;
    let area = centered_rect(width, height, frame.size());
    let buffer = frame.buffer_mut();
    Clear.render(area, buffer);
    frame.render_widget(widget, area);
}

fn centered_rect(w: u16, h: u16, r: Rect) -> Rect {
    let x_gap = (r.width as i32 - w as i32).clamp_min(0) / 2;
    let y_gap = (r.height as i32 - h as i32).clamp_min(0) / 2;
    Rect {
        x: r.x + x_gap as u16,
        y: r.y + y_gap as u16,
        width: w,
        height: h,
    }
}
