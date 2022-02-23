// use crossterm::{
//     event::{self, DisableMouseCapture, EnableMouseCapture, Event},
//     execute,
//     terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
// };
use tui::{
    layout::Alignment,
    style::{Style},
    text::{Span, Spans},
    widgets::{Block, BorderType, Borders, List, ListItem},
    // Frame, Terminal,
};

// use sodiumoxide::crypto::box_::{PublicKey, SecretKey};
// use sp_keyring::AccountKeyring;

use crate::material::color::{GREY_50, GREY_800, BLUE_200};
use crate::pane::{Pane, MenuAction};

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

pub fn list_menu(pane: Pane) -> List<'static> {
    let list: Vec<ListItem> = pane
        .menu
        .iter()
        // .filter(|item| item.group.eq(&pane.group))
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
