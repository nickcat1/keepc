## What is keepc?
Keep Command or Keepc is a meta cli program designed to keep important commands that are difficult to remember. Keepc is coded in Rust.

## How to install
Download the [keepc binary](https://github.com/nickcat1/keepc/releases) and move it to /home/$USER/.local/bin/keepc

## Keepc Commands
| Commands: | Descriptions: |
| ------- | ----------- |
| New (Add) | Add a new command. |
| List (ls) | List all saved commands. |
| Grep (Find, Search) | Search for commands matching a pattern. |
| Remove (Delete, rm) | Delete a saved command. |
| Edit | Edit commands in a text editor. |
| Run (Execute) | Execute a saved command. |
| Help | Print the list of cammands or the help of the given subcommands. |

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
....

## License
Distributed under the [GPL-3.0 License](LICENSE).

## Credits
[The Rust Programming Language](https://www.rust-lang.org/)

Inspired by [keep](https://github.com/OrkoHunter/keep/).