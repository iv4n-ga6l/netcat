use std::env;
use std::io::{self, BufRead, Write};
use std::net::{TcpListener, TcpStream, UdpSocket};
use std::process::{Command, Stdio};
use std::thread;

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

fn handle_tcp_client_with_process(mut stream: TcpStream, command: &str) -> io::Result<()> {
    let mut child = Command::new(command)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    let mut child_stdin = child.stdin.take().expect("Failed to open child stdin");
    let mut child_stdout = child.stdout.take().expect("Failed to open child stdout");

    let stream_clone = stream.try_clone()?;

    // Thread to read from client and write to the process's stdin
    let input_thread = thread::spawn(move || {
        let mut reader = io::BufReader::new(stream_clone);
        let mut buffer = String::new();

        while reader.read_line(&mut buffer).unwrap_or(0) > 0 {
            if let Err(e) = child_stdin.write_all(buffer.as_bytes()) {
                eprintln!("Failed to write to child stdin: {}", e);
                break;
            }
            buffer.clear();
        }
    });

    // Thread to read from the process's stdout and write to the client
    let output_thread = thread::spawn(move || {
        let mut buffer = [0; 1024];
        loop {
            match child_stdout.read(&mut buffer) {
                Ok(0) => break, // EOF
                Ok(n) => {
                    if let Err(e) = stream.write_all(&buffer[..n]) {
                        eprintln!("Failed to write to client: {}", e);
                        break;
                    }
                }
                Err(e) => {
                    eprintln!("Failed to read from child stdout: {}", e);
                    break;
                }
            }
        }
    });

    input_thread.join().expect("Input thread panicked");
    output_thread.join().expect("Output thread panicked");

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
        eprintln!("Usage: ccnc -l -p <port> [-u] [-e <command>] | ccnc -z <host> <port-range> [-v]");
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
            eprintln!("Usage: ccnc -l -p <port> [-u] [-e <command>]");
            std::process::exit(1);
        }

        let port = &args[3];
        let address = format!("0.0.0.0:{}", port);

        if args.len() > 4 && args[4] == "-u" {
            // UDP server mode
            let socket = UdpSocket::bind(&address)?;
            println!("Listening on {} (UDP)...", address);
            handle_udp_server(socket)?;
        } else if args.len() > 5 && args[4] == "-e" {
            // TCP server mode with process execution
            let command = &args[5];
            let listener = TcpListener::bind(&address)?;
            println!("Listening on {} (TCP) and executing '{}'...", address, command);

            for stream in listener.incoming() {
                match stream {
                    Ok(stream) => {
                        thread::spawn(move || {
                            if let Err(e) = handle_tcp_client_with_process(stream, command) {
                                eprintln!("Error handling client with process: {}", e);
                            }
                        });
                    }
                    Err(e) => {
                        eprintln!("Connection failed: {}", e);
                    }
                }
            }
        } else {
            // TCP server mode without process execution
            let listener = TcpListener::bind(&address)?;
            println!("Listening on {} (TCP)...", address);

            for stream in listener.incoming() {
                match stream {
                    Ok(stream) => {
                        thread::spawn(move || {
                            if let Err(e) = handle_tcp_client(stream) {
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
    } else {
        eprintln!("Usage: ccnc -l -p <port> [-u] [-e <command>] | ccnc -z <host> <port-range> [-v]");
        std::process::exit(1);
    }

    Ok(())
}
