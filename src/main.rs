use anyhow::{Context, Result};
use core::str;
use std::io::{BufRead, BufReader};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

#[derive(Debug)]
enum Command {
    HELO(String),
    Unknown,
    MailFrom(String),
    RcptTo(String),
    Data(String),
}

#[derive(Debug)]
struct EmailMessage {
    hello: Command,
    mail_from: Command,
    rcpt_to: Command,
    data: Command,
}

impl From<&str> for Command {
    fn from(value: &str) -> Self {
        match value {
            s if s.starts_with("HELO ") => Command::HELO(s[4..].trim().to_string()),
            s if s.starts_with("MAIL FROM:<") && s.ends_with(">") => {
                Command::MailFrom(s.trim()[11..s.len() - 1].to_string())
            }
            s if s.starts_with("RCPT TO:<") && s.ends_with(">") => {
                Command::RcptTo(s.trim()[9..s.len() - 1].to_string())
            }
            s if s == "DATA" => Command::Data(String::new()),
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
        let message = get_email_message(&mut writer_socket, &mut buffreader, &mut buffer)?;
        println!("{:#?}", message);
    } else {
        println!("connection failed");
    }
    Ok(())
}

fn get_email_message(
    socket: &mut TcpStream,
    buffreader: &mut BufReader<&TcpStream>,
    buffer: &mut String,
) -> Result<EmailMessage> {
    let email_message: EmailMessage;
    loop {
        let Ok(hello) = hello(socket, buffreader, buffer) else {
            continue;
        };
        let Ok(mail_from) = mail_from(socket, buffreader, buffer) else {
            continue;
        };
        let Ok(rcpt_to) = rcpt_to(socket, buffreader, buffer) else {
            continue;
        };
        let Ok(data) = data(socket, buffreader, buffer) else {
            continue;
        };
        email_message = EmailMessage {
            hello,
            mail_from,
            rcpt_to,
            data,
        };
        break;
    }

    Ok(email_message)
}

fn hello(
    socket: &mut TcpStream,
    buffreader: &mut BufReader<&TcpStream>,
    buffer: &mut String,
) -> Result<Command> {
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
    Ok(response)
}
fn mail_from(
    socket: &mut TcpStream,
    buffreader: &mut BufReader<&TcpStream>,
    buffer: &mut String,
) -> Result<Command> {
    buffreader.read_line(buffer)?;
    let response = buffer.trim();

    let response: Command = response.into();
    buffer.clear();

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
    Ok(response)
}
fn rcpt_to(
    socket: &mut TcpStream,
    buffreader: &mut BufReader<&TcpStream>,
    buffer: &mut String,
) -> Result<Command> {
    buffreader.read_line(buffer)?;
    let response = buffer.trim();

    let response: Command = response.into();
    buffer.clear();

    match response {
        Command::RcptTo(_) => socket.write_all(b"250 Ok.\r\n")?, // validate email
        Command::Unknown => {
            socket.write_all(b"502 Unrecognized command.\r\n")?;
            anyhow::bail!("Unrecognized command");
        }
        _ => {
            socket.write_all(b"503 Command out of order.\r\n")?;
            anyhow::bail!("Command out of order");
        }
    }
    Ok(response)
}
fn data(
    socket: &mut TcpStream,
    buffreader: &mut BufReader<&TcpStream>,
    buffer: &mut String,
) -> Result<Command> {
    buffreader.read_line(buffer)?;
    let response = buffer.trim();

    let mut response: Command = response.into();
    buffer.clear();

    match response {
        Command::Data(_) => socket.write_all(b"354 Go ahead\r\n")?, // validate email
        Command::Unknown => {
            socket.write_all(b"502 Unrecognized command.\r\n")?;
            anyhow::bail!("Unrecognized command");
        }
        _ => {
            socket.write_all(b"503 Command out of order.\r\n")?;
            anyhow::bail!("Command out of order");
        }
    }

    let mut buffer2 = Vec::new();
    loop {
        buffreader
            .read_until(0x0d, &mut buffer2)
            .context("reading the data message")?;
        let mut crlf_end_buffer = [0u8; 4];
        buffreader
            .read_exact(&mut crlf_end_buffer)
            .context("reading the crlf end of message")?;
        if let Ok(end_str) = str::from_utf8(&crlf_end_buffer) {
            if end_str == "\n.\r\n" {
                buffer2.pop(); // remove the \r in the end
                break;
            }
        }

        let mut crlf_end_buffer = crlf_end_buffer.to_vec();
        buffer2.append(&mut crlf_end_buffer);
    }

    if let Command::Data(body) = &mut response {
        body.push_str(str::from_utf8(&buffer2).context("body to utf8")?);
    }

    Ok(response)
}
