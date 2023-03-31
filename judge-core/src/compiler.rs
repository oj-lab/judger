use std::{
    process::Command,
    str::FromStr,
};
use crate::utils::TemplateCommand;

#[derive(Clone)]
pub enum Language {
    Rust,
    Cpp,
    Python,
    // add other supported languages here
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
            Language::Python => panic!("Cannot be compiled"),
            // add other supported language
        };
        let template_args = match language {
            Language::Rust => vec!["{src_path}".to_string(), "{target_path}".to_string()],
            Language::Cpp => vec!["{src_path}".to_string(), "{target_path}".to_string()],
            Language::Python => panic!("Cannot be compiled"),
            // add other supported language
        };
        let command = TemplateCommand::new(compiler_name.clone(), template_args);
        Self {
            language,
            command,
            compiler_args
        }
    }

    pub fn compile(&self, src_path: &str, target_path: &str) -> Result<String, String> {
        let output = Command::new("sh")
            .arg("-c")
            .arg(&self.command.get_command(vec![src_path.to_string(), target_path.to_string()]))
            .args(self.compiler_args.iter())
            .output()
            .map_err(|e| format!("Failed to execute compiler: {}", e))?;

        if output.status.success() {
            let compile_output = String::from_utf8_lossy(&output.stdout).to_string();
            Ok(compile_output)
        } else { // define error
            let error_output = String::from_utf8_lossy(&output.stderr).to_string();
            Err(error_output)
        }
    }
}


#[cfg(test)]
pub mod compiler {
    use super::{Compiler, Language};

    #[test]
    fn test_compile_cpp() {
        let compiler = Compiler::new(Language::Cpp, vec!["-std=c++17".to_string()]);
        match compiler.compile("../infinite_loop.cpp", "../infinite_loop_test") {
            Ok(out) => {
                println!("{}", out);
            }
            Err(e) => panic!("{:?}", e),
        }
    }
}
