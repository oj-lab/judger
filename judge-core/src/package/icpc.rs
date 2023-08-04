use std::path::PathBuf;

use super::PackageAgent;

pub struct ICPCPackageAgent;

impl PackageAgent for ICPCPackageAgent {
    fn validate(&self, package_path: PathBuf) -> bool {
        if !package_path.exists() || package_path.is_file() {
            return false;
        }
        true
    }
}
