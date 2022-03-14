use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyEvent, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use clipboard::{ClipboardProvider, ClipboardContext};
use std::{error::Error, io};
use std::time::{Duration, Instant};

// use log;
// use env_logger;
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Style},
    text::{Span, Spans},
    widgets::{Block, BorderType, Borders, Clear},
    Frame, Terminal,
};
use bs58;

use tui::style::Modifier;
use tui::widgets::{Paragraph};
use unicode_width::UnicodeWidthStr;
use serde::{Serialize, Deserialize};

mod helpers;
mod material;
use crate::material::color::{BLUE_200, GREEN_200, GREY_800, RED_200};

mod config;
use crate::config::constants::MAX_PANES;

mod pane;
use crate::pane::Pane;

mod ui;
mod menu;
use crate::menu::{Menu, MenuAction, MenuGroup, MenuItem};


#[derive(Clone)]
enum ConnectionStatus {
    Success,
    Error,
    Pending,
}

#[derive(Clone)]
struct App {
    pane_index: usize,
    menu: Menu,
    panes: Vec<Pane>,
    connection_status: ConnectionStatus,
    height: Option<u32>,
}

// #[derive(Debug, Deserialize)]
// struct ResponseError {
//     code: i32,
//     message: String,
// }

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SystemSyncState {
    current_block: u32,
    starting_block: u32,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct SystemSyncStateSuccess {
    jsonrpc: String,
    result: SystemSyncState,
    id: u8,
}

#[derive(Serialize)]
struct NodeRequest {
    id: u8,
    jsonrpc: String,
    method: String,
    params: Vec<String>,
}


impl App {
    fn new() -> App {
        App {
            pane_index: 0,
            menu: Menu::new(),
            panes: vec![
                Pane::new(),
                Pane::new()],
            connection_status: ConnectionStatus::Pending,
            height: None,
        }
    }

    async fn on_tick(&mut self) {
        let client = reqwest::Client::new();
        let req = NodeRequest {
            id: 1,
            jsonrpc: "2.0".to_string(),
            method: "system_syncState".to_string(),
            params: vec![],
        };

        let res = client
            .post("http://localhost:9933")
            .json(&serde_json::json!(req))
            .send()
            .await;


        match res {
            Ok(res) => {
                match res.status() {
                    reqwest::StatusCode::OK => {
                        match res.json::<SystemSyncStateSuccess>().await {
                            Ok(parsed) => {
                                self.height = Some(parsed.result.current_block);
                                self.connection_status = ConnectionStatus::Success;
                            },
                            Err(e) => {
                                println!("ERR {:?}", e)
                            },
                        }
                    },
                    _ => {
                        self.connection_status = ConnectionStatus::Error;
                    }
                }
            },
            Err(_e) => {
                self.connection_status = ConnectionStatus::Error;
            }
        }
    }

    fn add_pane(&mut self) {
        match self.panes.len() {
            MAX_PANES => return,
            _ => {},
        };

        self.panes[self.pane_index.clone()].state.select(None);

        let mut panes = self.panes.clone();
        // panes.insert(self.pane_index + 1, Pane::new(&self.menu.items));
        panes.insert(self.pane_index + 1, Pane::new());
        self.panes = panes.clone();
        self.pane_index += 1;
    }

    fn remove_pane(&mut self) {
        let mut panes = self.panes.clone();
        panes.remove(self.pane_index);
        self.panes = panes;
        match self.pane_index {
            0 => self.pane_index = 0,
            _ => self.pane_index -= 1,
        }
    }

    fn next_pane(&mut self) {
        self.panes[self.pane_index.clone()].state.select(None);
        self.pane_index = (self.pane_index + 1) % self.panes.len();
    }

    fn prev_pane(&mut self) {
        self.panes[self.pane_index.clone()].state.select(None);
        match self.pane_index {
            0 => self.pane_index = 0,
            _ => self.pane_index -= 1,
        }
    }
}

#[async_std::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // env_logger::init();
    // info!("Statred!");
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;


    let tick_rate = Duration::from_millis(500);
    let app = App::new();
    let res = run_app(&mut terminal, app, tick_rate).await;

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err);
    }

    Ok(())
}


async fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    mut app: App,
    tick_rate: Duration,
) -> io::Result<()> {
    let mut last_tick = Instant::now();

    loop {
        terminal.draw(|f| ui(f, &mut app))?;
        // let timeout = tick_rate
        //     .checked_sub(last_tick.elapsed())
        //     .unwrap_or_else(|| Duration::from_secs(0));

        // let pane_index = app.pane_index;
        // let panes = &mut app.panes;
        let mut pane = &mut app.panes[app.pane_index];
        // let mut pane = &mut app.pane;
        if let None = pane.state.selected() { pane.next(); }

        if crossterm::event::poll(Duration::from_secs(0))? {
            if let Event::Key(key) = event::read()? {
                match pane.action.clone() {
                    MenuAction::ShowAccountInfo(account) => {
                        match key {
                            KeyEvent{ modifiers: KeyModifiers::NONE, code: KeyCode::Esc} => {
                                pane.account_info = false;
                                pane.action = MenuAction::ShowAccountDown(account);
                            },
                            KeyEvent{ modifiers: KeyModifiers::NONE, code: KeyCode::Char('p')} => {
                                let mut ctx: ClipboardContext = ClipboardProvider::new().unwrap();
                                let public_key = account.pair.0;
                                let address = bs58::encode(public_key).into_string();
                                ctx.set_contents(address).unwrap();
                                pane.account_info = false;
                                pane.action = MenuAction::ShowAccountDown(account);
                            },
                            KeyEvent{ modifiers: KeyModifiers::NONE, code: KeyCode::Char('s')} => {
                                let mut ctx: ClipboardContext = ClipboardProvider::new().unwrap();
                                let public_key = account.clone().pair.1;
                                let address = bs58::encode(public_key).into_string();
                                ctx.set_contents(address).unwrap();
                                pane.account_info = false;
                                pane.action = MenuAction::ShowAccountDown(account);
                            },
                            KeyEvent{ modifiers: KeyModifiers::NONE, code: KeyCode::Backspace }=> {
                                pane.account_name.pop();
                            },

                            _ => {}
                        }
                    },
                    MenuAction::GenerateAccount(identity) => {
                       match key {
                           KeyEvent{ modifiers: KeyModifiers::NONE, code: KeyCode::Esc} => {
                               pane.generate_account = false;
                               pane.account_name = String::from("");
                               pane.action = MenuAction::ShowAccountsDown(identity);
                           },
                           KeyEvent{ modifiers: KeyModifiers::NONE, code: KeyCode::Enter} => {
                               match pane.account_name.len() {
                                   0 => {},
                                   _ => {
                                       let account = app.menu.save_account(
                                           pane.clone().account_name,
                                           identity);
                                       pane.update_full_menu(app.menu.items.clone());
                                       pane.action = MenuAction::ShowAccountDown(account.clone());
                                       pane.group = MenuGroup::Account(account.clone());

                                       let item = MenuItem {
                                           title: pane.account_name.clone(),
                                           action: pane.action.clone(),
                                           group: pane.group.clone(),
                                       };
                                       let mut path = pane.path.clone();
                                       path.push(item.to_owned());
                                       pane.path = path;

                                       pane.generate_account = false;
                                       pane.account_name = String::from("");
                                       pane.state.select(None);
                                       pane.update_pane_menu();
                                   },
                               }
                           },
                           KeyEvent{ modifiers: KeyModifiers::NONE | KeyModifiers::SHIFT, code: KeyCode::Char(c)} => {
                               pane.account_name.push(c);
                           },
                           KeyEvent{ modifiers: KeyModifiers::NONE, code: KeyCode::Backspace }=> {
                               pane.account_name.pop();
                           },
                           _ => {}
                       }
                    },
                    MenuAction::AddToWhiteList(account) => {
                        match key {
                            KeyEvent{ modifiers: KeyModifiers::NONE, code: KeyCode::Esc} => {
                                pane.add_to_whitelist = false;
                                pane.account_address = String::from("");
                                pane.action = MenuAction::ShowWhiteList(account);
                            },
                            KeyEvent{ modifiers: KeyModifiers::NONE, code: KeyCode::Enter} => {
                                match pane.account_address.len() {
                                    0 => {},
                                    _ => {
                                        app.menu.save_to_whitelist(pane.clone().account_address, account);
                                        pane.update_full_menu(app.menu.items.clone());
                                    },
                                }
                            },
                            KeyEvent{ modifiers: KeyModifiers::NONE | KeyModifiers::SHIFT, code: KeyCode::Char(c)} => {
                                pane.account_address.push(c);
                            },
                            KeyEvent{ modifiers: KeyModifiers::NONE, code: KeyCode::Backspace }=> {
                                pane.account_address.pop();
                            },
                            _ => {}
                        }
                    },
                    MenuAction::AddToBlackList(account) => {
                        match key {
                            KeyEvent{ modifiers: KeyModifiers::NONE, code: KeyCode::Esc} => {
                                pane.add_to_whitelist = false;
                                pane.account_address = String::from("");
                                pane.action = MenuAction::ShowBlackList(account);
                            },
                            KeyEvent{ modifiers: KeyModifiers::NONE, code: KeyCode::Enter} => {
                                match pane.account_address.len() {
                                    0 => {},
                                    _ => {
                                        app.menu.save_to_blacklist(pane.clone().account_address, account);
                                        pane.update_full_menu(app.menu.items.clone());
                                    },
                                }
                            },
                            KeyEvent{ modifiers: KeyModifiers::NONE | KeyModifiers::SHIFT, code: KeyCode::Char(c)} => {
                                pane.account_address.push(c);
                            },
                            KeyEvent{ modifiers: KeyModifiers::NONE, code: KeyCode::Backspace }=> {
                                pane.account_address.pop();
                            },
                            _ => {}
                        }
                    },
                    _ => {
                        match key {
                            KeyEvent { modifiers: KeyModifiers::NONE, code: KeyCode::Char('s')} => app.add_pane(),
                            KeyEvent { modifiers: KeyModifiers::NONE, code: KeyCode::Char('w')} => {
                                match app.panes.len() {
                                    1 => {},
                                    _ => app.remove_pane(),
                                }
                            },
                            KeyEvent {
                                modifiers: KeyModifiers::NONE,
                                code: KeyCode::Right | KeyCode::Char('n') | KeyCode::Tab
                            } => app.next_pane(),
                            KeyEvent { modifiers: KeyModifiers::NONE, code: KeyCode::Left | KeyCode::Char('p')} => app.prev_pane(),
                            KeyEvent { modifiers: KeyModifiers::NONE, code: KeyCode::Esc} => {
                                match pane.state.selected() {
                                    Some(_) => pane.state.select(None),
                                    None => {},
                                };
                            },
                            KeyEvent { modifiers: KeyModifiers::NONE, code: KeyCode::Down | KeyCode::Char('j')} => {
                                pane.next();
                                let new_pane = &mut app.panes[app.pane_index];
                                new_pane.update_full_menu(app.menu.items.clone());
                            },
                            KeyEvent { modifiers: KeyModifiers::NONE, code: KeyCode::Up | KeyCode::Char('k')} => {
                                pane.previous();
                                let new_pane = &mut app.panes[app.pane_index];
                                new_pane.update_full_menu(app.menu.items.clone());
                            },
                            KeyEvent { modifiers: KeyModifiers::NONE, code: KeyCode::Enter} => {
                                pane.select();
                            },
                            KeyEvent { modifiers: KeyModifiers::NONE, code: KeyCode::Char('q')} => return Ok(()),
                            _ => {}
                        }
                    }
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            app.on_tick().await;
            last_tick = Instant::now();
        }
    }
}


fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let size = f.size();
    let block = Block::default();
    f.render_widget(block, size);

    let main_area = Layout::default()
        .direction(Direction::Vertical)
        .margin(0)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(1),
            Constraint::Length(3)
        ])
        .split(f.size());



    let status =vec![
        Spans::from(vec![
            Span::styled(
                "Status",
                Style::default(),
            ),
            Span::raw(": "),
            match app.clone().connection_status {
                ConnectionStatus::Success => Span::styled(
                    "connected",
                    Style::default().fg(GREEN_200),
                ),
                ConnectionStatus::Error => Span::styled(
                    "error",
                    Style::default().fg(RED_200),
                ),
                ConnectionStatus::Pending => Span::styled(
                    "loading...",
                    Style::default(),
                )
            },
        ]),
    ];

    let height =vec![
        Spans::from(
            match app.connection_status {
                ConnectionStatus::Success => {
                    vec![
                        Span::styled(
                            "Height",
                            Style::default(),
                        ),
                        Span::raw(": "),
                        Span::styled(
                            match app.height {
                                Some(h) => h.to_string(),
                                None => "Loading...".to_string(),
                            },
                            Style::default(),
                        )
                    ]
                },
                _ => {vec![]}
            }
        ),
    ];

    let stats_layout = Layout::default()
        .direction(Direction::Horizontal)
        .margin(1)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(main_area[0]);
    let stats_block = Block::default()
        .borders(Borders::NONE)
        .title_alignment(Alignment::Center)
        .border_type(BorderType::Rounded)
        .border_style( Style::default().fg(GREY_800));
    let status_paragraph = Paragraph::new(status)
        .style(Style::default())
        .block(stats_block.clone())
        .alignment(Alignment::Left);
    let height_paragraph = Paragraph::new(height)
        .style(Style::default())
        .block(stats_block)
        .alignment(Alignment::Right);
    f.render_widget(status_paragraph, stats_layout[0]);
    f.render_widget(height_paragraph, stats_layout[1]);


    let qty: u16 = app.panes.len() as u16;
    let mut constraints: Vec<Constraint> = vec![];
    for _ in &app.panes { constraints.push(Constraint::Percentage(100/qty)) }
    let panes = Layout::default()
        .direction(Direction::Horizontal)
        .margin(0)
        .constraints(constraints)
        .split(main_area[1]);

    // for (i, p) in app.panes.iter().enumerate() {
    for i in 0..app.panes.len() {
        let p = &mut app.panes[i];

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .margin(2)
            .constraints([Constraint::Percentage(100)])
            .split(panes[i]);

        let pane_menu = Pane::filter_menu(
            app.menu.items.clone(),
            p.group.clone(),
        );
        let menu = ui::list_menu(pane_menu);

        let mut title = String::from("Select Identity");
        for (i, item) in p.path.clone().iter().enumerate() {
            if i == 0 {
                title = format!("/ {}", String::from(item.title.clone()))
            } else {
                title = format!(
                    "{} / {}",
                    title.clone(),
                    String::from(item.title.clone())
                );
            }
        }
        let active = app.pane_index.eq(&i);
        f.render_widget(ui::container(title, active), panes[i]);
        f.render_stateful_widget(menu, layout[0], &mut p.clone().state);

        if p.generate_account {
            let block = Block::default()
                .style(Style::default().bg(GREY_800));
            let area = helpers::account_edit_rect(layout[0]);
            f.render_widget(Clear, area); //this clears out the background
            f.render_widget(block, area);

            let input_area = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints([
                    Constraint::Length(3),
                ].as_ref())
                .split(area);

            let input = Paragraph::new(p.account_name.as_ref())
                .style(Style::default())
                .block(Block::default()
                    .borders(Borders::ALL)
                    .title(" Account Name "),
                );
            f.render_widget(input, input_area[0]);
            f.set_cursor(
                input_area[0].x + p.account_name.width() as u16 + 1,
                input_area[0].y + 1,
            )
        }

        if p.account_info {
            let block = Block::default()
                .title(" Account Info ")
                .title_alignment(Alignment::Center)
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .style(Style::default().bg(GREY_800));
            let area = helpers::account_info_rect(layout[0]);
            f.render_widget(Clear, area); //this clears out the background
            f.render_widget(block, area);

            let info_area = Layout::default()
                .direction(Direction::Vertical)
                .margin(2)
                .constraints([
                    Constraint::Length(1),
                    Constraint::Length(2),
                    Constraint::Length(1),
                    Constraint::Length(2),
                    Constraint::Min(1),
                ].as_ref())
                .split(area);


            if let MenuAction::ShowAccountInfo(account) = p.clone().action {

                let pk_title = Paragraph::new(vec![
                    Spans::from(vec![
                        Span::styled("Public Key (Address)", Style::default().add_modifier(Modifier::BOLD)),
                    ]),
                ]);
                f.render_widget(pk_title, info_area[0]);

                let pk_bs58 = Paragraph::new(vec![
                    Spans::from(vec![
                        Span::raw(bs58::encode(account.pair.0).into_string()),
                    ]),
                ]);
                f.render_widget(pk_bs58, info_area[1]);

                let sk_title = Paragraph::new(vec![
                    Spans::from(vec![
                        Span::styled("Secret Key", Style::default().add_modifier(Modifier::BOLD)),
                    ]),
                ]);
                f.render_widget(sk_title, info_area[2]);

                let pk_bs58 = Paragraph::new(vec![
                    Spans::from(vec![
                        Span::raw(bs58::encode(account.pair.1).into_string()),
                    ]),
                ]);
                f.render_widget(pk_bs58, info_area[3]);
            }
        }

        if p.add_to_whitelist {
            let block = Block::default()
                .style(Style::default().bg(GREY_800));
            let area = helpers::account_edit_rect(layout[0]);
            f.render_widget(Clear, area); //this clears out the background
            f.render_widget(block, area);

            let input_area = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints([
                    Constraint::Length(3),
                ].as_ref())
                .split(area);

            let input = Paragraph::new(p.account_address.as_ref())
                .style(Style::default())
                .block(Block::default()
                   .borders(Borders::ALL)
                   .title(" Account Address "),
                );
            f.render_widget(input, input_area[0]);
            f.set_cursor(
                input_area[0].x + p.account_address.width() as u16 + 1,
                input_area[0].y + 1,
            )
        }

        if p.add_to_blacklist {
            let block = Block::default()
                .style(Style::default().bg(GREY_800));
            let area = helpers::account_edit_rect(layout[0]);
            f.render_widget(Clear, area); //this clears out the background
            f.render_widget(block, area);

            let input_area = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints([
                    Constraint::Length(3),
                ].as_ref())
                .split(area);

            let input = Paragraph::new(p.account_address.as_ref())
                .style(Style::default())
                .block(Block::default()
                   .borders(Borders::ALL)
                   .title(" Account Address "),
                );
            f.render_widget(input, input_area[0]);
            f.set_cursor(
                input_area[0].x + p.account_address.width() as u16 + 1,
                input_area[0].y + 1,
            )
        }
    }

    let commands_normal =vec![
        Spans::from(vec![
            Span::styled(
                "<S>",
                Style::default().fg(GREY_800).bg(BLUE_200)
            ),
            Span::styled(
                "plit Pane",
                Style::default()
            ),
            Span::raw(" "),
            Span::styled(
                "<W>",
                Style::default().fg(GREY_800).bg(BLUE_200)
            ),
            Span::styled(
                "rap Pane",
                Style::default(),
            ),

            Span::raw(" "),
            Span::styled(
                "<N>",
                Style::default().fg(GREY_800).bg(BLUE_200)
            ),
            Span::styled(
                "ext Pane",
                Style::default(),
            ),
            Span::raw(" "),
            Span::styled(
                "<P>",
                Style::default().fg(GREY_800).bg(BLUE_200)
            ),
            Span::styled(
                "rev Pane",
                Style::default(),
            ),
            Span::raw(" "),
            Span::styled(
                "<Q>",
                Style::default().fg(GREY_800).bg(BLUE_200)
            ),
            Span::styled(
                "uit",
                Style::default(),
            ),
        ]),
    ];

    let commands_account_editing = vec![
        Spans::from(vec![
            Span::styled(
                "<Esc>",
                Style::default().fg(GREY_800).bg(BLUE_200)
            ),
            Span::styled(
                " Cancel Editing",
                Style::default(),
            ),
            Span::raw(" "),
            Span::styled(
                "<Enter>",
                Style::default().fg(GREY_800).bg(BLUE_200)
            ),
            Span::styled(
                " Save Account",
                Style::default(),
            ),
        ])
    ];

    let commands_account_info = vec![
        Spans::from(vec![
            Span::styled(
                "<Esc>",
                Style::default().fg(GREY_800).bg(BLUE_200)
            ),
            Span::styled(
                " Close",
                Style::default(),
            ),
            Span::raw(" "),
            Span::styled(
                "<P>",
                Style::default().fg(GREY_800).bg(BLUE_200)
            ),
            Span::styled(
                " Copy Public Key",
                Style::default(),
            ),
            Span::raw(" "),
            Span::styled(
                "<S>",
                Style::default().fg(GREY_800).bg(BLUE_200)
            ),
            Span::styled(
                " Copy Secret Key",
                Style::default(),
            ),
        ])
    ];

    let commands_add_to_whitelist = vec![
        Spans::from(vec![
            Span::styled(
                "<Esc>",
                Style::default().fg(GREY_800).bg(BLUE_200)
            ),
            Span::styled(
                " Cancel",
                Style::default(),
            ),
            Span::raw(" "),
            Span::styled(
                "<Enter>",
                Style::default().fg(GREY_800).bg(BLUE_200)
            ),
            Span::styled(
                " Add to Whitelist",
                Style::default(),
            ),
        ])
    ];

    let commands_add_to_blacklist = vec![
        Spans::from(vec![
            Span::styled(
                "<Esc>",
                Style::default().fg(GREY_800).bg(BLUE_200)
            ),
            Span::styled(
                " Cancel",
                Style::default(),
            ),
            Span::raw(" "),
            Span::styled(
                "<Enter>",
                Style::default().fg(GREY_800).bg(BLUE_200)
            ),
            Span::styled(
                " Add to Blacklist",
                Style::default(),
            ),
        ])
    ];


    let commands_layout = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([Constraint::Percentage(100)])
        .split(main_area[2]);

    let commands_block = Block::default()
        .borders(Borders::NONE)
        .title_alignment(Alignment::Center)
        .border_type(BorderType::Rounded)
        .border_style( Style::default().fg(GREY_800));

    let commands_paragraph = Paragraph::new(
        match app.panes[app.pane_index].action {
            MenuAction::ShowAccountInfo(_) => commands_account_info,
            MenuAction::GenerateAccount(_) => commands_account_editing,
            MenuAction::AddToWhiteList(_) => commands_add_to_whitelist,
            MenuAction::AddToBlackList(_) => commands_add_to_blacklist,
            _ => commands_normal,
        })
        .style(Style::default())
        .block(commands_block)
        .alignment(Alignment::Left);
    f.render_widget(commands_paragraph, commands_layout[0]);
}