## What is keepc?
Keep Command or Keepc is a simple, open-source, meta cli program designed to keep important commands that are difficult to remember. Keepc is coded in Rust.

## How to install
Download the [keepc binary](https://github.com/nickcat1/keepc/releases) and move it to /home/$USER/.local/bin/keepc

## Keepc Commands
| Commands: | Descriptions: |
| --------- | ------------- |
| New | Add a new command. |
| List | List all saved commands. |
| Grep | Search for commands matching a pattern. Note: "keepc pattern" will also search saved commands. |
| Remove | Delete a saved command. |
| Edit | Edit commands in a text editor. |
| Run | Execute a saved command. |
| Help | Print the list of Keepc commands or the help of the given subcommands. |

<details>
  <summary>Aliases for Keepc commands</summary>

> `New`: Add.
>
> `List`: ls.
>
> `Grep`: Find, Search.
>
> `Remove`: rm, Delete.
>
> `Run`: Execute.
</details>

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

## Credits and Mentions
[The Rust Programming Language](https://www.rust-lang.org).

Inspired by [keep](https://github.com/OrkoHunter/keep), coded in [Python](https://www.python.org).

Another tool for saving commands, coded in [GO](https://go.dev), [pet](https://github.com/knqyf263/pet).