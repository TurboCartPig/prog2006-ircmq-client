use std::sync::mpsc;
use crate::message::*;

/// Send messages to the server and process replies.
pub fn chat_task(req_socket: zmq::Socket, receiver: mpsc::Receiver<MessageType>) -> anyhow::Result<()> {
    let mut msg = zmq::Message::new();

    while let Ok(message) = receiver.recv() {
        let message_string = serde_json::to_string(&message)?;
        req_socket.send(&message_string, 0)?;
        req_socket.recv(&mut msg, 0)?;
    }

    Ok(())
}

/// Receive messages from the server and deserialize them.
pub fn feed_task(
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

/// Create the ZMQ sockets and spawn the processing tasks associated with them.
pub fn create_sockets(
    channel: &str,
    server: &str,
) -> anyhow::Result<(mpsc::Sender<MessageType>, mpsc::Receiver<MessageType>)> {
    // Create zmq context and sockets
    let context = zmq::Context::new();

    let (sender, receiver) = mpsc::channel();
    let (server_sender, server_receiver) = mpsc::channel();

    let req_socket = context.socket(zmq::REQ)?;
    req_socket.connect(&format!("tcp://{}:5555", server))?;

    let sub_socket = context.socket(zmq::SUB)?;
    sub_socket.set_subscribe(channel.as_ref())?;
    sub_socket.connect(&format!("tcp://{}:6666", server))?;

    std::thread::spawn(move || chat_task(req_socket, receiver).unwrap());
    std::thread::spawn(move || feed_task(sub_socket, server_sender).unwrap());

    Ok((sender, server_receiver))
}