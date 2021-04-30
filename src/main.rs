use serde::{Deserialize, Serialize};
use std::io;
use crate::MessageType::{Hello, Message};

/// Messages to be serialized and sent to the server.
#[derive(Clone, Debug, Deserialize, Serialize)]
enum MessageType {
    /// Initial message sent when a client first connects.
    Hello { name: String },
    /// A text message sent from a client to all the other clients, through the server.
    Message { content: String },
    /// Final message from client to server, notifying the server, that the client is leaving.
    Goodbye,
}

fn main() {
    let context = zmq::Context::new();
    let server = std::env::args().nth(1);

    if server.is_some() {
        let rep_socket = context.socket(zmq::REP).unwrap();
        rep_socket.bind("tcp://*:5555").unwrap();

        let pub_socket = context.socket(zmq::PUB).unwrap();
        pub_socket.bind("tcp://*:6666").unwrap();

        let mut msg = zmq::Message::new();

        loop {
            rep_socket.recv(&mut msg, 0).unwrap();
            println!("Received: {}", msg.as_str().unwrap());
            rep_socket.send("ACK", 0).unwrap();
            pub_socket.send("A", zmq::SNDMORE).unwrap();
            pub_socket.send(msg.as_str().unwrap(), 0).unwrap();
        }
    } else {
        let req_socket = context.socket(zmq::REQ).unwrap();
        req_socket.connect("tcp://localhost:5555").unwrap();

        let sub_socket = context.socket(zmq::SUB).unwrap();
        sub_socket.set_subscribe(b"A").unwrap();
        sub_socket.connect("tcp://localhost:6666").unwrap();

        let t1 = std::thread::spawn(move || {
            let mut msg = zmq::Message::new();
            let stdin = io::stdin();
            loop {
                let mut buffer = String::new();
                stdin.read_line(&mut buffer).unwrap();

                let message = Message { content: buffer.to_string() };
                let message_string = serde_json::to_string(&message).unwrap();

                req_socket.send(&message_string, 0).unwrap();
                req_socket.recv(&mut msg, 0).unwrap();
            }
        });

        let t2 = std::thread::spawn(move || {
            let mut msg = zmq::Message::new();

            loop {
                sub_socket.recv_string(0).unwrap().unwrap();
                sub_socket.recv(&mut msg, 0).unwrap();
                println!("Received: {}", msg.as_str().unwrap());
            }
        });

        t1.join().unwrap();
        t2.join().unwrap();
    }
}
