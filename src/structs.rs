use std::fmt::Display;
use std::net::SocketAddr;
pub enum Protocol {
    Connect,
    Join,
    Send,
    Disconnect,
    Users,
}

impl Clone for Protocol {
    fn clone(&self) -> Self {
        match self {
            Protocol::Connect => Protocol::Connect,
            Protocol::Join => Protocol::Join,
            Protocol::Send => Protocol::Send,
            Protocol::Disconnect => Protocol::Disconnect,
            Protocol::Users => Protocol::Users,
        }
    }
}

impl Display for Protocol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Protocol::Join => write!(f, "Joined"),
            Protocol::Connect => write!(f, "Connect"),
            Protocol::Send => write!(f, "Send"),
            Protocol::Disconnect => write!(f, "Disconnect"),
            Protocol::Users => write!(f, "Users"),
        }
    }
}

pub struct Client {
    pub username: String,
    pub address: SocketAddr,
    pub protocol: Protocol,
}

impl Client {
    pub fn new(addr: SocketAddr) -> Self {
        Client {
            username: String::from(""),
            address: addr,
            protocol: Protocol::Connect,
        }
    }
}

impl Clone for Client {
    fn clone(&self) -> Self {
        Client {
            username: self.username.clone(),
            address: self.address,
            protocol: self.protocol.clone(),
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
