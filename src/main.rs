use std::env;
use std::io::{self, BufRead, Write};
use std::net::{TcpListener, TcpStream, UdpSocket};
use std::process::{Command, Stdio};
use std::thread;

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

fn handle_tcp_client(mut stream: TcpStream, hex_dump_enabled: bool) -> io::Result<()> {
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

fn handle_udp_server(socket: UdpSocket, hex_dump_enabled: bool) -> io::Result<()> {
    let mut buffer = [0; 1024];

    loop {
        let (size, src) = socket.recv_from(&mut buffer)?; // Receive data from a client
        let received_data = &buffer[..size];

        if hex_dump_enabled {
            println!("Received {} bytes from the socket", size);
            hex_dump(received_data);
        } else {
            print!("{}", String::from_utf8_lossy(received_data));
        }

        socket.send_to(received_data, src)?; // Echo data back to the client

        if hex_dump_enabled {
            println!("Sent {} bytes to the socket", size);
            hex_dump(received_data);
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut listen_mode = false;
    let mut port = 0;
    let mut hex_dump_enabled = false;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-l" => listen_mode = true,
            "-p" => {
                i += 1;
                port = args[i].parse().expect("Invalid port number");
            }
            "-x" => hex_dump_enabled = true,
            _ => {}
        }
        i += 1;
    }

    if listen_mode {
        let address = format!("0.0.0.0:{}", port);
        let listener = TcpListener::bind(&address).expect("Failed to bind TCP listener");

        println!("Listening on {}", address);

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    let hex_dump_enabled = hex_dump_enabled; // Capture flag for thread
                    thread::spawn(move || {
                        if let Err(e) = handle_tcp_client(stream, hex_dump_enabled) {
                            eprintln!("Error handling client: {}", e);
                        }
                    });
                }
                Err(e) => eprintln!("Failed to accept connection: {}", e),
            }
        }
    } else {
        eprintln!("Client mode not implemented in this step.");
    }
}
