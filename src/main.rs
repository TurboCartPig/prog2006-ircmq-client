use crossterm::{
    event::{self, Event as CEvent, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use serde::{Deserialize, Serialize};
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

/// Messages to be serialized and sent to the server.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(tag = "tag")]
enum MessageType {
    /// Initial message sent when a client first connects.
    Hello { name: String, channel: String },
    /// A text message sent from a client to all the other clients, through the server.
    Message {
        name: String,
        channel: String,
        content: String,
    },
    /// Request a list of members of the current channel.
    RequestMembers { channel: String },
    /// Respond with a list of members of the current channel.
    ResponseMembers { members: Vec<String> },
    /// Final message from client to server, notifying the server, that the client is disconnecting.
    Goodbye { name: String, channel: String },
}

/// TUI event
enum Event {
    /// Input event generated from the user pressing a key.
    Input(crossterm::event::KeyEvent),
    /// Tick event generated from the tick thread in order to regularily update the UI.
    Tick,
}

// Generate tick and input events for the TUI.
// This was yanked from the tui-rs examples
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

// Some of this was yanked from the tui-rs examples
fn termui(
    sender: mpsc::Sender<MessageType>,
    server_receiver: mpsc::Receiver<MessageType>,
) -> anyhow::Result<()> {
    // Enable raw mode for the terminal
    enable_raw_mode()?;

    // Transistion the terminal from the main screen to the alternative screen
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;

    // Construct a Terminal abscraction from stdout
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Setup input handling
    let (event_sender, event_receiver) = mpsc::channel();
    let tick_rate = Duration::from_millis(100);
    thread::spawn(move || tick_task(tick_rate, event_sender));

    let mut feed = String::new();
    let mut input = String::new();
    let users = vec![String::from("Sebern")];
    let channels = vec![String::from("A")];

    loop {
        // Block on event input, either a tick to refresh the UI, or an input event from the user
        match event_receiver.recv()? {
            Event::Input(ev) => match ev.code {
                KeyCode::Esc => {
                    break;
                }
                KeyCode::Enter => {
                    // TODO: Take these as input
                    let name = String::from("Sebern");
                    let channel = String::from("A");

                    let content: String = input.drain(..).collect();
                    let message = MessageType::Message {
                        name,
                        channel,
                        content,
                    };
                    sender.send(message)?;
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
        while let Ok(MessageType::Message { name, content, .. }) = server_receiver.try_recv() {
            feed.push_str(&format!("{} -> {}\n", name, content));
        }

        // Draw the TUI
        terminal.draw(|f| {
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

            let channels: Vec<_> = channels
                .iter()
                .map(|channel| ListItem::new(channel.as_ref()))
                .collect();
            let channels_box =
                List::new(channels).block(Block::default().title("Channels").borders(Borders::ALL));
            f.render_widget(channels_box, chunks[0]);

            let vertical_chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(0)
                .constraints([Constraint::Percentage(90), Constraint::Percentage(10)].as_ref())
                .split(chunks[1]);

            let feed_box = Paragraph::new(feed.as_ref())
                .block(Block::default().title("Feed").borders(Borders::ALL));
            f.render_widget(feed_box, vertical_chunks[0]);

            let input_box = Paragraph::new(input.as_ref())
                .block(Block::default().title("Input").borders(Borders::ALL));
            f.render_widget(input_box, vertical_chunks[1]);

            let users: Vec<_> = users
                .iter()
                .map(|user| ListItem::new(user.as_str()))
                .collect();
            let users_box =
                List::new(users).block(Block::default().title("Users").borders(Borders::ALL));
            f.render_widget(users_box, chunks[2]);
        })?;
    }

    // Disable raw mode for the terminal, and switch back to the main screen
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;

    Ok(())
}

fn chat_task(req_socket: zmq::Socket, receiver: mpsc::Receiver<MessageType>) -> anyhow::Result<()> {
    let mut msg = zmq::Message::new();

    while let Ok(message) = receiver.recv() {
        let message_string = serde_json::to_string(&message)?;
        req_socket.send(&message_string, 0)?;
        req_socket.recv(&mut msg, 0)?;
    }

    Ok(())
}

fn print_task(
    sub_socket: zmq::Socket,
    server_sender: mpsc::Sender<MessageType>,
) -> anyhow::Result<()> {
    loop {
        let _ = sub_socket.recv_string(0)?.unwrap();
        let message = sub_socket.recv_string(0)?.unwrap();
        let message = serde_json::from_str(&message).expect("Serde");

        // If the channel has closed, quit
        if server_sender.send(message).is_err() {
            break;
        }
    }

    Ok(())
}

fn main() -> anyhow::Result<()> {
    let context = zmq::Context::new();

    let (sender, receiver) = mpsc::channel();
    let (server_sender, server_receiver) = mpsc::channel();

    let req_socket = context.socket(zmq::REQ)?;
    req_socket.connect("tcp://localhost:5555")?;

    let sub_socket = context.socket(zmq::SUB)?;
    sub_socket.set_subscribe(b"A")?;
    sub_socket.connect("tcp://localhost:6666")?;

    let t1 = std::thread::spawn(move || chat_task(req_socket, receiver).unwrap());
    let t2 = std::thread::spawn(move || print_task(sub_socket, server_sender).unwrap());

    termui(sender, server_receiver)?;

    println!("Hei");
    t1.join().unwrap();
    println!("Hei2");
    t2.join().unwrap();

    Ok(())
}

#[cfg(test)]
mod test {
    #[test]
    fn test_serialize_message() {
        let message = crate::MessageType::Message {
            name: "Sebern".into(),
            channel: "A".into(),
            content: "Heihei".into(),
        };

        let message =
            serde_json::to_string(&message).expect("Serde failed to serialize MessageType::Mesage");

        assert_eq!(
            message,
            "{\"tag\":\"Message\",\"name\":\"Sebern\",\"channel\":\"A\",\"content\":\"Heihei\"}"
        );
    }

    #[test]
    fn test_deserialize_message() {
        let message: &str =
            "{\"tag\":\"Message\",\"name\":\"Sebern\",\"channel\":\"A\",\"content\":\"Heihei\"}";

        let message: crate::MessageType = serde_json::from_str(message)
            .expect("Serde failed to deserialize MessageType::Message");

        assert_eq!(
            message,
            crate::MessageType::Message {
                name: "Sebern".into(),
                channel: "A".into(),
                content: "Heihei".into(),
            }
        );
    }
}
