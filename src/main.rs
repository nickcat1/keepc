use anyhow::{Context, Result};
use clap::{Parser, Subcommand, CommandFactory};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use tempfile::NamedTempFile;
use colored::Colorize;

#[derive(Serialize, Deserialize, Debug)]
struct CommandStore {
    commands: HashMap<String, String>,
}

impl CommandStore {
    fn new() -> Self {
        Self {
            commands: HashMap::new(),
        }
    }

    fn load(path: &PathBuf) -> Result<Self> {
        if !path.exists() {
            return Ok(Self::new());
        }

        let file = File::open(path).context("Failed to open commands file")?;
        let store: CommandStore =
        serde_json::from_reader(file).context("Failed to parse commands file")?;
        Ok(store)
    }

    fn save(&self, path: &PathBuf) -> Result<()> {
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).context("Failed to create directory")?;
        }

        let file = File::create(path).context("Failed to create commands file")?;
        serde_json::to_writer_pretty(file, self).context("Failed to write commands")?;
        Ok(())
    }
}

#[derive(Parser)]
#[command(name = "keepc", about = "Keep and manage useful commands")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    // Add a new command
    #[command(about = "Add a new command")]
    New {
        command: Option<String>,
        description: Option<String>,
    },
    #[command(hide = true)]
    Add {
        command: Option<String>,
        description: Option<String>,
    },
    // List all commands
    #[command(about = "List all saved commands")]
    List,
    #[command(hide = true)]
    Ls,
    // Search for a command
    #[command(about = "Search for commands matching a pattern")]
    Grep { pattern: String },
    #[command(hide = true)]
    Find { pattern: String },
    #[command(hide = true)]
    Search { pattern: String },
    // Delete a command
    #[command(about = "Delete a saved command")]
    Remove { pattern: String },
    #[command(hide = true)]
    Rm { pattern: String },
    #[command(hide = true)]
    Delete { pattern: String },
    // Edit commands in a text editor
    #[command(about = "Edit commands in a text editor")]
    Edit,
    // Execute a saved command
    #[command(about = "Execute a saved command")]
    Run { pattern: String },
    #[command(hide = true)]
    Execute { pattern: String },
}

fn get_commands_file() -> Result<PathBuf> {
    let mut path = dirs::config_dir().context("Could not determine config directory")?;
    path.push("keepc");
    Ok(path.join("commands.json"))
}

// Find all commands that match the pattern. Used in List, search and delete commands.
fn search_logic(pattern: String, store: &CommandStore) -> Vec<String> {
    let keywords: Vec<&str> = pattern.split_whitespace().collect();
    let mut matching_commands = Vec::new();

    for (cmd, desc) in &store.commands {
        let matched_keywords = keywords.iter()
        .filter(|keyword| {
            cmd.to_lowercase().contains(&keyword.to_lowercase())
            || desc.to_lowercase().contains(&keyword.to_lowercase())
        }).count();
        if matched_keywords == keywords.len() {
            matching_commands.push(cmd.clone());
        }
    }
    matching_commands
}

fn new_command(command: Option<String>, description: Option<String>) -> Result<()> {
    use std::io::{self, BufRead};
    let path = get_commands_file()?;
    let mut store = CommandStore::load(&path)?;

    // Get command from user
    let command = match command {
        Some(cmd) => cmd,
        None => {
            print!("Enter command: ");
            io::stdout().flush()?;
            let mut line = String::new();
            io::stdin().lock().read_line(&mut line)?;
            line.trim().to_string()
        }
    };
    if command.is_empty() {
        return Err(anyhow::anyhow!("Command cannot be empty"));
    }

    // Get description from user if provided
    let description = match description {
        Some(desc) => desc,
        None => {
            print!("Enter description (optional): ");
            io::stdout().flush()?;
            let mut line = String::new();
            io::stdin().lock().read_line(&mut line)?;
            line.trim().to_string()
        }
    };
    store.commands.insert(command, description);
    store.save(&path)?;
    Ok(())
}

fn list_commands() -> Result<()> {
    let path = get_commands_file()?;
    let store = CommandStore::load(&path)?;

    if store.commands.is_empty() {
        println!("No commands saved.");
        return Ok(());
    }

    for (cmd, desc) in &store.commands {
        println!("$ {}{}", cmd.bright_green(), (": ".to_owned() + desc).blue());
    };
    Ok(())
}

fn search_commands(pattern: String) -> Result<()> {
    let path = get_commands_file()?;
    let store = CommandStore::load(&path)?;

    let matching_commands = search_logic(pattern.clone(), &store);
    if matching_commands.is_empty() {
        println!("No commands found matching '{}'", pattern);
    } else {
        for cmd in matching_commands {
            println!("$ {}{}", cmd.bright_green(), 
            (": ".to_owned() + store.commands.get(&cmd).unwrap_or(&String::new())).blue());
        }
    }
    Ok(())
}

