use parking_lot::RwLock;

use connection::Connection;
use connection::NetConnection;

use std::io::Result;
use std::sync::Arc;
use std::thread;

pub struct Client {
    conn: Arc<RwLock<NetConnection>>,
	index: Arc<usize>
}

impl Client {
    pub fn new(conn: NetConnection, index: usize, list: Arc<RwLock<Vec<Client>>>) -> Self {
        let conn = Arc::new(RwLock::new(conn));
        let i = conn.clone();
		let index = Arc::new(index);
		let i_i = index.clone();
        thread::spawn(move || {
            loop {
                let i = i.read();
                let response = i.recv();
                match response {
                    Ok(_) => {}
                    Err(_) => break,
                }
            }
            println!("Removing Client: {}", &index);
            let mut list = list.write();
            list.remove(&i_i);
        });
        Client { 
			conn: conn,
		 }
    }

    pub fn send(&self, m: &str) -> Result<()> {
        let conn = self.conn.read();
        conn.send(m)?;
        Ok(())
    }

	pub fn set_index(&mut self, index: usize) {

	}
}
