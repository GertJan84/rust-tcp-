use std::io;
use std::error::Error;
use std::net::SocketAddr;
use tokio::{
    io::{AsyncReadExt, Interest, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};
use std::sync::{Arc};
use tokio::sync::{Mutex, MutexGuard};
use structs::{Client, Protocol, Channel};

mod structs;



#[tokio::main]
async fn main() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:1234").await?;
    let channels: Arc<Mutex<Vec<Channel>>> = Arc::new(Mutex::new(Vec::new()));

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

async fn handle_socket(stream: TcpStream, addr: SocketAddr, channels: Arc<Mutex<Vec<Channel>>>) -> Result<(), Box<dyn Error>> {
    println!("{:?}", addr.port());
    let mut client = Client::new(addr);
    let mut buf: [u8; 1024];
    let mut stream = stream;

    stream.write_all(b"Enter username: ").await.unwrap();


    loop {
        let ready = stream
        .ready(Interest::READABLE | Interest::WRITABLE)
        .await?;   

        if ready.is_readable() {
            buf = [0; 1024];

            
            match stream.try_read(&mut buf) {
                Ok(n) => {
                    println!("{}", client);
                    match client.protocol { 
                        Protocol::Connect => {

                            let mut guard: MutexGuard<'_, Vec<Channel>> = channels.lock().await;
                            let response = std::str::from_utf8(&buf[..n - 1]).unwrap();
                            client.username = response.to_string();
                            client.protocol = Protocol::Join;
                            for channel in guard.iter() {
                                let message = format!("Channel name {}\n", channel);
                                stream.write_all(message.as_bytes()).await.unwrap()
                            }
                            // Show all rooms
                            stream.write_all(b"Name the channel you want to join (or make new): ").await.unwrap();
                            drop(guard);

                        },
                        Protocol::Join => {
                            let mut guard: MutexGuard<'_, Vec<Channel>> = channels.lock().await;
                            let response = std::str::from_utf8(&buf[..n - 1]).unwrap();

                            let found_channel = guard.iter_mut().find(|c| c.name ==  response);
                            if found_channel.is_none() {
                                let mut new_channel = Channel::new(response.to_string());
                                new_channel.users.push(client.clone());
                                guard.push(new_channel);
                                
                            } else {
                                let channel = found_channel.unwrap();
                                channel.users.push(client.clone());
                            }

                            drop(guard);
                
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
