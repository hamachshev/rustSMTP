use anyhow::Result;
use std::fmt::{Debug, Display};
use std::io::Write;
use std::io::{BufRead, BufReader};
use std::net::TcpListener;

#[derive(Debug)]
enum Command {
    HELO(String),
    Unknown,
}

impl From<&str> for Command {
    fn from(value: &str) -> Self {
        match value {
            s if s.starts_with("HELO") => Command::HELO(s[4..].trim().to_string()),
            _ => Self::Unknown,
        }
    }
}
fn main() -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:2525")?;

    if let Ok((mut socket, addr)) = listener.accept() {
        println!("recieved connection from: {}", addr);
        socket.write_all(b"220 localhost SMTP\r\n")?;
        let mut buffreader = BufReader::new(&socket);
        let mut response = String::new();
        buffreader.read_line(&mut response)?;
        let response = response.trim();

        let response: Command = response.into();

        match response {
            Command::HELO(_) => {}
            Command::Unknown => socket.write_all(b"502 Unrecognized command.\r\n")?,
        }
        println!("{:?}", response);
    } else {
        println!("connection failed");
    }
    Ok(())
}
