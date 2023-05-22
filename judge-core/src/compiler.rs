use crate::error::JudgeCoreError;
use crate::utils::TemplateCommand;
use anyhow::anyhow;
use std::fmt;
use std::{process::Command, str::FromStr};

#[derive(Clone, PartialEq, Copy)]
pub enum Language {
    Rust,
    Cpp,
    Python,
    // add other supported languages here
}

impl fmt::Display for Language {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Rust => write!(f, "rust"),
            Self::Cpp => write!(f, "cpp"),
            Self::Python => write!(f, "python"),
            // add other supported languages here
        }
    }
}

impl FromStr for Language {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "rust" => Ok(Self::Rust),
            "cpp" => Ok(Self::Cpp),
            "python" => Ok(Self::Python),
            _ => Err(anyhow::anyhow!("Compiler not found: {}", s)),
        }
    }
}

#[derive(Clone)]
pub struct Compiler {
    language: Language,
    command: TemplateCommand,
    compiler_args: Vec<String>,
}

impl Compiler {
    pub fn new(language: Language, compiler_args: Vec<String>) -> Self {
        let compiler_name = match language {
            Language::Rust => "rustc {src_path} -o {target_path}".to_string(),
            Language::Cpp => "g++ {src_path} -o {target_path}".to_string(),
            Language::Python => "cp {src_path} {target_path}".to_string(),
            // add other supported language
        };
        let template_args = match language {
            Language::Rust => vec!["{src_path}".to_string(), "{target_path}".to_string()],
            Language::Cpp => vec!["{src_path}".to_string(), "{target_path}".to_string()],
            Language::Python => vec!["{src_path}".to_string(), "{target_path}".to_string()],
            // add other supported language
        };
        let command = TemplateCommand::new(compiler_name, template_args);
        Self {
            language,
            command,
            compiler_args,
        }
    }

    pub fn compile(&self, src_path: &str, target_path: &str) -> Result<String, JudgeCoreError> {
        log::info!(
            "Compiling language={} src={} target={}",
            self.language,
            src_path,
            target_path
        );
        let output = Command::new("sh")
            .arg("-c")
            .arg(
                &self
                    .command
                    .get_command(vec![src_path.to_string(), target_path.to_string()]),
            )
            .args(self.compiler_args.iter())
            .output()?;
        if output.status.success() {
            let compile_output = String::from_utf8_lossy(&output.stdout).to_string();
            log::info!("Compile output: {}", compile_output);
            Ok(compile_output)
        } else {
            let error_output = String::from_utf8_lossy(&output.stderr).to_string();
            log::error!("Compile error: {}", error_output);
            Err(JudgeCoreError::AnyhowError(anyhow!(error_output)))
        }
    }
}

#[cfg(test)]
pub mod compiler {
    use super::{Compiler, Language};

    fn init() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    #[test]
    fn test_compile_cpp() {
        init();
        let compiler = Compiler::new(Language::Cpp, vec!["-std=c++17".to_string()]);
        match compiler.compile(
            "../test-collection/src/programs/infinite_loop.cpp",
            "../tmp/infinite_loop_test",
        ) {
            Ok(out) => {
                log::info!("{}", out);
            }
            Err(e) => panic!("{:?}", e),
        }
    }

    #[test]
    fn test_compile_py() {
        init();
        let compiler = Compiler::new(Language::Python, vec![]);
        match compiler.compile(
            "../test-collection/src/programs/read_and_write.py",
            "../tmp/read_and_write",
        ) {
            Ok(out) => {
                log::info!("{}", out);
            }
            Err(e) => panic!("{:?}", e),
        }
    }
}
