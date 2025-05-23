## What is keepc?
Keep Command or Keepc is a meta cli program designed to keep important commands that are difficult to remember. Keepc is coded in Rust.

## How to install
Download the [keepc binary](https://github.com/nickcat1/keepc/releases) and move it to /home/$USER/.local/bin/keepc

## Keepc Commands
| Command | Description |
| ------- | ----------- |
| new | Add a new command |
| list | List all saved commands |
| grep | Search for commands matching a pattern |
| rm | Delete a saved command |
| edit | Edit commands in a text editor |
| run | Execute a saved command |
| help | Print the list of cammands or the help of the given subcommands |

## How to Build
`git clone https://github.com/nickcat1/keepc.git`

`cd keepc`

`cargo build`

`cargo run`

The keepc binary will be located at keepc/target/debug/keepc.

## How to Contribute
Create a pull request.

[Issue tracker](https://github.com/nickcat1/keepc/issues).

## TODO
Add other ways to run existing commands. For example, list/ls, new/add, run/execute....

Make search, rm, and run commands search text in any order.

## License
Distributed under the [GPL-3.0 License](LICENSE).

## Credits
[The Rust Programming Language](https://www.rust-lang.org/)

Inspired by [keep](https://github.com/OrkoHunter/keep/).