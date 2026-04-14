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

fn scan_ports(host: &str, ports: Vec<u16>, verbose: bool) {
    for port in ports {
        let address = format!("{}:{}", host, port);
        match TcpStream::connect(&address) {
            Ok(_) => {
                println!("Connection to {} port {} [tcp] succeeded!", host, port);
            }
            Err(_) => {
                if verbose {
                    eprintln!("Connection to {} port {} [tcp] failed!", host, port);
                }
            }
        }
    }
}

fn parse_ports(port_arg: &str) -> Vec<u16> {
    if let Some(range_sep) = port_arg.find('-') {
        let start = port_arg[..range_sep].parse::<u16>().unwrap_or(0);
        let end = port_arg[range_sep + 1..].parse::<u16>().unwrap_or(0);
        (start..=end).collect()
    } else {
        vec![port_arg.parse::<u16>().unwrap_or(0)]
    }
}

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: ccnc -l -p <port> [-u] | ccnc -z <host> <port-range> [-v]");
        std::process::exit(1);
    }

    if args[1] == "-z" {
        if args.len() < 4 {
            eprintln!("Usage: ccnc -z <host> <port-range> [-v]");
            std::process::exit(1);
        }

        let host = &args[2];
        let ports = parse_ports(&args[3]);
        let verbose = args.len() > 4 && args[4] == "-v";

        scan_ports(host, ports, verbose);
    } else if args[1] == "-l" {
        if args.len() < 5 || args[2] != "-p" {
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
    } else {
        eprintln!("Usage: ccnc -l -p <port> [-u] | ccnc -z <host> <port-range> [-v]");
        std::process::exit(1);
    }

    Ok(())
}