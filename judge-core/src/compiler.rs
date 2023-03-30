use std::{
    process::Command,
    str::FromStr,
};

#[derive(Clone)]
pub enum Language {
    Rust,
    Cpp,
    // add other supported languages here
}

impl FromStr for Language {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "rust" => Ok(Self::Rust),
            "cpp" => Ok(Self::Cpp),
            _ => Err(anyhow::anyhow!("Compiler not found: {}", s)),
        }
    }
}

#[derive(Clone)]
pub struct Compiler {
    language: Language,
    compiler_name: String,
    compiler_args: Vec<String>,
}

impl Compiler {
    pub fn new(language: Language, compiler_args: Vec<String>) -> Self {
        let compiler_name = match language {
            Language::Rust => "rustc".to_string(),
            Language::Cpp => "g++".to_string(),
            // add other supported language
        };
        Self {
            language,
            compiler_name,
            compiler_args
        }
    }

    pub fn compile(&self, src_path: &str, target_path: &str) -> Result<String, String> {
        let output = Command::new(&self.compiler_name)
            .args(self.compiler_args.iter().chain(["-o".to_string(), target_path.clone().to_string(), src_path.to_string()].iter().collect::<Vec<&String>>()))
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
