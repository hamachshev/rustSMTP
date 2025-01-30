use anyhow::{Context, Result};
use std::io::Write;
use std::io::{BufRead, BufReader};
use std::net::{TcpListener, TcpStream};

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

    if let Ok((mut writer_socket, addr)) = listener.accept() {
        println!("recieved connection from: {}", addr);
        let reader_socket = writer_socket.try_clone().context("cloning socket")?;
        let mut buffreader = BufReader::new(&reader_socket);
        let mut buffer = String::new();
        hello(&mut writer_socket, &mut buffreader, &mut buffer)?;
    } else {
        println!("connection failed");
    }
    Ok(())
}

fn hello(
    socket: &mut TcpStream,
    buffreader: &mut BufReader<&TcpStream>,
    buffer: &mut String,
) -> Result<()> {
    socket.write_all(b"220 localhost SMTP\r\n")?;
    buffreader.read_line(buffer)?;
    let response = buffer.trim();

    let response: Command = response.into();

    match response {
        Command::HELO(_) => {}
        Command::Unknown => socket.write_all(b"502 Unrecognized command.\r\n")?,
    }
    Ok(())
}
