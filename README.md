# ccnc

`ccnc` is a versatile command-line tool inspired by `netcat`, designed for handling TCP/UDP connections, file transfers, and more. It includes additional features such as hex dump functionality and configurable timeouts.

## Features

- **TCP/UDP Support**: Handle both TCP and UDP connections seamlessly.
- **Hex Dump Mode**: Visualize data in a hex dump format for debugging and analysis.
- **File Transfer**: Send or receive files over a network connection.
- **Timeouts**: Configure read/write timeouts for connections.

## Installation

To build and run `ccnc`, you need [Rust](https://www.rust-lang.org/) installed on your system. Clone the repository and use `cargo` to build the project:

```bash
$ git clone https://github.com/iv4n-ga6l/netcat.git
$ cd netcat
$ cargo build --release
```

The compiled binary will be available in the `target/release` directory.

## Usage

### Basic TCP Server

Start a TCP server that listens on a specific port:

```bash
$ ./ccnc -l -p 1234
```

### Basic TCP Client

Connect to a TCP server:

```bash
$ ./ccnc -p 1234 127.0.0.1
```

### Hex Dump Mode

Enable hex dump mode to visualize data in hexadecimal format:

```bash
$ ./ccnc -l -p 1234 --hex-dump
```

### File Transfer

#### Sending a File

Send a file to a TCP client:

```bash
$ ./ccnc -l -p 1234 --file-send /path/to/file.txt
```

#### Receiving a File

Receive a file from a TCP server:

```bash
$ ./ccnc -p 1234 --file-receive /path/to/save/file.txt
```

### Configuring Timeouts

Set a timeout for read/write operations (in seconds):

```bash
$ ./ccnc -l -p 1234 --timeout 10
```

### UDP Mode

Start a UDP server or client by specifying the `--udp` flag:

#### UDP Server

```bash
$ ./ccnc -l -p 1234 --udp
```

#### UDP Client

```bash
$ ./ccnc -p 1234 127.0.0.1 --udp
```

## Command-Line Options

- `-l`: Listen mode (server).
- `-p <port>`: Specify the port to use.
- `--hex-dump`: Enable hex dump mode.
- `--file-send <file>`: Send a file to the client.
- `--file-receive <file>`: Receive a file from the server.
- `--timeout <seconds>`: Set a timeout for read/write operations.
- `--udp`: Use UDP instead of TCP.

## Examples

### Example 1: Simple Chat

Start a TCP server:

```bash
$ ./ccnc -l -p 1234
```

Connect to the server from another terminal:

```bash
$ ./ccnc -p 1234 127.0.0.1
```

Type messages in either terminal to see them echoed back.

### Example 2: File Transfer

Send a file from the server:

```bash
$ ./ccnc -l -p 1234 --file-send /path/to/file.txt
```

Receive the file on the client:

```bash
$ ./ccnc -p 1234 --file-receive /path/to/save/file.txt
```

### Example 3: Hex Dump Debugging

Start a server with hex dump mode enabled:

```bash
$ ./ccnc -l -p 1234 --hex-dump
```

Connect to the server and send data to see it visualized in hex format.

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.

## Contributing

Contributions are welcome! Feel free to open issues or submit pull requests to improve the tool.