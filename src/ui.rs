//! This module deals with drawing a terminal UI, based on dynamic data,
//! like the state of the input field, the users in the channel, and so on.
//!
//! # Credits
//! Almost everything in this module is based on the excellent tui-rs examples provided at their
//! main repo.

use crate::client::*;
use crate::message::*;
use crossterm::{
    event::{self, Event as CEvent, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{
    io,
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};
use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Terminal,
};

/// TUI event.
enum Event {
    /// Input event generated from the user pressing a key.
    Input(crossterm::event::KeyEvent),
    /// Tick event generated from the tick thread in order to regularly update the UI.
    Tick,
}

/// Generate tick and input events for the TUI.
fn tick_task(tick_rate: Duration, event_sender: mpsc::Sender<Event>) {
    let mut last_tick = Instant::now();
    loop {
        // Poll for tick rate duration, if no new events, send tick event.
        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));
        if event::poll(timeout).unwrap() {
            if let CEvent::Key(key) = event::read().unwrap() {
                event_sender.send(Event::Input(key)).unwrap();
            }
        }
        if last_tick.elapsed() >= tick_rate {
            if event_sender.send(Event::Tick).is_err() {
                break;
            }
            last_tick = Instant::now();
        }
    }
}

/// Draw the TUI.
fn draw_ui(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    channels: &[String],
    users: &[String],
    feed: &str,
    input: &str,
) -> anyhow::Result<()> {
    terminal.draw(|f| {
        // Define the horizontal layout
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .margin(1)
            .constraints(
                [
                    Constraint::Percentage(20),
                    Constraint::Percentage(60),
                    Constraint::Percentage(20),
                ]
                .as_ref(),
            )
            .split(f.size());

        // Define the vertical layout
        let vertical_chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(0)
            .constraints([Constraint::Percentage(90), Constraint::Percentage(10)].as_ref())
            .split(chunks[1]);

        // Draw a list of channels in a sidebar
        let channels: Vec<_> = channels
            .iter()
            .map(|channel| ListItem::new(channel.as_ref()))
            .collect();
        let channels_box =
            List::new(channels).block(Block::default().title("Channels").borders(Borders::ALL));
        f.render_widget(channels_box, chunks[0]);

        // Draw the main message feed
        let feed_box =
            Paragraph::new(feed).block(Block::default().title("Feed").borders(Borders::ALL));
        f.render_widget(feed_box, vertical_chunks[0]);

        // Draw an input text box
        let input_box =
            Paragraph::new(input).block(Block::default().title("Input").borders(Borders::ALL));
        f.render_widget(input_box, vertical_chunks[1]);

        // Draw a list of users in a sidebar
        let users: Vec<_> = users
            .iter()
            .map(|user| ListItem::new(user.as_str()))
            .collect();
        let users_box =
            List::new(users).block(Block::default().title("Users").borders(Borders::ALL));
        f.render_widget(users_box, chunks[2]);
    })?;

    Ok(())
}

/// Run the TUI and process user input, until ESC is pressed.
pub fn termui(name: String, channel: String, server: String) -> anyhow::Result<()> {
    // Enable raw mode for the terminal
    enable_raw_mode()?;

    // Transition the terminal from the main screen to the alternative screen
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;

    // Construct a Terminal abstraction from stdout
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create the server <-> client sockets and spawn threads that poll on them
    let (to_server, from_server, t1, t2) = create_sockets(&channel, &server)?;
    let chat_thread = thread::spawn(t1);
    thread::spawn(t2);

    // Setup a thread to handle input events from the user
    // The thread also generates tick events to refresh the UI
    let (event_sender, event_receiver) = mpsc::channel();
    let tick_rate = Duration::from_millis(100);
    thread::spawn(move || tick_task(tick_rate, event_sender));

    // Data that drives the UI
    let mut feed = String::new();
    let mut input = String::new();
    let users = vec![name.clone()];
    let channels = vec![channel.clone()];

    loop {
        // Block on event input, either a tick to refresh the UI, or an input event from the user
        match event_receiver.recv()? {
            Event::Input(ev) => match ev.code {
                KeyCode::Esc => {
                    break;
                }
                KeyCode::Enter => {
                    let content: String = input.drain(..).collect();
                    let message = MessageType::Message {
                        name: name.clone(),
                        channel: channel.clone(),
                        content,
                    };
                    to_server.send(message)?;
                }
                KeyCode::Backspace => {
                    input.pop();
                }
                KeyCode::Char(c) => {
                    input.push(c);
                }
                _ => {}
            },
            Event::Tick => {}
        }

        // Pull new messages from the server, ignore any errors
        while let Ok(MessageType::Message { name, content, .. }) = from_server.try_recv() {
            feed.push_str(&format!("{} -> {}\n", name, content));
        }

        draw_ui(&mut terminal, &channels, &users, &feed, &input)?;
    }

    // Wait for the thread to send the goodbye message,
    // that was initiated by the channel closing.
    chat_thread.join().unwrap();

    // Disable raw mode for the terminal, and switch back to the main screen
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;

    Ok(())
}
