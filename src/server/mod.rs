use std::net::{TcpListener, TcpStream, SocketAddr};
use std::io::Read;
use std::thread;


pub struct Client {
    stream: TcpStream,
    disconnect: bool,
}

impl Client {
    pub fn new(stream: TcpStream) -> Client {
        Client {
            stream,
            disconnect: false,
        }
    }

    fn addr(&mut self) -> Result<SocketAddr, Box<dyn std::error::Error + Send + Sync>> {
        self.stream.peer_addr().map_err(|err| err.into())
    }

    pub fn read(&mut self) -> Result<Option<Vec<u8>>, Box<dyn std::error::Error + Send + Sync>> {
        let mut buffer = [0u8; 128];

        match self.stream.read(&mut buffer) {
            Ok(bytes) => Ok((bytes > 0).then(|| buffer[..bytes].to_vec())),
            Err(err) => Err(Box::new(err)),
        }
    }

    pub fn handle(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        while !self.disconnect {
            match self.read()? {
                Some(bytes) => {
                    println!("[INFO] {:?} -> [{:?}, {:?}]", self.addr()?, bytes.clone(), String::from_utf8(bytes));
                },
                None => self.disconnect = true,
            }
        }

        println!("[INFO] client disconnected: {:?}", self.addr()?);

        Ok(())
    }
}

pub struct Listener {
    listener: TcpListener,
}

impl Listener {
    pub fn new(bind: &str) -> Result<Listener, Box<dyn std::error::Error + Send + Sync>> {
        Ok(Listener {
            listener: TcpListener::bind(bind)?,
        })
    }

    pub fn run(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        println!("[INFO] listening on `{:?}`", self.listener.local_addr()?);

        for stream in self.listener.incoming() {
            let stream = stream?;

            println!("[INFO] client connected: {}", stream.peer_addr()?);

            thread::spawn(move || {
                let mut client = Client::new(stream);

                client.handle()
            });
        }

        Ok(())
    }
}


