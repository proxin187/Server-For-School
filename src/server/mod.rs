use std::net::{TcpListener, TcpStream, SocketAddr};
use std::sync::{Arc, Mutex};
use std::io::{Read, Write};
use std::thread;

macro_rules! lock {
    ($mutex:expr) => {
        $mutex.lock().map_err(|_| Into::<Box<dyn std::error::Error + Send + Sync>>::into("failed to lock"))
    }
}


pub struct Client {
    clients: Arc<Mutex<Vec<TcpStream>>>,
    stream: TcpStream,
    disconnect: bool,
}

impl Client {
    pub fn new(stream: TcpStream, clients: Arc<Mutex<Vec<TcpStream>>>) -> Result<Client, Box<dyn std::error::Error + Send + Sync>> {
        lock!(clients)?.push(stream.try_clone()?);

        Ok(Client {
            clients,
            stream,
            disconnect: false,
        })
    }

    fn addr(&mut self) -> Result<SocketAddr, Box<dyn std::error::Error + Send + Sync>> {
        self.stream.peer_addr().map_err(|err| err.into())
    }

    fn read(&mut self) -> Result<Option<Vec<u8>>, Box<dyn std::error::Error + Send + Sync>> {
        let mut buffer = [0u8; 128];

        match self.stream.read(&mut buffer) {
            Ok(bytes) => Ok((bytes > 0).then(|| buffer[..bytes].to_vec())),
            Err(err) => Err(Box::new(err)),
        }
    }

    fn broadcast(&mut self, bytes: &[u8]) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        for stream in lock!(self.clients)?.iter_mut() {
            stream.write_all(bytes)?;
        }

        Ok(())
    }

    fn find(&mut self, addr: SocketAddr) -> Result<Option<usize>, Box<dyn std::error::Error + Send + Sync>> {
        let index = lock!(self.clients)?.iter()
            .enumerate()
            .find(|(_, stream)| stream.peer_addr().map_err(|_| ()) == Ok(addr))
            .map(|(idx, _)| idx);

        Ok(index)
    }

    pub fn handle(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        while !self.disconnect {
            match self.read()? {
                Some(bytes) => {
                    println!("[INFO] {:?} -> [{:?}, {:?}]", self.addr()?, bytes.clone(), String::from_utf8(bytes.clone()));

                    self.broadcast(&bytes)?;
                },
                None => self.disconnect = true,
            }
        }

        println!("[INFO] client disconnected: {:?}", self.addr()?);

        let addr = self.addr()?;

        if let Some(index) = self.find(addr)? {
            lock!(self.clients)?.remove(index);
        }

        Ok(())
    }
}

pub struct Listener {
    listener: TcpListener,
    clients: Arc<Mutex<Vec<TcpStream>>>,
}

impl Listener {
    pub fn new(bind: &str) -> Result<Listener, Box<dyn std::error::Error + Send + Sync>> {
        Ok(Listener {
            listener: TcpListener::bind(bind)?,
            clients: Arc::new(Mutex::new(Vec::new())),
        })
    }

    pub fn run(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        println!("[INFO] listening on `{:?}`", self.listener.local_addr()?);

        for stream in self.listener.incoming() {
            let stream = stream?;

            println!("[INFO] client connected: {}", stream.peer_addr()?);

            let mut client = Client::new(stream, self.clients.clone())?;

            thread::spawn(move || client.handle());
        }

        Ok(())
    }
}


