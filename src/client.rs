use crate::message::*;
use std::sync::mpsc;

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
    let hello = serde_json::to_string(&hello)?;
    req_socket.send(&hello, 0)?;
    req_socket.recv(&mut msg, 0)?;

    // TODO: Get list of other users in the same channel
    // TODO: Get list of channels on the server

    // Request members of the channel we are joining
    // let req = MessageType::RequestMembers {
    //     channel: channel.clone(),
    // };
    // let req = serde_json::to_string(&req)?;
    // req_socket.send(&req, 0)?;
    // let res = req_socket.recv_string(0)?.unwrap();
    // println!("Res: {}", res);

    // Forward any message we reveive to the server,
    // until the channel is closed.
    while let Ok(message) = receiver.recv() {
        let message = serde_json::to_string(&message)?;
        req_socket.send(&message, 0)?;
        req_socket.recv(&mut msg, 0)?;
    }

    // Send goodbye message, notifying the server
    // that the client is leaving.
    let goodbye = MessageType::Goodbye { name, channel };
    let goodbye = serde_json::to_string(&goodbye)?;
    req_socket.send(&goodbye, 0)?;
    req_socket.recv(&mut msg, 0)?;

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
    name: String,
    channel: String,
    server: &str,
) -> anyhow::Result<(
    mpsc::Sender<MessageType>,
    mpsc::Receiver<MessageType>,
    impl FnOnce(),
    impl FnOnce(),
)> {
    // Create zmq context and sockets
    let context = zmq::Context::new();

    let (to_server_sender, to_server_receiver) = mpsc::channel();
    let (from_server_sender, from_server_receiver) = mpsc::channel();

    let req_socket = context.socket(zmq::REQ)?;
    req_socket.connect(&format!("tcp://{}:5555", server))?;

    let sub_socket = context.socket(zmq::SUB)?;
    sub_socket.set_subscribe(b"broadcast")?;
    sub_socket.set_subscribe(channel.as_ref())?;
    sub_socket.connect(&format!("tcp://{}:6666", server))?;

    let t1 = move || chat_task(name, channel, req_socket, to_server_receiver).unwrap();
    let t2 = move || feed_task(sub_socket, from_server_sender).unwrap();

    Ok((to_server_sender, from_server_receiver, t1, t2))
}
