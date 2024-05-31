# Caesar-Transfer
![caesar-gui-send-linux](https://github.com/PatrykHegenberg/caesar-transfer/assets/112555272/8e8bc3a9-cf2d-4a46-8280-fe88304e0a84)

This program provides a simple end-to_end encrypted filesharing system.
Either the cli version or the gui version can be used for this.
## Prerequisites
Rust toolchain installed
## Installation
1. Clone the repository:
  ```bash
  git clone https://github.com/PatrykHegenberg/caesar-transfer.git
  ```
2. Change to the project directory:
  ```bash
  cd caesar-transfer
  ```
3. Build the program:
```bash
cargo build --bin caesar --release
```

## Usage

### cli

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
By default, the server listens on 0.0.0.0:8000.

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
### GUI
To use the Gui version of Ceasar-Transfer, you can download the version that suits you under Releases. Currently supported operating systems are Windows, Linux and Android. 
#### Desktop 
Copy the folder contained in the zip/tar file to a folder of your choice and add the path to it to your PATH variable.

Start the application and configure your relay server in the settings.

#### Android
As the Android version is currently in beta status, the APK must also be downloaded from the release page.
Open it with your smartphone's file manager and install it.

Start the application and configure your relay server in the settings.
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
./target/release/caesar -r ws://0.0.0.0:8000 send
```

Open a receive window in another terminal:
```bash
./target/release/caesar -r ws://0.0.0.0:8000 receive
```

Now you can test the functionality of the relay server using the cli version.
