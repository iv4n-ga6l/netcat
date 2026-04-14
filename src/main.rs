use std::env;
use std::io::{self, BufRead, Write};
use std::net::{TcpListener, TcpStream, UdpSocket};

fn handle_tcp_client(mut stream: TcpStream) -> io::Result<()> {
    let mut buffer = String::new();
    let mut reader = io::BufReader::new(&stream);

    while reader.read_line(&mut buffer)? > 0 {
        print!("{}", buffer);
        stream.write_all(buffer.as_bytes())?;
        buffer.clear();
    }

    Ok(())
}

fn handle_udp_server(socket: UdpSocket) -> io::Result<()> {
    let mut buffer = [0; 1024];

    loop {
        let (size, src) = socket.recv_from(&mut buffer)?; // Receive data from a client
        let received_data = &buffer[..size];

        print!("{}", String::from_utf8_lossy(received_data)); // Print received data

        socket.send_to(received_data, src)?; // Echo data back to the client
    }
}

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 5 || args[1] != "-l" || args[2] != "-p" {
        eprintln!("Usage: ccnc -l -p <port> [-u]");
        std::process::exit(1);
    }

    let port = &args[3];
    let address = format!("0.0.0.0:{}", port);

    if args.len() > 4 && args[4] == "-u" {
        // UDP server mode
        let socket = UdpSocket::bind(&address)?;
        println!("Listening on {} (UDP)...", address);
        handle_udp_server(socket)?;
    } else {
        // TCP server mode
        let listener = TcpListener::bind(&address)?;
        println!("Listening on {} (TCP)...", address);

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    if let Err(e) = handle_tcp_client(stream) {
                        eprintln!("Error handling client: {}", e);
                    }
                }
                Err(e) => eprintln!("Connection failed: {}", e),
            }
        }
    }

    Ok(())
}