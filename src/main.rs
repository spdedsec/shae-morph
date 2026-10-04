use clap::{Parser, Subcommand};
use shae::transform;
use std::fs;
use std::io::{self, IsTerminal, Read};
use std::path::PathBuf;
use std::process;

const BANNER: &str = r#"
   ____  _                 
  / ___|| |__   __ _  ___  
  \___ \| '_ \ / _` |/ _ \ 
   ___) | | | | (_| |  __/ 
  |____/|_| |_|\__,_|\___| 
  Data plumbing without the tears.
"#;

const TIPS: &[&str] = &[
    "💡 Pro-tip: Use `??` to provide fallback defaults. Null crashes are so 2015.",
    "💡 Pro-tip: You can filter arrays with `where`, like `from .users[] where .age >= 18 select { ... }`.",
    "💡 Pro-tip: Pipe curl straight into Shae: `curl -s https://api.com/data | shae run transform.shae`.",
    "💡 Pro-tip: Transform strings effortlessly with `.trim().uppercase()` or `.capitalize()`.",
    "💡 Pro-tip: Convert string numbers from sloppy APIs using `.toInt()` or `.toNumber()`.",
];

#[derive(Parser, Debug)]
#[command(name = "shae")]
#[command(version = "0.1.0")]
#[command(
    about = "Shae — Structural Data Transformation & JSON Plumbing Language",
    before_help = BANNER
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Show a random developer tip or wisdom
    #[arg(long)]
    tip: bool,
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
    /// Show a helpful tip for data plumbing
    Tip,
}

fn main() {
    let cli = Cli::parse();

    if cli.tip {
        print_random_tip();
        return;
    }

    match cli.command {
        Some(Commands::Tip) => {
            print_random_tip();
        }
        Some(Commands::Run {
            script,
            input,
            compact,
        }) => {
            // Read script content
            let script_content = match fs::read_to_string(&script) {
                Ok(content) => content,
                Err(err) => {
                    eprintln!("Shae Error: Could not read script file '{}': {}", script.display(), err);
                    eprintln!("(Did you check if the filename is spelled right?)");
                    process::exit(1);
                }
            };

            // Read JSON input from file or stdin
            let json_str = if let Some(path) = input {
                match fs::read_to_string(&path) {
                    Ok(content) => content,
                    Err(err) => {
                        eprintln!("Shae Error: Could not read input JSON '{}': {}", path.display(), err);
                        process::exit(1);
                    }
                }
            } else {
                let stdin = io::stdin();
                // Check if user is running interactively in terminal without piping
                if stdin.is_terminal() {
                    eprintln!("🤔 Shae is waiting for JSON input from stdin...");
                    eprintln!("👉 Tip: Pass a JSON file: `shae run {} data.json`", script.display());
                    eprintln!("👉 Or pipe into it: `cat data.json | shae run {}`", script.display());
                }

                let mut buffer = String::new();
                if let Err(err) = io::stdin().read_to_string(&mut buffer) {
                    eprintln!("Shae Error: Failed to read from stdin: {}", err);
                    process::exit(1);
                }
                buffer
            };

            let input_val: serde_json::Value = match serde_json::from_str(&json_str) {
                Ok(v) => v,
                Err(err) => {
                    eprintln!("Shae Error: Input is not valid JSON: {}", err);
                    eprintln!("(Make sure your input payload has valid quotes and brackets!)");
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
                            eprintln!("Shae Error: Failed to format output JSON: {}", err);
                            process::exit(1);
                        }
                    }
                }
                Err(err) => {
                    eprintln!("{}", err);
                    process::exit(1);
                }
            }
        }
        None => {
            println!("{}", BANNER);
            println!("Usage: shae run <script.shae> [input.json]");
            println!("       cat data.json | shae run <script.shae>");
            println!("\nRun `shae --help` for full options, or `shae --tip` for a friendly hint!");
        }
    }
}

fn print_random_tip() {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos() as usize;
    let idx = nanos % TIPS.len();
    println!("{}", TIPS[idx]);
}
