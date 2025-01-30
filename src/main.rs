use anyhow::Result;
use std::fmt::{Debug, Display};
use std::io::Write;
use std::io::{BufRead, BufReader};
use std::net::TcpListener;

#[derive(Debug)]
enum Command {
    HELO(String),
}

impl<'a> TryFrom<&'a str> for Command {
    type Error = CommandParseError;

    fn try_from(value: &'a str) -> std::result::Result<Self, Self::Error> {
        match value {
            s if s.starts_with("HELO") => Ok(Command::HELO(s[4..].trim().to_string())),
            _ => Err(CommandParseError {
                command: value.to_string(), // need this because the error message must be static
                                            // because it is escaping the place of the error, ie the main fn is returning
                                            // Result
            }),
        }
    }
}
fn main() -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:2525")?;

    if let Ok((mut socket, addr)) = listener.accept() {
        println!("recieved connection from: {}", addr);
        socket.write_all(b"220 localhost SMTP\r\n")?;
        let mut buffreader = BufReader::new(socket);
        let mut response = String::new();
        buffreader.read_line(&mut response)?;
        let response = response.trim();

        let response: Command = response.try_into()?;
        println!("{:?}", response);
    } else {
        println!("connection failed");
    }
    Ok(())
}

#[derive(Debug)]
struct CommandParseError {
    command: String,
}

impl Display for CommandParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Invalid Command: {}", self.command)
    }
}

impl std::error::Error for CommandParseError {}
