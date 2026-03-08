use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "calpad", version, about = "Natural language calculator")]
struct Cli {
    /// Expression to evaluate (e.g. "5kg in pounds")
    expression: Option<String>,

    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// Interactive TUI mode
    Tui,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Some(Command::Tui) => {
            eprintln!("TUI mode coming soon.");
            std::process::exit(0);
        }
        None => {}
    }

    if let Some(expr) = cli.expression {
        let results = calpad_core::evaluate_document(&expr);
        for r in results {
            if !r.display.is_empty() {
                println!("{}", r.display);
            }
        }
    } else {
        // REPL mode: read from stdin
        use std::io::{self, BufRead};
        let stdin = io::stdin();
        let input: String = stdin.lock().lines().map_while(Result::ok).collect::<Vec<_>>().join("\n");
        if !input.is_empty() {
            let results = calpad_core::evaluate_document(&input);
            for r in results {
                if !r.display.is_empty() {
                    println!("{}", r.display);
                }
            }
        }
    }
}
