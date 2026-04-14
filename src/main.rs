use std::env;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream, UdpSocket};
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

fn hex_dump(data: &[u8]) {
    let mut offset = 0;
    while offset < data.len() {
        let chunk = &data[offset..std::cmp::min(offset + 16, data.len())];
        let hex: Vec<String> = chunk.iter().map(|byte| format!("{:02X}", byte)).collect();
        let ascii: String = chunk
            .iter()
            .map(|&byte| if byte.is_ascii_graphic() { byte as char } else { '.' })
            .collect();

        println!("{:08X}  {:<48}  {}", offset, hex.join(" "), ascii);
        offset += 16;
    }
}

fn handle_tcp_client(mut stream: TcpStream, hex_dump_enabled: bool, file_transfer: Option<String>, timeout: Option<Duration>) -> io::Result<()> {
    if let Some(duration) = timeout {
        stream.set_read_timeout(Some(duration))?;
        stream.set_write_timeout(Some(duration))?;
    }

    if let Some(file_path) = file_transfer {
        let mut file = File::open(file_path)?;
        let mut buffer = [0; 1024];

        loop {
            let bytes_read = file.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }

            stream.write_all(&buffer[..bytes_read])?;

            if hex_dump_enabled {
                println!("Sent {} bytes to the socket", bytes_read);
                hex_dump(&buffer[..bytes_read]);
            }
        }

        return Ok(());
    }

    let mut buffer = String::new();
    let mut reader = io::BufReader::new(&stream);

    while reader.read_line(&mut buffer)? > 0 {
        if hex_dump_enabled {
            println!("Received {} bytes from the socket", buffer.len());
            hex_dump(buffer.as_bytes());
        } else {
            print!("{}", buffer);
        }

        stream.write_all(buffer.as_bytes())?;

        if hex_dump_enabled {
            println!("Sent {} bytes to the socket", buffer.len());
            hex_dump(buffer.as_bytes());
        }

        buffer.clear();
    }

    Ok(())
}

fn handle_tcp_server(listener: TcpListener, hex_dump_enabled: bool, file_transfer: Option<String>, timeout: Option<Duration>) {
    println!("Listening on {}", listener.local_addr().unwrap());

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let hex_dump_enabled = hex_dump_enabled;
                let file_transfer = file_transfer.clone();
                let timeout = timeout;

                thread::spawn(move || {
                    if let Err(e) = handle_tcp_client(stream, hex_dump_enabled, file_transfer, timeout) {
                        eprintln!("Error handling client: {}", e);
                    }
                });
            }
            Err(e) => {
                eprintln!("Connection failed: {}", e);
            }
        }
    }
}

fn handle_tcp_file_receive(mut stream: TcpStream, file_path: String, hex_dump_enabled: bool, timeout: Option<Duration>) -> io::Result<()> {
    if let Some(duration) = timeout {
        stream.set_read_timeout(Some(duration))?;
        stream.set_write_timeout(Some(duration))?;
    }

    let mut file = File::create(file_path)?;
    let mut buffer = [0; 1024];

    loop {
        let bytes_read = stream.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }

        file.write_all(&buffer[..bytes_read])?;

        if hex_dump_enabled {
            println!("Received {} bytes from the socket", bytes_read);
            hex_dump(&buffer[..bytes_read]);
        }
    }

    Ok(())
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut listen_mode = false;
    let mut port = 0;
    let mut hex_dump_enabled = false;
    let mut file_transfer: Option<String> = None;
    let mut timeout: Option<Duration> = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-l" => listen_mode = true,
            "-p" => {
                i += 1;
                port = args[i].parse().expect("Invalid port number");
            }
            "-x" => hex_dump_enabled = true,
            "-f" => {
                i += 1;
                file_transfer = Some(args[i].clone());
            }
            "-t" => {
                i += 1;
                let timeout_secs: u64 = args[i].parse().expect("Invalid timeout value");
                timeout = Some(Duration::new(timeout_secs, 0));
            }
            _ => {}
        }
        i += 1;
    }

    if listen_mode {
        let address = format!("0.0.0.0:{}", port);
        let listener = TcpListener::bind(&address).expect("Failed to bind TCP listener");

        handle_tcp_server(listener, hex_dump_enabled, file_transfer, timeout);
    } else {
        let address = format!("127.0.0.1:{}", port);
        let stream = TcpStream::connect(&address).expect("Failed to connect to server");

        if let Some(file_path) = file_transfer {
            handle_tcp_file_receive(stream, file_path, hex_dump_enabled, timeout).expect("Failed to receive file");
        } else {
            handle_tcp_client(stream, hex_dump_enabled, None, timeout).expect("Failed to handle TCP client");
        }
    }
}