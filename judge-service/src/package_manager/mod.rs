pub mod discription;

use std::path::PathBuf;

use crate::error::JudgeServiceError;

use self::discription::StoragedPackageDiscriptionMap;

pub struct PackageManager {
    pub folder_path: PathBuf,
    pub package_discription_map: StoragedPackageDiscriptionMap,
}

impl PackageManager {
    pub fn new(folder_path: PathBuf) -> Result<Self, JudgeServiceError> {
        if folder_path.exists() && folder_path.is_file() {
            return Err(JudgeServiceError::AnyhowError(anyhow::anyhow!(
                "Package folder '{}' appears to be a file.",
                folder_path.display()
            )));
        }

        if !folder_path.exists() {
            std::fs::create_dir_all(&folder_path)?;
        }

        let discription_file_path = folder_path.join(discription::PACKAGES_DISCRIPTION_FILE_NAME);
        if discription_file_path.exists() && discription_file_path.is_dir() {
            return Err(JudgeServiceError::AnyhowError(anyhow::anyhow!(
                "Discription file '{}' appears to be a folder.",
                folder_path.display()
            )));
        }
        let package_discription_map;
        if !discription_file_path.exists() {
            package_discription_map = StoragedPackageDiscriptionMap::init(folder_path.clone())?;
        } else {
            package_discription_map = StoragedPackageDiscriptionMap::load(folder_path.clone())?;
        }

        Ok(Self {
            folder_path,
            package_discription_map,
        })
    }
}
