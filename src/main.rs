use std::io;
use std::sync::Arc;
use structs::{Channel, Client, Commands, Protocol, UserCommandType};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::{Mutex, MutexGuard};
use tokio::{io::AsyncWriteExt, net::TcpListener, sync::broadcast};

mod structs;

async fn show_channels(
    client: Client,
    channels: Arc<Mutex<Vec<Channel>>>,
    mut writer: &'_ mut tokio::net::tcp::WriteHalf<'_>,
) -> io::Result<()> {
    client
        .send_message(&mut writer, "Channels | user_amount:".to_string(), true)
    .await;
    let guard: MutexGuard<'_, Vec<Channel>> = channels.lock().await;
    
    for channel in guard.iter() {
        if channel.visible {
            client
                .send_message(
                    &mut writer,
                    format!("- {} | {}", channel.name, channel.users.len()),
                    true,
                )
            .await;
        }
    }
    drop(guard);

    Ok(())
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:1234").await.unwrap();
    let real_channels: Arc<Mutex<Vec<Channel>>> = Arc::new(Mutex::new(Vec::new()));
    let (tx, _rx) = broadcast::channel(69);

    loop {
        let channels = Arc::clone(&real_channels);
        let tx = tx.clone();
        let mut rx = tx.subscribe();
        let (mut stream, addr) = listener.accept().await.unwrap();

        tokio::spawn(async move {
            let (reader, mut writer) = stream.split();
            let mut reader = BufReader::new(reader);
            let mut line = String::new();
            let mut client = Client::new(addr);

            loop {
                match client.protocol {
                    Protocol::Connect => {
                        client
                            .send_message(
                                &mut writer,
                                "Welcome to the server, please enter your username: ".to_string(),
                                false,
                            )
                            .await;

                        let name_size = reader.read_line(&mut line).await.unwrap();

                        match Client::handle_input(line[..name_size - 1].to_string()) {
                            UserCommandType::Message => {
                                client.username = line[..name_size - 1].to_string();
                                client
                                    .send_message(
                                        &mut writer,
                                        format!("Nice to see you, {}!\n", client.username),
                                        true,
                                    )
                                    .await;
                                client.protocol = Protocol::Join;
                            }
                            _ => {
                                client
                                    .send_message(
                                        &mut writer,
                                        "Invalid username!".to_string(),
                                        true,
                                    )
                                    .await;
                                continue;
                            }
                        };
                        line.clear();
                        client.protocol = Protocol::Join;
                    }
                    Protocol::Join => {
                        show_channels(client.clone(), channels.clone(), &mut writer).await.unwrap();
                        client
                            .send_message(
                                &mut writer,
                                "Please enter a channel name (or it will create a new channel): "
                                    .to_string(),
                                false,
                            )
                            .await;
                        let response_size = reader.read_line(&mut line).await.unwrap();
                        let response = line[..response_size - 1].to_string();

                        match Client::handle_input(response.clone()) {
                            UserCommandType::Invalid => {
                                client
                                    .send_message(&mut writer, "Invalid Input!".to_string(), true)
                                    .await;
                            }

                            UserCommandType::Message => {
                                let mut guard: MutexGuard<'_, Vec<Channel>> = channels.lock().await;
                                let found_channel = guard
                                    .iter_mut()
                                    .enumerate()
                                    .find(|(_, c)| c.name == response);
                                if found_channel.is_none() {
                                    let mut new_channel = Channel::new(response.to_string());
                                    client.channel_index = Some(guard.len());
                                    new_channel.users.push(client.clone());
                                    guard.push(new_channel);
                                    drop(guard)
                                } else {
                                    let (index, channel) = found_channel.unwrap();
                                    let found = channel
                                        .users
                                        .iter()
                                        .find(|c| c.username == client.username);
                                    if found.is_some() {
                                        drop(guard);
                                        client
                                            .send_message(
                                                &mut writer,
                                                "This username is already in use for this channel!"
                                                    .to_string(),
                                                true,
                                            )
                                            .await;
                                        continue;
                                    }
                                    client.channel_index = Some(index);
                                    channel.users.push(client.clone());
                                    drop(guard);
                                }

                                println!("{} joined channel {}", client.username, response);
                                tx.send((
                                    format!(
                                        "**{} joined channel {}!**\n",
                                        client.username, response
                                    ),
                                    addr,
                                    client.channel_index,
                                ))
                                .unwrap();
                                client
                                    .send_message(
                                        &mut writer,
                                        format!(
                                            "**{} joined channel {}!**",
                                            client.username, response
                                        ),
                                        true,
                                    )
                                    .await;

                                client.protocol = Protocol::Chat;
                            }
                        
                            UserCommandType::Command => {
                                match Commands::get_command(line.clone()) {
                                    Commands::Group => {
                                        show_channels(client.clone(), channels.clone(), &mut writer);
                                    }
                                    _ => {
                                        todo!();
                                    }
                                }
                            }
                        }

                        line.clear();
                    }
                    Protocol::Chat => {
                        tokio::select! {
                            result = reader.read_line(&mut line)=> {
                                if result.unwrap() == 0 {
                                    break;
                                }

                                match Client::handle_input(line.clone()) {
                                    UserCommandType::Message => {
                                        tx.send((line.clone(), addr, client.channel_index)).unwrap();
                                    },
                                    UserCommandType::Command => {
                                        println!("{} is trying to use a command\n{}",client.username, line);
                                        match Commands::get_command(line.clone()){
                                            Commands::Invalid | Commands::Group => {
                                                client.send_message(&mut writer, "Invalid command! (Possible: 'Exit', 'Disconnect', 'Users')".to_string(), true).await;

                                            },
                                            Commands::Leave => {
                                                let mut guard = channels.lock().await;
                                                let channel = guard.get_mut(client.channel_index.unwrap()).unwrap();
                                                let index = channel.users.iter().position(|c| c.username == client.username).unwrap();
                                                channel.users.remove(index);
                                                if channel.users.len() == 0 {
                                                    channel.visible = false;
                                                } else {
                                                    tx.send((format!("**{} left channel!**\n", client.username), addr, client.channel_index)).unwrap();
                                                    client.send_message(&mut writer, "Leaving...".to_string(), true).await;
                                                }
                                                drop(guard);
                                                client.protocol = Protocol::Join;
                                            }
                                            Commands::Exit => {
                                                let mut guard = channels.lock().await;
                                                let channel = guard.get_mut(client.channel_index.unwrap()).unwrap();
                                                let index = channel.users.iter().position(|c| c.username == client.username).unwrap();
                                                channel.users.remove(index);
                                                if channel.users.len() == 0 {
                                                    channel.visible = false;
                                                } else {
                                                    tx.send((format!("**{} left channel!**\n", client.username), addr, client.channel_index)).unwrap();
                                                    client.send_message(&mut writer, "Exiting...".to_string(), true).await;
                                                }
                                            },
                                            Commands::Users => {
                                                client.send_message(&mut writer, "Users:".to_string(), true).await;
                                                let guard = channels.lock().await;
                                                let channel = guard.get(client.channel_index.unwrap()).unwrap();
                                                for user in channel.users.iter(){
                                                    client.send_message(&mut writer, format!("- {}", user.username), true).await;
                                                }

                                            }
                                        }
                                    }
                                    UserCommandType::Invalid => {
                                        client.send_message(&mut writer, "Invalid Input!".to_string(), true).await;
                                    }
                                }

                                line.clear();
                            }

                            result = rx.recv() => {

                                let (message, other_addr, channel_index) = result.unwrap();

                                if addr != other_addr && channel_index == client.channel_index{
                                    writer.write_all(message.as_bytes()).await.unwrap();
                                }
                            }
                        }
                    }
                }
            }
        });
    }
}
