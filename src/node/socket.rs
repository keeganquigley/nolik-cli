use std::net::TcpStream;
use tungstenite::stream::MaybeTlsStream;
use tungstenite::{connect, Message, WebSocket};
use crate::node::calls::NodeRequest;
use crate::NodeError;

pub struct Socket {
    pub ws: WebSocket<MaybeTlsStream<TcpStream>>
}

impl Socket {
    pub fn new() -> Result<Socket, NodeError> {
        let (socket, _response) = match connect("ws://127.0.0.1:9944") {
            Ok(res) => res,
            Err(e) => {
                eprintln!("Error: {:?}", e);
                return Err(NodeError::CouldNotGetAccountNonce);
            }
        };

        Ok(Socket { ws: socket })
    }

    pub fn close(&mut self) -> tungstenite::Result<()> {
        self.ws.close(None)
    }
}


pub struct SocketMessage {
    pub message: String,
}


impl SocketMessage {
    pub fn send(socket: &mut Socket, req: NodeRequest) -> Result<(), NodeError> {
        let req = match serde_json::to_string(&req) {
            Ok(res) => res,
            Err(e) => {
                eprintln!("Error: {:?}", e);
                return Err(NodeError::CouldNotGetAccountNonce);
            }
        };

        if let Err(e) = &socket.ws.write_message(Message::Text(req)) {
            eprintln!("Error: {:?}", e);
            return Err(NodeError::CouldNotGetAccountNonce);
        };

        Ok(())
    }


    pub fn read(socket: &mut Socket) -> Result<String, NodeError> {
        let msg = match socket.ws.read_message() {
            Ok(res) => res,
            Err(e) => {
                eprintln!("Error: {:?}", e);
                return Err(NodeError::CouldNotGetAccountNonce);
            },
        };

        match msg {
            tungstenite::Message::Text(res) => Ok(res),
            _ => {
                eprintln!("Node response format is other than Text");
                return Err(NodeError::CouldNotGetAccountNonce);
            }
        }
    }
}