use parking_lot::RwLock;

use connection::Connection;
use connection::NetConnection;
use super::ServerList;

use std::error::Error;
use std::io::Result;
use std::net::TcpStream;
use std::sync::Arc;
use std::thread;

pub struct ServerInner {
    pub host: RwLock<NetConnection>,
    pub clients: RwLock<Vec<NetConnection>>,
    pub master_list: Arc<ServerList>,
}

impl ServerInner {
    pub fn new(stream: TcpStream, list: Arc<ServerList>) -> Self {
        let stream = NetConnection::from(stream);
        ServerInner {
            host: RwLock::new(stream),
            clients: RwLock::new(Vec::new()),
            master_list: list,
        }
    }

	pub fn add_client(&self, conn: NetConnection) -> Result<()> {
		let mut clients = self.clients.write();
		clients.iter().any(|c| c == &conn);
		clients.push(conn);
		Ok(())
	}

    pub fn send_message(&self, message: &str) -> Result<()> {
        let clients = self.clients.read();
        for con in clients.iter() {
            con.send(message)?;
        }
        Ok(())
    }
}

pub struct Server(Arc<ServerInner>);

impl Server {
    pub fn new(stream: TcpStream, list: Arc<ServerList>) -> Self {
        Server(Arc::new(ServerInner::new(stream, list)))
    }

    pub fn run(&self) -> Result<()> {
        let socket = self.0.clone();
        thread::spawn(move || {
            loop {
                let connection = socket.host.read();
                let response = connection.recv();
                match response {
                    Ok(msg) => {
                        println!("Message received");
                        println!("Recieved: {}", msg);
                        let _ = socket
                            .send_message(&format!("SEND: {}", msg))
                            .map_err(|_| println!("Failed to send message to clients"));
                    }
                    Err(ref err) if err.description() == "EOF" => {}
                    Err(_) => break,
                }
            }
            println!(
                "Client disconnected, ending thread \"{:?}\"",
                thread::current().id()
            );
        });
        Ok(())
    }

    pub fn add_client(&self, stream: TcpStream) {
        let mut clients = self.0.clients.write();
        clients.push(NetConnection::from(stream));
    }
}

impl Drop for Server {
    fn drop(&mut self) {}
}