fn delete_command(pattern: String) -> Result<()> {
    use std::io::{self, BufRead};
    let path = get_commands_file()?;
    let mut store = CommandStore::load(&path)?;

    let matching_commands = search_logic(pattern.clone(), &store);
    if matching_commands.is_empty() {
        println!("No commands found matching '{}'", pattern);
    } else {
        println!("Found {} matching commands:", matching_commands.len());
        for (i, cmd) in matching_commands.iter().enumerate() {
            println!("[{}] {}{}", 
            i + 1, 
            cmd.bright_green(), 
            (": ".to_owned() + store.commands.get(cmd).unwrap_or(&String::new())).blue());
        };
        print!("Enter a number to delete: ");
        io::stdout().flush()?;

        let mut line = String::new();
        io::stdin().lock().read_line(&mut line)?;

        if let Ok(choice) = line.trim().parse::<usize>() {
            if choice <= matching_commands.len() {
                let cmd_to_delete = &matching_commands[choice - 1];
                store.commands.remove(cmd_to_delete);
                store.save(&path)?;
                println!("Deleted command: {}", cmd_to_delete);
            }
        };
    };
    Ok(())
}

fn edit_commands() -> Result<()> {
    let path = get_commands_file()?;
    let mut store = CommandStore::load(&path)?;

    // Create and write commands a temporary file
    let mut temp_file = NamedTempFile::new().context("Failed to create temporary file")?;
    for (cmd, desc) in &store.commands {
        writeln!(temp_file, "{}:::{}", cmd, desc).context("Failed to write to temp file")?;
    }
    let temp_path = temp_file.path().to_owned();
    temp_file.flush().context("Failed to flush temp file")?;
    let editor = std::env::var("EDITOR").unwrap_or_else(|_| "nano".to_string());
    let status = Command::new(&editor)
    .arg(&temp_path)
    .status()
    .context(format!("Failed to open editor: {}", editor))?;

    if !status.success() {
        return Err(anyhow::anyhow!("Editor exited with non-zero status"));
    }
    let mut content = String::new();
    std::io::Read::read_to_string(
        &mut File::open(&temp_path).context("Failed to open temporary file after editing")?,
        &mut content
    ).context("Failed to read temporary file after editing")?;
    let mut new_commands = HashMap::new();
    for line in content.lines() {
        if let Some((cmd, desc)) = line.split_once(":::") {
            new_commands.insert(cmd.trim().to_string(), desc.trim().to_string());
        }
    }
    store.commands = new_commands;
    store.save(&path)?;

    println!("Commands updated.");
    Ok(())
}

fn execute_command(pattern: String) -> Result<()> {
    use std::io::{self, BufRead};
    let path = get_commands_file()?;
    let store = CommandStore::load(&path)?;

    let matching_commands = search_logic(pattern.clone(), &store);
    if matching_commands.is_empty() {
        println!("No commands found matching '{}'", pattern);
    } else {
        println!("Found {} matching commands:", matching_commands.len());
        for (i, cmd) in matching_commands.iter().enumerate() {
            println!("[{}] {}{}", 
            i + 1, 
            cmd.bright_green(), 
            (": ".to_owned() + store.commands.get(cmd).unwrap_or(&String::new())).blue());
        };
        print!("Enter a number to execute: ");
        io::stdout().flush()?;

        let mut line = String::new();
        io::stdin().lock().read_line(&mut line)?;

        if let Ok(choice) = line.trim().parse::<usize>() {
            if choice <= matching_commands.len() {
                let cmd_to_execute = &matching_commands[choice - 1];
                println!("Executing: {}", cmd_to_execute);
                let (shell, shell_arg) = if cfg!(target_os = "windows") {
                    ("cmd", "/C")
                } else {
                    ("sh", "-c")
                };
                Command::new(shell)
                .arg(shell_arg)
                .arg(&cmd_to_execute)
                .stdin(Stdio::inherit())
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .status()
                .context(format!("Failed to execute: {}", cmd_to_execute))?;
            }
        }
    };
    Ok(())
}

fn main() -> Result<()> {
    let mut commands = Vec::new();
    let cli_command = Cli::command();
    for subcommand in cli_command.get_subcommands() {
        commands.push(subcommand.get_name());
    }
    commands.extend_from_slice(&["help", "--help", "-h"]);
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 { //search saved commands
        if !commands.contains(&args[1].as_str()) {
            let store = CommandStore::load(&get_commands_file()?)?;
            let matching_commands = search_logic((args[1..].join(" ")).clone(), &store);
            if !matching_commands.is_empty() {
                return Ok(for cmd in matching_commands {
                    println!("$ {}{}",
                    cmd.bright_green(), 
                    (": ".to_owned() + store.commands.get(&cmd).unwrap_or(&String::new())).blue());
                });
            }
        }
    }
    match Cli::parse().command {
        Some(Commands::New { command, description })
        | Some(Commands::Add { command, description }) => new_command(command, description),
        Some(Commands::List)
        | Some(Commands::Ls) => list_commands(),
        Some(Commands::Grep { pattern })
        | Some(Commands::Find { pattern })
        | Some(Commands::Search { pattern }) => search_commands(pattern),
        Some(Commands::Remove { pattern })
        | Some(Commands::Rm { pattern })
        | Some(Commands::Delete { pattern }) => delete_command(pattern),
        Some(Commands::Edit) => edit_commands(),
        Some(Commands::Run { pattern })
        | Some(Commands::Execute { pattern }) => execute_command(pattern),
        None => {
            Cli::parse_from(["keepc", "--help"]);
            Ok(())
        }
    }
}