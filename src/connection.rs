use std::io::{BufReader, BufWriter, Result, Error, ErrorKind};
use std::io::prelude::*;
use std::net::{TcpStream, SocketAddr};
use std::sync::Mutex;

pub trait Connection {
    fn send(&self, msg: &str) -> Result<()>;
    fn recv(&self) -> Result<String>;
    fn reconnect(&self) -> Result<()>;
    fn add_reconnect_attempt(&self) -> Result<()>;
    fn get_reconnect_attempts(&self) -> u8;
}

pub struct NetConnection {
    socket: SocketAddr,
    reconnect_attempts: Mutex<u8>,
    reader: Mutex<BufReader<TcpStream>>,
    writer: Mutex<BufWriter<TcpStream>>,
}

impl PartialEq for NetConnection {
    fn eq(&self, other: &NetConnection) -> bool {
        self.socket == other.socket
    }
}

#[allow(dead_code)]
impl NetConnection {
    fn new(
        host: &str,
        port: u16,
        reader: BufReader<TcpStream>,
        writer: BufWriter<TcpStream>,
    ) -> Self {
        NetConnection {
            socket: SocketAddr::new(host.parse().unwrap(), port),
            reconnect_attempts: Mutex::new(0),
            reader: Mutex::new(reader),
            writer: Mutex::new(writer),
        }
    }

    pub fn connect(host: &str, port: u16) -> Result<Self> {
        let socket = TcpStream::connect(&format!("{}:{}", host, port))?;
        Ok(NetConnection::new(
            host,
            port,
            BufReader::new(socket.try_clone()?),
            BufWriter::new(socket),
        ))
    }
}

impl From<TcpStream> for NetConnection {
    fn from(stream: TcpStream) -> NetConnection {
        let socket = stream.peer_addr().unwrap();
        NetConnection {
            socket: socket,
            reconnect_attempts: Mutex::new(0),
            reader: Mutex::new(BufReader::new(stream.try_clone().unwrap())),
            writer: Mutex::new(BufWriter::new(stream)),
        }
    }
}

impl Connection for NetConnection {
    fn send(&self, msg: &str) -> Result<()> {
        let mut writer = self.writer.lock().unwrap();
        writer.write_all(msg.as_bytes())?;
        writer.flush()
    }

    fn recv(&self) -> Result<String> {
        let mut ret = String::new();
        self.reader.lock().unwrap().read_line(&mut ret)?;
        if ret.is_empty() {
            Err(Error::new(ErrorKind::Other, "EOF"))
        } else {
            Ok(ret)
        }
    }

    fn reconnect(&self) -> Result<()> {
        let socket = TcpStream::connect(self.socket)?;
        *self.reader.lock().unwrap() = BufReader::new(socket.try_clone()?);
        *self.writer.lock().unwrap() = BufWriter::new(socket);
        *self.reconnect_attempts.lock().unwrap() = 0;
        println!("Reconnect success!");
        Ok(())
    }

    fn add_reconnect_attempt(&self) -> Result<()> {
        (*self.reconnect_attempts.lock().unwrap()) += 1;
        Ok(())
    }

    fn get_reconnect_attempts(&self) -> u8 {
        *self.reconnect_attempts.lock().unwrap()
    }
}
