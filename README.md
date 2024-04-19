# Caesar
This program provides a simple relay server that can be controlled via the command line.
## Prerequisites
Rust toolchain installed
## Installation
1. Clone the repository:
  ```bash
  git clone https://github.com/your-username/caesar.git
  ```
2. Change to the project directory:
  ```bash
  cd caesar
  ```
3. Build the program:
```bash
cargo build --release
```

## Usage
The program offers the following commands:
`serve`
Starts the relay server.
```bash
./target/release/caesar serve
```

You can optionally specify the listening address and port using flags:
```bash
./target/release/caesar serve -p 8080 -l 192.168.1.100
```
By default, the server listens on 0.0.0.0:1323.

`send`
Sends data through the relay server.
```bash
./target/release/caesar send
```

`receive`
Receives data through the relay server.
```bash
./target/release/caesar receive
```

## Help
For more information about the commands and arguments, use:
```bash
./target/release/caesar --help
```

## Development
To start a test system, please follow these steps:
Start the relay server:
```bash
./target/release/caesar serve
```

Open a send window in another terminal:
```bash
./target/release/caesar send
```

Open a receive window in another terminal:
```bash
./target/release/caesar receive
```

Now you can test the functionality of the relay server.
