use std::io::Write;
use std::io::{BufRead, BufReader, Result};
use std::net::TcpListener;

fn main() -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:2525")?;

    if let Ok((mut socket, addr)) = listener.accept() {
        println!("recieved connection from: {}", addr);
        socket.write_all(b"220 localhost SMTP")?;
        let mut buffreader = BufReader::new(socket);
        let mut response = String::new();
        buffreader.read_line(&mut response)?;
        let response = response
            .trim()
            .split_once(" ")
            .expect("must send HELO <domain> command");
        println!("{}", response.0);
    } else {
        println!("connection failed");
    }
    Ok(())
}
