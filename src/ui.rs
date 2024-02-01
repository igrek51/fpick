use ratatui::{prelude::*, widgets::*};
use ratatui::{
    prelude::{Alignment, Frame},
    style::{Color, Style},
    widgets::{Block, BorderType, Borders, Paragraph},
};

use crate::app::App;

const VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn render(app: &mut App, frame: &mut Frame) {
    let area = frame.size();

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![
            Constraint::Max(3),
            Constraint::Min(10),
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
        .style(Style::default().fg(Color::LightRed))
        .alignment(Alignment::Center);

    frame.render_widget(widget, area);
}

fn render_dir_tree(app: &mut App, frame: &mut Frame, area: Rect) {
    // let list_items: Vec<ListItem> = vec![];
    // let mut list_state = ListState::default().with_selected(Some(list_items));
    // frame.render_stateful_widget(widget, area, &mut list_state);
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
