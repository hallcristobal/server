extern crate parking_lot;
extern crate rand;
extern crate schedule_recv;

mod connection;
mod server;
mod client;
mod server_inner;


use parking_lot::RwLock;
use server::Server;
use rand::Rng;

use std::io::Result;
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;
use std::collections::HashMap;

pub type ServerMap<T> = Arc<RwLock<HashMap<usize, T>>>;

pub static mut TICK_RATE: u8 = 64;

#[allow(unused_assignments)]
fn main() {
    let server_map: ServerMap<Server> = Arc::new(RwLock::new(HashMap::new()));
    let mut rng = rand::thread_rng();
    let listener = TcpListener::bind("127.0.0.1:1337").expect("Unable to bind listener");

    println!("Listening on {}", listener.local_addr().unwrap());
    for stream in listener.incoming() {
        // main listening loop
        if let Ok(stream) = stream {
            let mut key = 0;
            {
                // scope so read lock drops after getting a usable key
                let map = server_map.read();
                loop {
                    key = rng.gen_range::<usize>(100000, 999999); // generate a 6 didget key
                    if !map.contains_key(&key) {
                        break;
                    }
                }
            }
            println!("Key: {}", key);
            let _ = handle_client(stream, key, server_map.clone()).map_err(|e| {
                println!("Error when making server {}", e);
            });
        } else {
            println!("Error unwrappng stream");
        }
    }
}

#[allow(unused_assignments)]
fn handle_client(stream: TcpStream, key: usize, map: ServerMap<Server>) -> Result<()> {
    let peer_addr = stream.peer_addr()?;
    println!("Stream incomming at {}", peer_addr);
    let mut len = 0;
    {
        len = map.read().len();
    }
    if len > 0 {
        stream.set_nonblocking(true)?;
        let map = map.write();
        let mut ve = Vec::new();
        for (k, _) in map.iter() {
            ve.push(k);
        }
        if let Some(server) = map.get(ve[0]) {
            server.add_client(stream)?;
            println!("Added stream \"{}\"as client to {}", peer_addr, ve[0]);
        }
        Ok(())
    } else {
        let server = Server::new(stream, map.clone(), key);
        let _ = server.run()?;
        let mut map = map.write();
        if let Some(_) = map.insert(key, server) {
            Ok(())
        } else {
            Ok(())
        }
    }
}
