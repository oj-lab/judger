pub mod discription;

use std::path::PathBuf;

use judge_core::package::PackageType;

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
        let package_discription_map = if !discription_file_path.exists() {
            StoragedPackageDiscriptionMap::init(folder_path.clone())?
        } else {
            StoragedPackageDiscriptionMap::load(folder_path.clone())?
        };

        Ok(Self {
            folder_path,
            package_discription_map,
        })
    }

    pub fn import_package(
        &mut self,
        package_name: String,
        package_type: PackageType,
    ) -> Result<(), JudgeServiceError> {
        let package_discription = self.package_discription_map.get(&package_name);
        if package_discription.is_some() {
            return Err(JudgeServiceError::AnyhowError(anyhow::anyhow!(
                "Package '{}' already exists.",
                package_name
            )));
        }

        if package_type
            .get_package_agent(self.folder_path.join(&package_name))?
            .validate()
        {
            let package_discription =
                discription::PackageDiscription::new(package_name, package_type)?;
            self.package_discription_map.insert(package_discription)?;
        } else {
            return Err(JudgeServiceError::AnyhowError(anyhow::anyhow!(
                "Package '{}' is not valid.",
                package_name
            )));
        }

        Ok(())
    }
}
