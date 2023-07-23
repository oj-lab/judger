use std::path::PathBuf;

use clap::{Parser, Subcommand};
use judge_core::{compiler::{Compiler, Language}, builder::{PackageType, JudgeBuilder, JudgeBuilderInput}, judge::{JudgeConfig, common::run_judge}};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Compile a single file source code into an executable
    Compile {
        /// Path of the src file
        #[arg(short, long)]
        source: String,
        /// Path to place the compiled executable
        #[arg(short, long)]
        target: String,
        #[arg(short, long)]
        /// Supported are: rust | cpp | python
        language: Language,
    },
    /// Run a batch of judges with specified problem package and src input
    BatchJudge {
        /// Path of the testing src file
        #[arg(short, long)]
        source: PathBuf,
        /// Supported are: rust | cpp | python
        #[arg(short = 'l', long)]
        source_language: Language,
        /// Path of the problem package to run the judge
        #[arg(short, long)]
        package: PathBuf,
        /// Supported are: icpc
        #[arg(short = 't', long)]
        package_type: PackageType,
        /// Path to run and store the runtime files
        #[arg(short, long, default_value = "/tmp")]
        runtime_path: PathBuf,
    },
}

fn main() {
    // TODO: use some flags to control weather log is printed
    // let _ = env_logger::builder()
    //         .filter_level(log::LevelFilter::Debug)
    //         .try_init();
    let cli = Cli::parse();
    match cli.command {
        Some(Commands::Compile {
            source,
            target,
            language,
        }) => {
            let compiler = Compiler::new(language, vec!["-std=c++17".to_string()]);
            let output = compiler.compile(&PathBuf::from(source), &PathBuf::from(target));
            println!("{:?}", output)
        }
        Some(Commands::BatchJudge {
            source,
            source_language,
            package,
            package_type,
            runtime_path,    
        }) => {
            let new_builder_result = JudgeBuilder::new(JudgeBuilderInput {
                package_type: package_type,
                package_path: package,
                runtime_path: runtime_path,
                src_language: source_language,
                src_path: source,
            });
            if new_builder_result.is_err() {
                println!("Failed to new builder result: {:?}", new_builder_result.err());
                return;
            }
            let builder = new_builder_result.unwrap();
            println!("Builder created: {:?}", builder);
            for idx in 0..builder.testdata_configs.len() {
                let judge_config = JudgeConfig {
                    test_data: builder.testdata_configs[idx].clone(),
                    program: builder.program_config.clone(),
                    checker: builder.checker_config.clone(),
                    runtime: builder.runtime_config.clone(),
                };
                let result = run_judge(&judge_config);
                println!("Judge result: {:?}", result);
            }

            println!("BatchJudge finished")
        }
        None => {
            println!("Please specify a COMMAND, use --help to see more")
        }
    }
}
