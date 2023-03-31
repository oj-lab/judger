use clap::{Parser, Subcommand};
use judge_core::compiler::{Compiler, Language};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Compile {
        #[arg(short, long)]
        source: String,
        #[arg(short, long)]
        target: String,
        #[arg(short, long)]
        language: Language,
    },
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Some(Commands::Compile {
            source,
            target,
            language,
        }) => {
            let compiler = Compiler::new(language, vec!["-std=c++17".to_string()]);
            let output = compiler.compile(&source, &target);
            println!("{:?}", output)
        }
        None => {}
    }
}
