use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use tempfile::NamedTempFile;

type CommandsMap = HashMap<String, String>;

#[derive(Serialize, Deserialize, Debug)]
struct CommandStore {
    commands: CommandsMap,
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
            fs::create_dir_all(parent).context("Failed to create directory for commands file")?;
        }

        let file = File::create(path).context("Failed to create commands file")?;
        serde_json::to_writer_pretty(file, self).context("Failed to write commands to file")?;
        Ok(())
    }
}

#[derive(Parser)]
#[command(name = "keepc")]
#[command(about = "Keep and manage useful commands", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    // Add a new command
    #[command(about = "Add a new command")]
    New {
        #[clap(required = false)]
        command: Option<String>,
        #[clap(required = false)]
        description: Option<String>,
    },
    // List all commands
    #[command(about = "List all saved commands")]
    List,
    // Search for a command
    #[command(about = "Search for commands matching a pattern")]
    Grep {
        pattern: String,
    },
    // Delete a command
    #[command(about = "Delete a saved command")]
    Rm {
        command: String,
    },
    // Edit commands in a text editor
    #[command(about = "Edit commands in a text editor")]
    Edit,
    // Execute a saved command
    #[command(about = "Execute a saved command")]
    Run {
        command: String,
    },
}


fn get_commands_file() -> Result<PathBuf> {
    let mut path = dirs::config_dir().context("Could not determine home directory")?;
    path.push("keepc");
    Ok(path.join("commands.json"))
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
            let stdin = io::stdin();
            let mut line = String::new();
            stdin.lock().read_line(&mut line)?;
            line.trim().to_string()
        }
    };

    if command.is_empty() {
        return Err(anyhow::anyhow!("Command cannot be empty"));
    }

    // Get description from user if not provided
    let description = match description {
        Some(desc) => desc,
        None => {
            print!("Enter description (optional): ");
            io::stdout().flush()?;
            let stdin = io::stdin();
            let mut line = String::new();
            stdin.lock().read_line(&mut line)?;
            line.trim().to_string()
        }
    };

    store.commands.insert(command.clone(), description);
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
        println!("$ \x1b[34m{}\x1b[0m: {}", cmd, desc);
    }
    Ok(())
}

fn search_commands(pattern: String) -> Result<()> {
    let path = get_commands_file()?;
    let store = CommandStore::load(&path)?;

    if store.commands.is_empty() {
        println!("No commands saved.");
        return Ok(());
    }

    let pattern = pattern.to_lowercase();
    let mut found = false;

    for (cmd, desc) in &store.commands {
        if cmd.to_lowercase().contains(&pattern) || desc.to_lowercase().contains(&pattern) {
            println!("$ \x1b[34m{}\x1b[0m: {}", cmd, desc);
            found = true;
        }
    }

    if !found {
        println!("No commands found matching '{}'", pattern);
    }
    Ok(())
}

fn delete_command(command: String) -> Result<()> {
    use std::io::{self, BufRead};

    let path = get_commands_file()?;
    let mut store = CommandStore::load(&path)?;

    // Find all commands that match the pattern
    let pattern = command.to_lowercase();
    let matching_commands: Vec<String> = store.commands.keys()
        .filter(|cmd| cmd.to_lowercase().contains(&pattern))
        .cloned()
        .collect();
    match matching_commands.len() {
        0 => {
            println!("No commands found matching '{}'", command);
        },
        _ => {
            // Multiple matches, ask user which one to delete
            println!("Found {} matching commands:", matching_commands.len());
            for (i, cmd) in matching_commands.iter().enumerate() {
                println!("[{}] \x1b[34m{}\x1b[0m: {}", 
                         i + 1, 
                         cmd, 
                         store.commands.get(cmd).unwrap_or(&String::new()));
            }

            print!("Enter a number to delete: ");
            io::stdout().flush()?;

            let stdin = io::stdin();
            let mut line = String::new();
            stdin.lock().read_line(&mut line)?;

            if let Ok(choice) = line.trim().parse::<usize>() {
                if choice <= matching_commands.len() {
                    let cmd_to_delete = &matching_commands[choice - 1];
                    store.commands.remove(cmd_to_delete);
                    store.save(&path)?;
                    println!("Deleted command: {}", cmd_to_delete);
                }
            };
        }
    }
    Ok(())
}

fn edit_commands() -> Result<()> {
    let path = get_commands_file()?;
    let mut store = CommandStore::load(&path)?;

    // Create a temporary file
    let mut temp_file = NamedTempFile::new().context("Failed to create temporary file")?;

    // Write commands to the temporary file
    for (cmd, desc) in &store.commands {
        writeln!(temp_file, "{}:::{}", cmd, desc).context("Failed to write to temporary file")?;
    }

    // Get the path to the temporary file
    let temp_path = temp_file.path().to_owned();

    // Close the file to ensure all data is written
    temp_file.flush().context("Failed to flush temporary file")?;

    // Determine which editor to use
    let editor = std::env::var("EDITOR").unwrap_or_else(|_| "nano".to_string());

    // Open the editor
    let status = Command::new(&editor)
    .arg(&temp_path)
    .status()
    .context(format!("Failed to open editor: {}", editor))?;

    if !status.success() {
        return Err(anyhow::anyhow!("Editor exited with non-zero status"));
    }

    // Read the edited file
    let mut content = String::new();
    File::open(&temp_path)
    .context("Failed to open temporary file after editing")?
    .read_to_string(&mut content)
    .context("Failed to read temporary file after editing")?;

    // Parse the edited content
    let mut new_commands = HashMap::new();
    for line in content.lines() {
        if let Some((cmd, desc)) = line.split_once(":::") {
            new_commands.insert(cmd.trim().to_string(), desc.trim().to_string());
        }
    }

    // Update and save the commands
    store.commands = new_commands;
    store.save(&path)?;

    println!("Commands updated.");
    Ok(())
}

fn execute_command(command: String) -> Result<()> {
    let path = get_commands_file()?;
    let store = CommandStore::load(&path)?;

    if let Some(_cmd) = store.commands.get(&command) {
        println!("Executing: {}", command);

        // Determine the shell to use
        let shell = if cfg!(target_os = "windows") {
            "cmd"
        } else {
            "sh"
        };

        let shell_arg = if cfg!(target_os = "windows") {
            "/C"
        } else {
            "-c"
        };

        let status = Command::new(shell)
        .arg(shell_arg)
        .arg(&command)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .context(format!("Failed to execute command: {}", command))?;

        if !status.success() {
            return Err(anyhow::anyhow!("Command exited with non-zero status"));
        }
    } else {
        println!("Command not found: {}", command);
    }
    Ok(())
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    
    match cli.command {
        Some(Commands::New { command, description }) => new_command(command, description),
        Some(Commands::List) => list_commands(),
        Some(Commands::Grep { pattern }) => search_commands(pattern),
        Some(Commands::Rm { command }) => delete_command(command),
        Some(Commands::Edit) => edit_commands(),
        Some(Commands::Run { command }) => execute_command(command),
        None => {
            Cli::parse_from(["keepc", "--help"]);
            Ok(())
        }
    }
}