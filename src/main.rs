use std::env;
use std::io::{self, BufRead, Write};
use std::net::{TcpListener, TcpStream};

fn handle_client(mut stream: TcpStream) -> io::Result<()> {
    let mut buffer = String::new();
    let mut reader = io::BufReader::new(&stream);

    while reader.read_line(&mut buffer)? > 0 {
        print!("{}", buffer);
        stream.write_all(buffer.as_bytes())?;
        buffer.clear();
    }

    Ok(())
}

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 4 || args[1] != "-l" || args[2] != "-p" {
        eprintln!("Usage: ccnc -l -p <port>");
        std::process::exit(1);
    }

    let port = &args[3];
    let address = format!("0.0.0.0:{}", port);

    let listener = TcpListener::bind(&address)?;
    println!("Listening on {}...", address);

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                if let Err(e) = handle_client(stream) {
                    eprintln!("Error handling client: {}", e);
                }
            }
            Err(e) => eprintln!("Connection failed: {}", e),
        }
    }

    Ok(())
}