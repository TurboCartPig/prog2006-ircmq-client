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
    widgets::{Block, Borders, Paragraph, Widget},
    Terminal,
};

/// Messages to be serialized and sent to the server.
#[derive(Clone, Debug, Deserialize, Serialize)]
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

enum Event<I> {
    Input(I),
    Tick,
}

fn termui(sender: mpsc::Sender<String>) -> anyhow::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Setup input handling
    let (tx, rx) = mpsc::channel();

    let tick_rate = Duration::from_millis(100);
    thread::spawn(move || {
        let mut last_tick = Instant::now();
        loop {
            // poll for tick rate duration, if no events, sent tick event.
            let timeout = tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or_else(|| Duration::from_secs(0));
            if event::poll(timeout).unwrap() {
                if let CEvent::Key(key) = event::read().unwrap() {
                    tx.send(Event::Input(key)).unwrap();
                }
            }
            if last_tick.elapsed() >= tick_rate {
                tx.send(Event::Tick).unwrap();
                last_tick = Instant::now();
            }
        }
    });

    let mut input = String::new();

    loop {
        match rx.recv()? {
            Event::Input(ev) => match ev.code {
                KeyCode::Esc => {
                    break;
                }
                KeyCode::Enter => {
                    let msg: String = input.drain(..).collect();
                    sender.send(msg)?;
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

            let channels = Block::default().title("Channels").borders(Borders::ALL);
            f.render_widget(channels, chunks[0]);

            let input_box = Paragraph::new(input.as_ref())
                .block(Block::default().title("Input").borders(Borders::ALL));
            f.render_widget(input_box, chunks[1]);

            let users = Block::default().title("Users").borders(Borders::ALL);
            f.render_widget(users, chunks[2]);
        })?;
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;

    Ok(())
}

fn chat_task(req_socket: zmq::Socket, receiver: mpsc::Receiver<String>) -> anyhow::Result<()> {
    let mut msg = zmq::Message::new();

    loop {
        let content = match receiver.recv() {
            Ok(content) => content,
            Err(_) => break,
        };
        let message = MessageType::Message {
            name: "BOB".to_string(),
            channel: "A".to_string(),
            content,
        };
        let message_string = serde_json::to_string(&message)?;

        req_socket.send(&message_string, 0)?;
        req_socket.recv(&mut msg, 0)?;
    }

    Ok(())
}

fn print_task(sub_socket: zmq::Socket) -> anyhow::Result<()> {
    let mut msg = zmq::Message::new();

    loop {
        sub_socket.recv_string(0)?.unwrap();
        sub_socket.recv(&mut msg, 0)?;
        println!("Received: {}", msg.as_str().unwrap());
    }
}

fn main() -> anyhow::Result<()> {
    let context = zmq::Context::new();
    let server = std::env::args().nth(1);

    if server.is_some() {
        let rep_socket = context.socket(zmq::REP)?;
        rep_socket.bind("tcp://*:5555")?;

        let pub_socket = context.socket(zmq::PUB)?;
        pub_socket.bind("tcp://*:6666")?;

        let mut msg = zmq::Message::new();

        loop {
            rep_socket.recv(&mut msg, 0)?;
            println!("Received: {}", msg.as_str().unwrap());
            rep_socket.send("ACK", 0)?;
            pub_socket.send("A", zmq::SNDMORE)?;
            pub_socket.send(msg.as_str().unwrap(), 0)?;
        }
    } else {
        let (sender, receiver) = mpsc::channel();

        let req_socket = context.socket(zmq::REQ)?;
        req_socket.connect("tcp://localhost:5555")?;

        let sub_socket = context.socket(zmq::SUB)?;
        sub_socket.set_subscribe(b"A")?;
        sub_socket.connect("tcp://localhost:6666")?;

        let t1 = std::thread::spawn(move || chat_task(req_socket, receiver).unwrap());
        // let t2 = std::thread::spawn(move || print_task(sub_socket).unwrap());

        termui(sender)?;

        t1.join().unwrap();
        // t2.join().unwrap();
    }

    Ok(())
}
