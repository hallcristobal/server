use connection::Connection;
use connection::NetConnection;
use super::ServerMap;
use server_inner::ServerInner;

use std::error::Error;
use std::io::Result;
use std::net::TcpStream;
use std::sync::Arc;
use std::thread;

pub struct Server {
    inner: Arc<ServerInner>,
    index: usize,
    master_list: ServerMap<Server>,
}

impl Server {
    pub fn new(stream: TcpStream, list: ServerMap<Server>, index: usize) -> Self {
        Server {
            inner: Arc::new(ServerInner::new(stream)),
            index: index,
            master_list: list,
        }
    }

    pub fn run(&self) -> Result<()> {
        let socket = self.inner.clone();
        let map = self.master_list.clone();
        let index = Arc::new(self.index);
        self.inner.send_to_host(&format!("Key: {}\r\n", self.index))?;
        thread::spawn(move || {
            loop {
                let connection = socket.host.read();
                let response = connection.recv();
                match response {
                    Ok(msg) => {
                        println!("Recieved: {}", msg.trim());
                        let _ = socket.send_message(&format!("SEND: {}", msg)).map_err(
                            |_| {
                                println!("Failed to send message to clients")
                            },
                        );
                    }
                    Err(ref err) if err.description() == "EOF" => {}
                    Err(_) => break,
                }
            }
            println!(
                "Host disconnected, ending thread \"{:?}\"",
                thread::current().id()
            );
            // Stop all client threads
            socket.kill_clients();

            // Remove self from master list of servers
            let mut _map = map.write();
            _map.remove(&index);
        });
        Ok(())
    }

    pub fn add_client(&self, stream: TcpStream) -> Result<()> {
        self.inner.add_client(NetConnection::from(stream))?;
        Ok(())
    }
}
