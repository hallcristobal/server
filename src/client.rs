use parking_lot::RwLock;

use connection::Connection;
use connection::NetConnection;
use super::ServerMap;

use std::io::Result;
use std::sync::Arc;
use std::sync::mpsc::sync_channel;
use std::sync::mpsc::{SyncSender, Receiver};
use std::thread;
use std::error::Error;

pub struct Client {
    conn: Arc<RwLock<NetConnection>>,
    tx: SyncSender<bool>,
}

impl Client {
    pub fn new(conn: NetConnection, index: usize, list: ServerMap<Client>) -> Self {
        let conn = Arc::new(RwLock::new(conn));
        let i = conn.clone();

        let (tx, rx) = sync_channel(1);

        thread::spawn(move || {
            run(rx, i);
            println!("Removing Client: {}", index);
            let mut list = list.write();
            list.remove(&index);
        });

        Client { conn: conn, tx: tx }
    }

    pub fn send(&self, m: &str) -> Result<()> {
        let conn = self.conn.read();
        conn.send(m)?;
        Ok(())
    }
    pub fn notify_kill(&self) {
        let _ = self.tx.send(true).map_err(|_| {
            println!("Couldn't send message to thread")
        });
    }
}

fn run(rx: Receiver<bool>, i: Arc<RwLock<NetConnection>>) {
    loop {
        if let Ok(v) = rx.try_recv() {
            println!("CLIENT RECIEVED STOP");
            if v {
                return;
            }
        }

        let response = i.read().recv();
        match response {
            Ok(_) => {}
            Err(ref err) if err.description() == "operation would block" => {}
            Err(_) => return,
        }
    }
}
