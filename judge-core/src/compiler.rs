use std::process::{Command, Output};

use crate::error::JudgeCoreError;

pub enum CompilerType {
    GccV9,
    GppV9,
}

pub struct CompileConfig {
    pub compiler_type: CompilerType,
    pub src_path: String,
    pub target_path: String,
}

pub struct CompileCommand {
    pub program: &'static str,
    pub args: Vec<String>,
}

pub fn compile(config: &CompileConfig) -> Result<Output, JudgeCoreError> {
    let compile_command = get_command(&config);
    Ok(Command::new(compile_command.program)
        .args(compile_command.args)
        .output()?)
}

fn get_command(config: &CompileConfig) -> CompileCommand {
    match config.compiler_type {
        CompilerType::GccV9 => CompileCommand {
            program: "gcc",
            args: vec![
                "-o".to_string(),
                config.target_path.clone(),
                config.src_path.clone(),
            ],
        },
        CompilerType::GppV9 => CompileCommand {
            program: "g++",
            args: vec![
                "-o".to_string(),
                config.target_path.clone(),
                config.src_path.clone(),
            ],
        },
    }
}

#[cfg(test)]
pub mod compiler {
    use super::{CompileConfig, compile};

    #[test]
    fn test_compile() {
        let config = CompileConfig {
            compiler_type: super::CompilerType::GppV9,
            src_path: "../infinite_loop.cpp".to_string(),
            target_path: "../infinite_loop_test".to_string(),
        };
        match compile(&config) {
            Ok(out) => {
                if !out.status.success() {
                    panic!("{:?}", out)
                }
            },
            Err(e) => panic!("{:?}", e),
        }
    }
}
