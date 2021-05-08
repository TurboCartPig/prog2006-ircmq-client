# Rust client for IRCMQ by IRCMQ boys

This is a client for IRCMQ, a clone of IRC, based on ZeroMQ, built in Rust.

## Features

[x] - Users -> Multiple users can enter a channel.

[x] - Channels -> Multiple channels can be created, joined and chatted in.

[x] - Servers -> Multiple servers can run and you can connect to one of them at a time.

[x] - TUI -> A terminal based UI based on TUI-rs.

[x] - ZeroMQ -> JSON messages over ZeroMQ sockets makes for a robust and extensible core.


## Compatibility

As this is a somewhat advanced terminal user interface, it has some requirements to the terminal running it. We have tested the following terminals for compatibility.

[x] - cmd - Compatible

[x] - Windows Terminal - Compatible

[_] - Windows PowerShell (builtin) - Not compatible

[?] - Other windows terminals - Not tested

[x] - GNOME Terminal - Compatible

[x] - Alacritty - Compatible

[x] - Kitty - Compatible

[x] - Other linux terminals - Not tested (you should use alacritty anyway)

[x] - MacOS terminals - works for default terminal

## Instructions

**Clone the repo into a clean directory**

```
git clone https://git.gvk.idi.ntnu.no/course/prog2006/as/denniskr/ircmq-boys/client-rust.git
```

### Installation instructions - Windows

**source:** https://forge.rust-lang.org/infra/other-installation-methods.html

**sourde:** https://chocolatey.org/install

- Download and configure Chocolatey
  - Using administrative shell
  - Copy and run the following command
    ```
      Set-ExecutionPolicy Bypass -Scope Process -Force; [System.Net.ServicePointManager]::SecurityProtocol = [System.Net.ServicePointManager]::SecurityProtocol -bor 3072; iex ((New-Object System.Net.WebClient).DownloadString('https://chocolatey.org/install.ps1'))
    ```
  - Paste it into your shell and press Enter
  - If no errors appear, you are ready to use **Chocolatey**. Test it with `choco` command in the shell.
  
- To install Rust (GNU ABI), run the following command from the command line or from PowerShell:

  ```
  choco install rust
  ```

  **To install Rust with an installer**

- Download the installer from
  ```
  https://static.rust-lang.org/rustup/dist/i686-pc-windows-gnu/rustup-init.exe
  ```

### Installation instructions - Linux/MacOS

source: https://www.rust-lang.org/tools/install

- In the terminal, run

  ```
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  ```

- If the above command did not work, try:

  ```
  curl https://sh.rustup.rs -sSf | sh
  ```

If you want more information about other installation methods, go to:

https://forge.rust-lang.org/infra/other-installation-methods.html

### Run the client

**Inside the project directory**

* Build the program

```
cargo build
```

* Run with arguments (all arguments are optional, see `--help` for more):

```
cargo run -- --name Sebern --channel Rust --server localhost
```

### Documentation

cargo dock is the standard documentation tool for a rust project. To generate the documentation, with cargo installed, you can run:

```
cargo dock
```

You can also open the documentation right away, after generating, with:
```
cargo dock --open
```

