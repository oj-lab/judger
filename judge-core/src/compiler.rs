use crate::error::JudgeCoreError;
use crate::utils::get_pathbuf_str;
use anyhow::anyhow;
use std::fmt;
use std::path::PathBuf;
use std::{process::Command, str::FromStr};

const TEMPLATE_ARG_SRC_PATH: &str = "{src_path}";
const TEMPLATE_ARG_TARGET_PATH: &str = "{target_path}";

const RUST_COMPILE_COMMAND_TEMPLATE: &str = "rustc {src_path} -o {target_path}";
const CPP_COMPILE_COMMAND_TEMPLATE: &str = "g++ {src_path} -o {target_path}";
const PYTHON_COMPILE_COMMAND_TEMPLATE: &str = "cp {src_path} {target_path}";

#[derive(Clone)]
struct CommandBuilder {
    command_template: String,
    template_args: Vec<String>,
}

impl CommandBuilder {
    pub fn new(command_template: String, template_args: Vec<String>) -> Self {
        Self {
            command_template,
            template_args,
        }
    }

    // TODO: check if args match template_args
    pub fn get_command(&self, args: Vec<String>) -> String {
        let mut command = self.command_template.to_string();
        for (i, arg) in self.template_args.iter().enumerate() {
            command = command.replace(arg, &args[i]);
        }
        command
    }
}

#[derive(Debug, Clone, PartialEq, Copy)]
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
    command_builder: CommandBuilder,
    compiler_args: Vec<String>,
}

impl Compiler {
    pub fn new(language: Language, compiler_args: Vec<String>) -> Self {
        let command_template = match language {
            Language::Rust => RUST_COMPILE_COMMAND_TEMPLATE,
            Language::Cpp => CPP_COMPILE_COMMAND_TEMPLATE,
            Language::Python => PYTHON_COMPILE_COMMAND_TEMPLATE,
            // TODO: add other supported language
        }
        .to_owned();
        let template_args = match language {
            Language::Rust | Language::Cpp | Language::Python => vec![
                TEMPLATE_ARG_SRC_PATH.to_owned(),
                TEMPLATE_ARG_TARGET_PATH.to_owned(),
            ],
            // TODO: add other supported language
        };
        let command_builder = CommandBuilder::new(command_template, template_args);
        Self {
            language,
            command_builder,
            compiler_args,
        }
    }

    pub fn compile(
        &self,
        src_path: &PathBuf,
        target_path: &PathBuf,
    ) -> Result<String, JudgeCoreError> {
        if !PathBuf::from(src_path).exists() {
            return Err(JudgeCoreError::AnyhowError(anyhow!(
                "Source file not found: {:?}",
                src_path
            )));
        }

        let src_path_string = get_pathbuf_str(src_path)?;
        let target_path_string = get_pathbuf_str(target_path)?;

        log::info!(
            "Compiling language={} src={} target={}",
            self.language,
            src_path_string,
            target_path_string
        );

        if PathBuf::from(target_path).exists() {
            std::fs::remove_file(target_path)?;
        }

        let output = Command::new("sh")
            .arg("-c")
            .arg(
                &self
                    .command_builder
                    .get_command(vec![src_path_string, target_path_string]),
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
    use std::path::PathBuf;

    use super::{Compiler, Language};

    fn init() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    #[test]
    fn test_compile_cpp() {
        init();
        let compiler = Compiler::new(Language::Cpp, vec!["-std=c++17".to_string()]);
        match compiler.compile(
            &PathBuf::from("../test-collection/src/programs/infinite_loop.cpp"),
            &PathBuf::from("../tmp/infinite_loop_test"),
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
            &PathBuf::from("../test-collection/src/programs/read_and_write.py"),
            &PathBuf::from("../tmp/read_and_write"),
        ) {
            Ok(out) => {
                log::info!("{}", out);
            }
            Err(e) => panic!("{:?}", e),
        }
    }
}
