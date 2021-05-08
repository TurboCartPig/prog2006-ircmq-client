# Rust client for IRCMQ by IRCMQ boys

This is a client for IRCMQ, a clone of IRC, based on ZeroMQ, built in Rust.

## Features

- Users - What would a chat program be without multiple users?
- Channels - Multiple channels can be created, joined and chatted in.
- Servers - Multiple servers can be run and you can connect to one of them at a time.
- A terminal UI based on tui-rs.
- JSON messages over ZeroMQ sockets makes for a robust and extensible core.

## Instructions

Build:
```bash
cargo build
```

Run with arguments (all arguments are optional, see `--help` for more):
```bash
cargo run -- --name Sebern --channel Rust --server localhost
```

## A word on compatibility

As this is a somewhat advanced terminal user interface, it has some requirements to the terminal running it. We have tested the following terminals for compatibility.

* cmd - Compatible
* Windows Terminal - Compatible
* Windows PowerShell (builtin) - Not compatible
* Other windows terminals - Not tested
* GNOME Terminal - Compatible
* Alacritty - Compatible
* Kitty - Compatible
* Other linux terminals - Not tested (you should use alacritty anyway)
* MacOS terminals - Not tested
