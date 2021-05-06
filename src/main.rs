mod client;
mod message;
mod terminalui;

use crate::client::*;
use crate::terminalui::*;
use clap::{App, Arg};

fn main() -> anyhow::Result<()> {
    // Setup cli options
    let matches = App::new("IRCMQ Chat your life away")
        .version("0.1.0")
        .author("IRCMQ Boys")
        .about("The only chat program you will ever need")
        .arg(
            Arg::with_name("name")
                .short("n")
                .long("name")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("channel")
                .short("c")
                .long("channel")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("server")
                .short("s")
                .long("server")
                .takes_value(true),
        )
        .get_matches();

    // Extracted config
    let name = matches.value_of("name").unwrap_or("Sebern").to_string();
    let channel = matches.value_of("channel").unwrap_or("A").to_string();
    let server = matches
        .value_of("server")
        .unwrap_or("localhost")
        .to_string();

    let (sender, server_receiver) = create_sockets(&channel, &server)?;

    termui(name, channel, sender, server_receiver)?;

    Ok(())
}
