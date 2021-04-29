fn main() {
    let context = zmq::Context::new();
    let server = std::env::args().nth(1);

    if server.is_some() {
        let socket = context.socket(zmq::REP).unwrap();
        socket.bind("tcp://*:5555").unwrap();

        let mut msg = zmq::Message::new();

        loop {
            socket.recv(&mut msg, 0).unwrap();
            socket.send(msg.as_str().unwrap(), 0).unwrap();
        }
    } else {
        let socket = context.socket(zmq::REQ).unwrap();
        socket.connect("tcp://localhost:5555").unwrap();

        let mut msg = zmq::Message::new();

        for n in 0..10 {
            socket.send(&format!("Hello {}", n), 0).unwrap();
            socket.recv(&mut msg, 0).unwrap();
            println!("Received: {}", msg.as_str().unwrap());
        }
    }
}
