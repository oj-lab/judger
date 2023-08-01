pub mod discription;

use std::path::PathBuf;

use judge_core::error::JudgeCoreError;

use self::discription::StoragedPackageDiscriptionMap;

pub struct PackageManager {
   pub folder_path: PathBuf,
   pub package_discription_map: StoragedPackageDiscriptionMap,
}

impl PackageManager {
    pub fn new(folder_path: PathBuf) -> Result<Self, JudgeCoreError> {
        if !folder_path.exists() || folder_path.is_file() {
            return Err(JudgeCoreError::AnyhowError(anyhow::anyhow!(
                "Package folder not found (or the path is a file): {}",
                folder_path.display()
            )));
        }

        let discription_file_path = folder_path.join(discription::PACKAGES_DISCRIPTION_FILE_NAME);
        if !discription_file_path.exists() || discription_file_path.is_file() {
            return Err(JudgeCoreError::AnyhowError(anyhow::anyhow!(
                "Package discription file not found (or the path is not a file): {}",
                discription_file_path.display()
            )));
        }

        let package_discription_map = StoragedPackageDiscriptionMap::init(folder_path.clone())?;

        Ok(Self { folder_path, package_discription_map })
    }
}
