extern crate parking_lot;
mod connection;
mod server;

use parking_lot::RwLock;
use server::Server;

use std::io::Result;
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;

pub type ServerList = RwLock<Vec<Server>>;

fn main() {
    let mut servers = Arc::new(RwLock::new(Vec::new()));
    let listener = TcpListener::bind("127.0.0.1:1337").expect("Unable to bind listener");
    println!("Listening on {}", listener.local_addr().unwrap());
    for stream in listener.incoming() {
        // main listening loop
        if let Ok(stream) = stream {
            let _ = handle_client(stream, &mut servers).map_err(|e| {
                println!("Error when making server {}", e);
            });
        } else {
            println!("Error unwrappng stream");
        }
    }
}

fn handle_client(stream: TcpStream, servers: &mut Arc<ServerList>) -> Result<()> {
    println!("Client incomming at {}", stream.peer_addr()?);
    let server = Server::new(stream, servers.clone());
    let _ = server.run()?;
    // servers.get_mut().push(server); // bad?
    Ok(())
}
