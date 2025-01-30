use anyhow::{Context, Result};
use std::io::Write;
use std::io::{BufRead, BufReader};
use std::net::{TcpListener, TcpStream};

#[derive(Debug)]
enum Command {
    HELO(String),
    Unknown,
    MailFrom(String),
}

impl From<&str> for Command {
    fn from(value: &str) -> Self {
        match value {
            s if s.starts_with("HELO ") => Command::HELO(s[4..].trim().to_string()),
            s if s.starts_with("MAIL FROM:<") && s.ends_with(">") => {
                Command::MailFrom(s[11..s.len() - 1].trim().to_string())
            }
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
        writer_socket.write_all(b"220 localhost SMTP\r\n")?;
        loop {
            let Ok(_) = hello(&mut writer_socket, &mut buffreader, &mut buffer) else {
                continue;
            };
            let Ok(_) = mail_from(&mut writer_socket, &mut buffreader, &mut buffer) else {
                continue;
            };
        }
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
    buffreader.read_line(buffer)?;
    let response = buffer.trim();

    let response: Command = response.into();

    buffer.clear();

    match response {
        Command::HELO(_) => socket.write_all(b"250 localhost at your service \r\n")?,
        Command::Unknown => {
            socket.write_all(b"502 Unrecognized command.\r\n")?;
            anyhow::bail!("Unrecognized command");
        }
        _ => {
            socket.write_all(b"503 Command out of order. Must do HELO/EHLO first.\r\n")?;
            anyhow::bail!("Command out of order");
        }
    }
    Ok(())
}
fn mail_from(
    socket: &mut TcpStream,
    buffreader: &mut BufReader<&TcpStream>,
    buffer: &mut String,
) -> Result<()> {
    buffreader.read_line(buffer)?;
    let response = buffer.trim();

    let response: Command = response.into();

    match response {
        Command::MailFrom(_) => socket.write_all(b"250 Ok.\r\n")?, // validate email
        Command::Unknown => {
            socket.write_all(b"502 Unrecognized command.\r\n")?;
            anyhow::bail!("Unrecognized command");
        }
        _ => {
            socket.write_all(b"503 Command out of order.\r\n")?;
            anyhow::bail!("Command out of order");
        }
    }
    Ok(())
}
