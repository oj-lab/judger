pub mod description;

use std::path::PathBuf;

use judge_core::package::PackageType;

use crate::service::error::JudgeServiceError;

use self::description::StoragedPackageDescriptionMap;

pub struct PackageManager {
    pub folder_path: PathBuf,
    pub package_description_map: StoragedPackageDescriptionMap,
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

        let description_file_path = folder_path.join(description::PACKAGES_DESCRIPTION_FILE_NAME);
        if description_file_path.exists() && description_file_path.is_dir() {
            return Err(JudgeServiceError::AnyhowError(anyhow::anyhow!(
                "Description file '{}' appears to be a folder.",
                folder_path.display()
            )));
        }
        let package_description_map = if !description_file_path.exists() {
            StoragedPackageDescriptionMap::init(folder_path.clone())?
        } else {
            StoragedPackageDescriptionMap::load(folder_path.clone())?
        };

        Ok(Self {
            folder_path,
            package_description_map,
        })
    }

    pub fn import_package(
        &mut self,
        package_name: String,
        package_type: PackageType,
    ) -> Result<(), JudgeServiceError> {
        let package_description = self.package_description_map.get(&package_name);
        if package_description.is_some() {
            return Err(JudgeServiceError::AnyhowError(anyhow::anyhow!(
                "Package '{}' already exists.",
                package_name
            )));
        }

        if package_type
            .get_package_agent(self.folder_path.join(&package_name))?
            .validate()
        {
            let package_description =
                description::PackageDescription::new(package_name, package_type)?;
            self.package_description_map.insert(package_description)?;
        } else {
            return Err(JudgeServiceError::AnyhowError(anyhow::anyhow!(
                "Package '{}' is not valid.",
                package_name
            )));
        }

        Ok(())
    }
}
