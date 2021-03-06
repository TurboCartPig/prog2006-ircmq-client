use crate::message::*;
use anyhow::Context;
use std::sync::mpsc;
use std::thread;

/// Send messages to the server and process replies.
pub fn chat_task(
    name: String,
    channel: String,
    req_socket: zmq::Socket,
    receiver: mpsc::Receiver<MessageType>,
) -> anyhow::Result<()> {
    let mut msg = zmq::Message::new();

    // Send hello message, notifying the server
    // that the client is connecting.
    let hello = MessageType::Hello {
        name: name.clone(),
        channel: channel.clone(),
    };
    let hello = serde_json::to_string(&hello).context("Failed to serialize hello")?;
    req_socket.send(&hello, 0)?;
    req_socket.recv(&mut msg, 0)?;

    // Forward any message we receive to the server,
    // until the channel is closed.
    while let Ok(message) = receiver.recv() {
        let message = serde_json::to_string(&message).context("Failed to serialize message")?;
        req_socket.send(&message, 0)?;
        req_socket.recv(&mut msg, 0)?;
    }

    // Send goodbye message, notifying the server
    // that the client is leaving.
    let goodbye = MessageType::Goodbye { name, channel };
    let goodbye = serde_json::to_string(&goodbye).context("Failed to serialize goodbye")?;
    req_socket.send(&goodbye, 0)?;
    req_socket.recv(&mut msg, 0)?;

    Ok(())
}

/// Receive messages from the server and deserialize them.
pub fn feed_task(
    sub_socket: zmq::Socket,
    server_sender: mpsc::Sender<MessageType>,
) -> anyhow::Result<()> {
    let mut msg = zmq::Message::new();

    // Block on listening to messages from server
    loop {
        sub_socket.recv(&mut msg, 0)?;
        sub_socket.recv(&mut msg, 0)?;
        let message = serde_json::from_str(
            msg.as_str()
                .expect("Failed to convert zmq message to string"),
        )
        .context("Failed to deserialize message")?;

        // If the channel has closed, quit
        if server_sender.send(message).is_err() {
            break;
        }
    }

    Ok(())
}

/// Create the ZMQ sockets and spawn the processing tasks associated with them.
pub fn create_sockets(
    name: String,
    channel: String,
    server: &str,
) -> anyhow::Result<(mpsc::Sender<MessageType>, mpsc::Receiver<MessageType>)> {
    // Create zmq context and sockets
    let context = zmq::Context::new();

    let (to_server_sender, to_server_receiver) = mpsc::channel();
    let (from_server_sender, from_server_receiver) = mpsc::channel();

    let req_socket = context
        .socket(zmq::REQ)
        .context("Failed to create request socket")?;
    req_socket
        .connect(&format!("tcp://{}:5555", server))
        .context("Failed to connect to reply socket")?;

    let sub_socket = context
        .socket(zmq::SUB)
        .context("Failed to create subscriber")?;
    sub_socket.set_subscribe(b"broadcast")?;
    sub_socket.set_subscribe(channel.as_ref())?;
    sub_socket
        .connect(&format!("tcp://{}:6666", server))
        .context("Failed to connect to publisher")?;

    thread::spawn(move || {
        chat_task(name, channel, req_socket, to_server_receiver)
            .context("Failed to complete chat task")
            .unwrap()
    });
    thread::spawn(move || {
        feed_task(sub_socket, from_server_sender)
            .context("Failed to complete feed task")
            .unwrap()
    });

    Ok((to_server_sender, from_server_receiver))
}
