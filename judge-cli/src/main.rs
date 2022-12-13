use clap::{Parser, Subcommand};
use judge_core::compiler::CompilerType;

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
        compiler: CompilerType,
    },
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Some(Commands::Compile {
            source,
            target,
            compiler,
        }) => {
            let config = judge_core::compiler::CompileConfig {
                compiler_type: compiler,
                src_path: source,
                target_path: target,
            };
            let output = judge_core::compiler::compile(&config).unwrap();
            println!("{:?}", output)
        }
        None => {}
    }
}
