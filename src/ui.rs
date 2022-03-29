use tui::{
    layout::{Constraint, Direction, Layout, Rect, Alignment},
    style::{Style},
    text::{Span, Spans},
    widgets::{Block, BorderType, Borders, List, ListItem},
};

use crate::material::color::{GREY_50, GREY_800, BLUE_200};
use crate::menu::{MenuAction, MenuItem};

pub fn container(title: String, active: bool) -> Block<'static> {
    Block::default()
        .borders(Borders::ALL)
        .title(format!(" {} ", title))
        .title_alignment(Alignment::Center)
        .border_type(BorderType::Rounded)
        .border_style(match active {
            true => Style::default().fg(GREY_50),
            false => Style::default().fg(GREY_800),
        })
}

pub fn list_menu(pane_menu: Vec<MenuItem>) -> List<'static> {
    let list: Vec<ListItem> = pane_menu
        .iter()
        .map(|i| {
            let lines = vec![
                Spans::from(
                    Span::styled(
                        i.clone().title,
                        match i.clone().action {
                            MenuAction::GenerateAccount(_) => Style::default().fg(BLUE_200),
                            MenuAction::ImportAccount(_) => Style::default().fg(BLUE_200),
                            MenuAction::ComposeMessage(_) => Style::default().fg(BLUE_200),
                            MenuAction::ShowAccountInfo(_) => Style::default().fg(BLUE_200),
                            MenuAction::ShowInbox(_) => Style::default().fg(BLUE_200),
                            MenuAction::ShowSent(_) => Style::default().fg(BLUE_200),
                            MenuAction::AddToWhiteList(_) => Style::default().fg(BLUE_200),
                            MenuAction::AddToBlackList(_) => Style::default().fg(BLUE_200),
                            _ => Style::default(),
                        }
                    )
                ),
            ];
            ListItem::new(lines).style(Style::default())
        })
        .collect();

    List::new(list)
        .highlight_style(Style::default().fg(GREY_800).bg(BLUE_200))
}

pub fn account_edit_rect(r: Rect) -> Rect {
    let percent_x = 80;
    let percent_y = 20;

    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Length(5),
                Constraint::Min(1),
            ]
                .as_ref(),
        )
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ].as_ref(),
        )
        .split(popup_layout[1])[1]
}

pub fn account_info_rect(r: Rect) -> Rect {
    let percent_x = 80;
    let percent_y = 20;

    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Length(9),
                Constraint::Min(1),
            ]
                .as_ref(),
        )
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ].as_ref(),
        )
        .split(popup_layout[1])[1]
}

pub fn compose_message_rect(r: Rect) -> Rect {
    let percent_x = 80;
    let percent_y = 40;

    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ].as_ref(),
        )
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ].as_ref(),
        )
        .split(popup_layout[1])[1]
}

// pub fn error_rect(r: Rect) -> Rect {
//     let percent_x = 60;
//     let percent_y = 10;
//
//     let popup_layout = Layout::default()
//         .direction(Direction::Vertical)
//         .constraints(
//             [
//                 Constraint::Percentage((100 - percent_y) / 2),
//                 Constraint::Percentage(percent_y),
//                 Constraint::Percentage((100 - percent_y) / 2),
//             ].as_ref(),
//         )
//         .split(r);
//
//     Layout::default()
//         .direction(Direction::Horizontal)
//         .constraints(
//             [
//                 Constraint::Percentage((100 - percent_x) / 2),
//                 Constraint::Percentage(percent_x),
//                 Constraint::Percentage((100 - percent_x) / 2),
//             ].as_ref(),
//         )
//         .split(popup_layout[1])[1]
// }
