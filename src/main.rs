use std::io;
use std::error::Error;
use std::net::SocketAddr;
use tokio::{
    io::{AsyncReadExt, Interest, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};
use std::sync::{Arc};
use tokio::sync::{Mutex, MutexGuard};
#[derive(Debug)]
enum Protocol {
    Connect,
    Join,
    Send,
    Dissconnect,
    Users
}  
#[derive(Debug)]
struct Client {
    username:String,
    channel: Option<usize>,
    address: SocketAddr,
    protocol: Protocol,
}

impl Client {
    fn new(addr: SocketAddr) -> Self {
        Client{
            username: String::from(""),
            channel: None,
            address:addr,
            protocol: Protocol::Connect,
        }
    }
}



#[tokio::main]
async fn main() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:1234").await?;
    let channels: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));

    loop {
        let channels_clone = Arc::clone(&channels);
        match listener.accept().await {
            Ok((stream, addr)) => {
                tokio::spawn(async move {
                    handle_socket(stream, addr, channels_clone).await;
                });
            },
            Err(e) => println!("couldn't get client: {:?}", e),
        }

    }
}

async fn handle_socket(stream: TcpStream, addr: SocketAddr, channels: Arc<Mutex<Vec<String>>>) -> Result<(), Box<dyn Error>> {
    println!("{:?}", addr.port());
    let mut client = Client::new(addr);
    let mut buf: [u8; 1024];
    let mut stream = stream;

    stream.write_all(b"Enter username: ").await.unwrap();

    let mut guard: MutexGuard<'_, Vec<String>> = channels.lock().await;

    guard.push("gj".to_string());

    loop {
        let ready = stream
        .ready(Interest::READABLE | Interest::WRITABLE)
        .await?;   

        if ready.is_readable() {
            buf = [0; 1024];

            
            match stream.try_read(&mut buf) {
                Ok(n) => {
                    match client.protocol { 
                        Protocol::Connect => {
                            let response = std::str::from_utf8(&buf[..n - 1]).unwrap();
                            client.username = response.to_string();
                            client.protocol = Protocol::Join;
                            for channel in guard.iter() {
                                let message = format!("Channel name {}\n", channel);
                                stream.write_all(message.as_bytes()).await.unwrap()
                            }
                            // Show all rooms
                            stream.write_all(b"Name the channel you want to join (or make new): ").await.unwrap();

                        },
                        Protocol::Join => {
                            let response = std::str::from_utf8(&buf[..n - 1]).unwrap();

                            for (index, value) in guard.iter().enumerate() {
                                if value == response {
                                    client.channel = Some(index);
                                    break;
                                }
                            }

                            if client.channel.is_none()  {
                                guard.push(response.to_string());
                                client.channel = Some(guard.len() - 1);
                            };
                
                            println!("{} joined {}", client.username, client.channel.unwrap())
                        },
                        _ => todo!()
                    }
                    println!("read {} bytes", n);
                }
                Err(ref e) if e.kind() ==  io::ErrorKind::WouldBlock => {
                    continue;
                }
                Err(e) => {
                    return Err(e.into());
                }
            }
        }

    }
}
