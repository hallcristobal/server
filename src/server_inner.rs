use parking_lot::RwLock;
use rand::{self, Rng};

use connection::Connection;
use connection::NetConnection;
use client::Client;
use super::ServerMap;

use std::collections::HashMap;
use std::io::Result;
use std::net::TcpStream;
use std::sync::Arc;

pub struct ServerInner {
    pub host: RwLock<NetConnection>,
    pub clients: ServerMap<Client>,
}

impl ServerInner {
    pub fn new(stream: TcpStream) -> Self {
        let stream = NetConnection::from(stream);
        ServerInner {
            host: RwLock::new(stream),
            clients: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn add_client(&self, conn: NetConnection) -> Result<()> {
        let mut clients = self.clients.write();
        let index = rand::thread_rng().gen::<usize>();
        let client = Client::new(conn, index, self.clients.clone());
        clients.insert(index, client);
        Ok(())
    }

    pub fn send_to_host(&self, message: &str) -> Result<()> {
        let host = self.host.read();
        host.send(message)?;
        Ok(())
    }

    pub fn send_message(&self, message: &str) -> Result<()> {
        let clients = self.clients.read();
        for (_, v) in clients.iter() {
            v.send(message)?;
        }
        Ok(())
    }

    pub fn kill_clients(&self) {
        let clients = self.clients.read();
        for (k, v) in clients.iter() {
            println!("Notifying {}", k);
            v.notify_kill();
        }
    }
}
