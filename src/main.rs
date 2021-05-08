mod client;
mod message;
mod ui;

use crate::ui::*;
use anyhow::Context;
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
                .takes_value(true)
                .required(true),
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

    // Extract config
    let name = matches.value_of("name").unwrap().to_string();
    let mut channel = matches
        .value_of("channel")
        .unwrap_or("Channel #1")
        .to_string();
    let server = matches
        .value_of("server")
        .unwrap_or("localhost")
        .to_string();

    // Run the main program.
    // If the user changes channel, we restart the whole program.
    while let Some(c) =
        termui(name.clone(), channel.clone(), server.clone()).context("Failed to run UI")?
    {
        channel = c;
    }

    Ok(())
}
