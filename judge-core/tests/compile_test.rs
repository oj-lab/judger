use std::path::PathBuf;

use judge_core::compiler::{Compiler, Language};

const TEST_DATA_PATH: &str = "tests/data";
const TEST_TEMP_PATH: &str = "tests/temp";

fn init() {
    let _ = env_logger::builder().is_test(true).try_init();
}

#[test]
fn test_compile_cpp() {
    init();
    let compiler = Compiler::new(Language::Cpp, vec!["-std=c++17".to_string()]);
    match compiler.compile(
        &PathBuf::from(TEST_DATA_PATH).join("built-in-programs/src/programs/infinite_loop.cpp"),
        &PathBuf::from(TEST_TEMP_PATH).join("infinite_loop_test.o"),
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
        &PathBuf::from(TEST_DATA_PATH).join("built-in-programs/src/programs/read_and_write.py"),
        &PathBuf::from(TEST_TEMP_PATH).join("read_and_write.o"),
    ) {
        Ok(out) => {
            log::info!("{}", out);
        }
        Err(e) => panic!("{:?}", e),
    }
}
