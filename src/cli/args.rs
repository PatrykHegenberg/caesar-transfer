use crate::receiver;
use crate::relay;
use crate::sender;
use clap::{Parser, Subcommand};
use std::env;
use tracing::debug;

/// This struct defines the CLI arguments and subcommands for the caesar command line application.
///
/// The #[derive(Parser, Debug)] macro generates code that:
///  - parses the command line arguments using the clap library
///  - provides a Debug implementation for the struct
///
/// The #[command(version, about, long_about = None)] macro generates code that:
///  - defines the version and about strings for the application
///  - specifies that there is no long about help text
///
/// The #[command(subcommand)] macro generates code that:
///  - defines a subcommand for the caesar command line application.
///    Subcommands are used to break up a large number of options into
///    smaller, more manageable groups.
///
/// The #[command] macro is used to annotate the `command` field of the struct.
/// The `command` field is an Option<Commands> type, which means that the
/// subcommand is optional.
/// If the subcommand is not provided, the program will exit with a status code
/// of 0 and without printing any output.
///
/// The Commands enum defines the possible subcommands for the caesar command
/// line application.
/// See the Commands enum definition for more information about the available
/// subcommands.
#[derive(Parser, Debug)]
#[command(version = env!("CARGO_PKG_VERSION"), about = "Send and receive files securely")]
#[command(long_about = None)]
pub struct Args {
    /// The subcommand for the caesar command line application.
    /// Subcommands are used to break up a large number of options into smaller,
    /// more manageable groups.
    /// If no subcommand is provided, the program will exit with a status code
    /// of 0 and without printing any output.
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Send files to the receiver or relay server
    Send {
        /// Address of the relay server. Accepted formats are: 127.0.0.1:8080, [::1]:8080, example.com
        #[arg(short, long)]
        relay: Option<String>,
        /// Path to file(s)
        #[arg(value_name = "FILES")]
        files: Vec<String>,
    },
    /// Receives Files from the sender with the matching password
    Receive {
        /// Address of the relay server. Accepted formats are: 127.0.0.1:8080, [::1]:8080, example.com
        #[arg(short, long)]
        relay: Option<String>,

        /// Overwrite existing Files
        #[arg(short, long)]
        overwrite: bool,

        /// Name of Transfer to download files
        #[arg(value_name = "Transfer_Name")]
        name: String,
    },
    /// Start a relay server
    Serve {
        /// Port to run the relay server on
        #[arg(short, long)]
        port: Option<i32>,
        /// The Listen address to run the relay server on
        #[arg(short, long)]
        listen_address: Option<String>,
    },
}

impl Default for Args {
    // This function is called by the Default trait when no value is
    // provided for a field of type Args. It returns an instance of
    // Args that has been created by calling the new() function.
    //
    // The Default trait is used by various parts of the program to
    // provide a sensible default value for a field when no value is
    // provided. For example, the clap crate uses the Default trait when
    // parsing command line arguments to provide a default value for
    // a field.
    //
    // The new() function is a constructor function for Args that
    // creates an instance of Args with default field values.
    fn default() -> Self {
        Self::new()
    }
}

impl Args {
    /// Creates a new instance of Args by parsing command line arguments
    ///
    /// This function is a constructor for Args. It uses the clap crate to parse
    /// command line arguments and creates an instance of Args with the values
    /// provided by the user.
    ///
    /// The clap crate is a command line argument parser that is well tested and
    /// widely used. It provides a simple way to define command line
    /// arguments and generate helpful documentation for the user.
    ///
    /// The `parse()` function is used to parse the command line arguments and
    /// return an instance of Args.
    pub fn new() -> Self {
        Self::parse()
    }

    /// Runs the command specified by the user
    ///
    /// This function is called after the command line arguments have been
    /// parsed. It matches on the `command` field of the Args struct to determine
    /// what command the user wants to run.
    ///
    /// The match statement checks the value of `command` and calls the
    /// appropriate function to run the command. The functions that are called
    /// are located in other modules of the program.
    ///
    /// The `run()` function is called by the `main()` function of the program.
    /// The program's entry point is the `main()` function, which parses the
    /// command line arguments and then calls `run()` on the resulting Args
    /// instance.
    ///
    /// The `run()` function returns a Result. The error type is `Box<dyn
    /// std::error::Error + Send + Sync>`. This means that the error type is a
    /// trait object that represents an error that can be sent across threads
    /// and sent over a network connection. The `Send` and `Sync` traits are part
    /// of the standard library and are used to indicate that the error type can
    /// be sent across threads and sent over a network connection.
    ///
    /// The `run()` function does not return anything if the command is `None`.
    /// This is because `command` is an `Option<Commands>`. If the user does
    /// not specify a command, then `command` is `None`. In this case, there is
    /// nothing to run, so `run()` returns early with no error.
    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        debug!("args: {:#?}", self);
        match &self.command {
            // If the user wants to send files, call `start_sender()` in the
            // `sender` module with the list of files that the user wants to
            // send.
            Some(Commands::Send { relay, files }) => {
                sender::start_sender(
                    relay.as_deref().unwrap_or(
                        env::var("APP_ORIGIN")
                            .unwrap_or("wss://caesar-transfer-iu.shuttleapp.rs/ws".to_string())
                            .as_str(),
                    ),
                    files,
                )
                .await;
            }
            // If the user wants to receive files, call `start_receiver()` in the
            // `receiver` module with the name of the transfer that the user
            // wants to download.
            Some(Commands::Receive {
                relay,
                overwrite: _,
                name,
            }) => {
                println!("Receive for {name:?}");
                receiver::start_receiver(
                    relay.as_deref().unwrap_or(
                        env::var("APP_ORIGIN")
                            .unwrap_or("ws://0.0.0.0:8000/ws".to_string())
                            .as_str(),
                    ),
                    name,
                )
                .await;
            }
            // If the user wants to start a relay server, call `start_ws()` in the
            // `relay` module with the port and listen address that the user
            // specified.
            Some(Commands::Serve {
                port,
                listen_address,
            }) => {
                println!("Serve with address '{listen_address:?}' and '{port:?}'");
                relay::server::start_ws(port.as_ref(), listen_address.as_ref()).await;
            }
            // If the user does not specify a command, return early with no error.
            // This is because `command` is an `Option<Commands>`. If the user does
            // not specify a command, then `command` is `None`.
            None => {}
        }
        Ok(())
    }
}
