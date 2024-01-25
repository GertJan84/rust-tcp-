use std::fmt::Display;
use std::net::SocketAddr;

use tokio::io::AsyncWriteExt;
pub enum Protocol {
    Connect,
    Join,
    Chat,
}

pub enum Commands {
    Invalid,
    Disconnect,
    Users,
    Group,
    Exit,
}

impl Commands {
    pub fn get_command(mut command: String) -> Self {
        if command.contains("\n") {
            command.pop();
        }
        match command.to_lowercase().as_str() {
            "/exit" | "/quit" => Commands::Exit,
            "/disconnect" => Commands::Disconnect,
            "/users" => Commands::Users,
            "/groups" => Commands::Group,
            _ => Commands::Invalid,
        }
    }
}

impl Clone for Protocol {
    fn clone(&self) -> Self {
        match self {
            Protocol::Connect => Protocol::Connect,
            Protocol::Join => Protocol::Join,
            Protocol::Chat => Protocol::Chat,
        }
    }
}

impl Display for Protocol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Protocol::Join => write!(f, "Joined"),
            Protocol::Connect => write!(f, "Connect"),
            Protocol::Chat => write!(f, "Chat"),
        }
    }
}

pub struct Client {
    pub username: String,
    pub address: SocketAddr,
    pub protocol: Protocol,
    pub channel_index: Option<usize>,
}

impl Client {
    pub fn new(addr: SocketAddr) -> Self {
        Client {
            username: String::from(""),
            address: addr,
            protocol: Protocol::Connect,
            channel_index: None,
        }
    }

    pub async fn send_message(
        &self,
        writer: &mut tokio::net::tcp::WriteHalf<'_>,
        message: String,
        enter: bool,
    ) {
        let mut message = message;
        if enter {
            message.push('\n');
        }
        writer.write_all(message.as_bytes()).await.unwrap();
    }

    pub fn handle_input(message: String) -> UserCommandType {
        if message.len() == 0 {
            return UserCommandType::Invalid;
        } else if message.chars().nth(0).unwrap() == '/' {
            return UserCommandType::Command;
        } else {
            return UserCommandType::Message;
        }
    }
}

impl Clone for Client {
    fn clone(&self) -> Self {
        Client {
            username: self.username.clone(),
            address: self.address,
            protocol: self.protocol.clone(),
            channel_index: self.channel_index,
        }
    }
}

impl Display for Client {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}, {}", self.username, self.protocol,)
    }
}

pub struct Channel {
    pub name: String,
    pub users: Vec<Client>,
}

impl Channel {
    pub fn new(name: String) -> Self {
        Channel {
            name,
            users: Vec::new(),
        }
    }
}

impl Display for Channel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}, {}", self.name, self.users.len())
    }
}

pub enum UserCommandType {
    Message,
    Command,
    Invalid,
}
