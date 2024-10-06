use std::str::Chars;

use crate::action_menu::MenuAction;
use crate::appdata::WindowFocus;
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
    if app.window_focus == WindowFocus::ActionMenu {
        render_action_popup(app, frame);
    } else if app.window_focus == WindowFocus::ActionMenuStep2 {
        render_action_popup_step2(app, frame);
    }
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

    let max_title_w = area.width as usize - 2;
    let mut title_text = app.get_current_string_path();
    if title_text.len() > max_title_w {
        let split_pos = title_text
            .char_indices()
            .nth_back(max_title_w - 1 - 1)
            .unwrap()
            .0;
        title_text = format!("…{}", &title_text[split_pos..]);
    }

    let title_block = Block::default()
        .title(title_text)
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

fn render_action_popup(app: &mut App, frame: &mut Frame) {
    let list_items: Vec<ListItem> = app
        .known_menu_actions
        .iter()
        .map(|it: &MenuAction| ListItem::new(it.name))
        .collect();
    let mut list_state = ListState::default().with_selected(Some(app.action_menu_cursor_y));
    let widget = List::new(list_items)
        .block(
            Block::default()
                .title("Run action")
                .borders(Borders::ALL)
                .bg(Color::DarkGray),
        )
        .style(Style::default().fg(Color::White).bg(Color::DarkGray))
        .highlight_style(Style::new().add_modifier(Modifier::REVERSED))
        .highlight_symbol(">> ");

    let height = app.known_menu_actions.len() as u16 + 2;
    let width: u16 = app
        .known_menu_actions
        .iter()
        .map(|it: &MenuAction| it.name.len() as u16)
        .max()
        .unwrap_or(0)
        + 8;
    let area = centered_rect(width, height, frame.size());
    let buffer = frame.buffer_mut();
    Clear.render(area, buffer);
    frame.render_stateful_widget(widget, area, &mut list_state);
}

fn render_action_popup_step2(app: &mut App, frame: &mut Frame) {
    let display_buffer = format!("{} ", app.action_menu_buffer);
    let chars: Chars<'_> = display_buffer.chars();
    let cx = app.action_menu_cursor_x;
    let buffer_pre: String = chars.clone().take(cx).collect::<String>();
    let highlighted: String = chars.clone().skip(cx).take(1).collect::<String>();
    let buffer_post: String = chars.skip(cx + 1).collect::<String>();
    let p_line = Line::from(vec![
        Span::styled(buffer_pre, Style::default().fg(Color::White)),
        Span::styled(
            highlighted,
            Style::default().fg(Color::Black).bg(Color::White),
        ),
        Span::styled(buffer_post, Style::default().fg(Color::White)),
    ]);

    let title = Block::default()
        .title(app.action_menu_title.as_str())
        .title_style(Style::new().bold())
        .title_alignment(Alignment::Left)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .bg(Color::DarkGray);
    let widget = Paragraph::new(p_line)
        .wrap(Wrap { trim: true })
        .block(title)
        .alignment(Alignment::Left);

    let width = frame.size().width * 3 / 4;
    let height = 4;
    let area = centered_rect(width, height, frame.size());
    let buffer = frame.buffer_mut();
    Clear.render(area, buffer);
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

    let error_window = Paragraph::new(error_message)
        .wrap(Wrap { trim: true })
        .block(title)
        .style(Style::default().fg(Color::White));

    let ok_label = Paragraph::new("OK")
        .style(Style::default().bold().fg(Color::LightRed).bg(Color::White))
        .alignment(Alignment::Center);

    let width: u16 = (frame.size().width as f32 * 0.75f32) as u16;
    let height: u16 = frame.size().height / 2;
    let area = centered_rect(width, height, frame.size());

    let ok_label_area = Rect {
        x: area.x + 1,
        y: area.y + area.height - 2,
        width: area.width - 2,
        height: 1,
    };

    let buffer = frame.buffer_mut();
    Clear.render(area, buffer);
    frame.render_widget(error_window, area);
    frame.render_widget(ok_label, ok_label_area);
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
