use clap::{Parser, Subcommand};
use shae::transform;
use std::fs;
use std::io::{self, Read};
use std::path::PathBuf;
use std::process;

#[derive(Parser, Debug)]
#[command(name = "shae")]
#[command(version = "0.1.0")]
#[command(about = "Shae — Structural Data Transformation & JSON Plumbing Language", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Run a Shae script against a JSON payload
    Run {
        /// Path to the .shae script file
        script: PathBuf,

        /// Path to input JSON file (reads from stdin if omitted)
        input: Option<PathBuf>,

        /// Output compact JSON instead of pretty-printed JSON
        #[arg(short, long)]
        compact: bool,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Run {
            script,
            input,
            compact,
        } => {
            // Read script content
            let script_content = match fs::read_to_string(&script) {
                Ok(content) => content,
                Err(err) => {
                    eprintln!("Error reading script file '{}': {}", script.display(), err);
                    process::exit(1);
                }
            };

            // Read JSON input from file or stdin
            let json_str = if let Some(path) = input {
                match fs::read_to_string(&path) {
                    Ok(content) => content,
                    Err(err) => {
                        eprintln!("Error reading input JSON '{}': {}", path.display(), err);
                        process::exit(1);
                    }
                }
            } else {
                let mut buffer = String::new();
                if let Err(err) = io::stdin().read_to_string(&mut buffer) {
                    eprintln!("Error reading input from stdin: {}", err);
                    process::exit(1);
                }
                buffer
            };

            let input_val: serde_json::Value = match serde_json::from_str(&json_str) {
                Ok(v) => v,
                Err(err) => {
                    eprintln!("Error: Input is not valid JSON: {}", err);
                    process::exit(1);
                }
            };

            match transform(&script_content, &input_val) {
                Ok(output) => {
                    let formatted = if compact {
                        serde_json::to_string(&output)
                    } else {
                        serde_json::to_string_pretty(&output)
                    };

                    match formatted {
                        Ok(s) => println!("{}", s),
                        Err(err) => {
                            eprintln!("Error formatting output JSON: {}", err);
                            process::exit(1);
                        }
                    }
                }
                Err(err) => {
                    eprintln!("Shae Error: {}", err);
                    process::exit(1);
                }
            }
        }
    }
}
