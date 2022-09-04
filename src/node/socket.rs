use std::net::TcpStream;
use tungstenite::stream::MaybeTlsStream;
use tungstenite::{connect, Message, WebSocket};
use crate::node::calls::NodeRequest;
use crate::NodeError;

pub struct Socket {
    pub ws: WebSocket<MaybeTlsStream<TcpStream>>
}

impl Socket {
    pub fn new(node_url: &String) -> Result<Socket, NodeError> {
        let (socket, _response) = match connect(node_url) {
            Ok(res) => res,
            Err(e) => {
                eprintln!("Error: {:?}", e);
                return Err(NodeError::CouldNotConnectToNode);
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
                return Err(NodeError::CouldNotSendMessageToNode);
            }
        };

        if let Err(e) = &socket.ws.write_message(Message::Text(req)) {
            eprintln!("Error: {:?}", e);
            return Err(NodeError::CouldNotSendMessageToNode);
        };

        Ok(())
    }


    pub fn read(socket: &mut Socket) -> Result<String, NodeError> {
        loop {
            let msg = match socket.ws.read_message() {
                Ok(res) => res,
                Err(e) => {
                    eprintln!("Error: {:?}", e);
                    return Err(NodeError::CouldNotReadMessageFromNode);
                },
            };

            return match msg {
                tungstenite::Message::Text(res) => Ok(res),
                _ => continue
            }
        }
    }
}