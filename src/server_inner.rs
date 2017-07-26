use parking_lot::RwLock;

use connection::Connection;
use connection::NetConnection;
use client::Client;

use std::io::Result;
use std::net::TcpStream;
use std::sync::Arc;

pub struct ServerInner {
    pub host: RwLock<NetConnection>,
    pub clients: Arc<RwLock<Vec<Client>>>,
}

impl ServerInner {
    pub fn new(stream: TcpStream) -> Self {
        let stream = NetConnection::from(stream);
        ServerInner {
            host: RwLock::new(stream),
            clients: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub fn add_client(&self, conn: NetConnection) -> Result<()> {
        let mut clients = self.clients.write();
        let client = Client::new(conn, clients.len(), self.clients.clone());
        clients.push(client);
        Ok(())
    }

    pub fn send_to_host(&self, message: &str) -> Result<()> {
        let host = self.host.read();
        host.send(message)?;
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
