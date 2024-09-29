use crate::error::JudgeCoreError;
use crate::utils::get_pathbuf_str;
use anyhow::anyhow;
use serde_derive::{Deserialize, Serialize};
use std::path::PathBuf;
use std::{fmt, fs};
use std::{process::Command, str::FromStr};

const TEMPLATE_ARG_SRC_PATH: &str = "{src_path}";
const TEMPLATE_ARG_TARGET_PATH: &str = "{target_path}";

const RUST_COMPILE_COMMAND_TEMPLATE: &str = "rustc {src_path} -o {target_path}";
const CPP_COMPILE_COMMAND_TEMPLATE: &str = "g++ {src_path} -o {target_path} -O2 -static";
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

#[derive(Debug, Clone, PartialEq, Copy, Serialize, Deserialize)]
pub enum Language {
    #[serde(rename = "rust")]
    Rust,
    #[serde(rename = "cpp")]
    Cpp,
    #[serde(rename = "python")]
    Python,
    // add other supported languages here
}

impl Language {
    pub fn get_extension(&self) -> String {
        match self {
            Self::Rust => "rs".to_string(),
            Self::Cpp => "cpp".to_string(),
            Self::Python => "py".to_string(),
        }
    }
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

/// Get specific compiler for a language, then compile source code into an executable
///
/// # Example
///
/// ```
/// use judge_core::compiler::{Compiler, Language};
/// use std::path::PathBuf;
///
/// let compiler = Compiler::new(Language::Cpp, vec!["-std=c++17".to_string()]);
/// match compiler.compile(
///     &PathBuf::from("tests/data/built-in-programs/src/programs/infinite_loop.cpp"),
///     &PathBuf::from("tests/temp/infinite_loop_test"),
/// ) {
///     Ok(out) => {
///         log::info!("compiled with output: {}", out);
///     }
///     Err(e) => panic!("compile err: {:?}", e),
/// }
/// ```
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

        if let Some(target_parent) = target_path.parent() {
            fs::create_dir_all(target_parent)?;
        }

        if PathBuf::from(target_path).exists() {
            std::fs::remove_file(target_path)?;
        }

        let output = Command::new("sh")
            .arg("-c")
            .arg(
                self.command_builder
                    .get_command(vec![src_path_string, target_path_string]),
            )
            .args(self.compiler_args.iter())
            .output()?;
        if output.status.success() {
            let compile_output = String::from_utf8_lossy(&output.stdout).to_string();
            log::debug!("Compile output: {}", compile_output);
            Ok(compile_output)
        } else {
            let error_output = String::from_utf8_lossy(&output.stderr).to_string();
            log::error!("Compile error: {}", error_output);
            Err(JudgeCoreError::CompileError(error_output))
        }
    }
}
