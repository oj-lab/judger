use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

use judge_core::{error::JudgeCoreError, package::PackageType};
use serde_derive::{Deserialize, Serialize};

use crate::service::error::JudgeServiceError;

pub const PACKAGES_DESCRIPTION_FILE_NAME: &str = "judge-pd.json";

#[derive(Debug, Serialize, Deserialize)]
pub struct PackageDescription {
    pub name: String,
    pub revision: u32,
    pub package_type: PackageType,
}

impl PackageDescription {
    pub fn new(name: String, package_type: PackageType) -> Result<Self, JudgeServiceError> {
        Ok(Self {
            name,
            revision: 0,
            package_type,
        })
    }
}

pub struct StoragedPackageDescriptionMap {
    pub folder_path: PathBuf,
    pub package_description_map: HashMap<String, PackageDescription>,
}

impl StoragedPackageDescriptionMap {
    pub fn init(folder_path: PathBuf) -> Result<Self, JudgeServiceError> {
        init_package_description_file(&folder_path)?;
        let package_description_map = HashMap::new();
        Ok(Self {
            folder_path,
            package_description_map,
        })
    }

    pub fn load(folder_path: PathBuf) -> Result<Self, JudgeServiceError> {
        let package_description_map = load_package_description_map(&folder_path)?;
        Ok(Self {
            folder_path,
            package_description_map,
        })
    }

    pub fn insert(
        &mut self,
        package_description: PackageDescription,
    ) -> Result<(), JudgeCoreError> {
        self.package_description_map
            .insert(package_description.name.clone(), package_description);
        update_package_description_file(&self.folder_path, &self.package_description_map)?;
        Ok(())
    }

    pub fn get(&self, package_name: &str) -> Option<&PackageDescription> {
        self.package_description_map.get(package_name)
    }
}

fn load_package_description_map(
    folder: &Path,
) -> Result<HashMap<String, PackageDescription>, JudgeCoreError> {
    let description_file_content = fs::read_to_string(folder.join(PACKAGES_DESCRIPTION_FILE_NAME))?;
    let package_description_map: HashMap<String, PackageDescription> =
        serde_json::from_str(&description_file_content)?;
    Ok(package_description_map)
}

fn init_package_description_file(folder: &Path) -> Result<(), JudgeCoreError> {
    let package_description_map = HashMap::<String, PackageDescription>::new();
    let package_description_file_content = serde_json::to_string_pretty(&package_description_map)?;

    fs::write(
        folder.join(PACKAGES_DESCRIPTION_FILE_NAME),
        package_description_file_content,
    )?;
    Ok(())
}

fn update_package_description_file(
    folder: &Path,
    package_description_map: &HashMap<String, PackageDescription>,
) -> Result<(), JudgeCoreError> {
    let package_description_file_content = serde_json::to_string_pretty(package_description_map)?;

    fs::write(
        folder.join(PACKAGES_DESCRIPTION_FILE_NAME),
        package_description_file_content,
    )?;
    Ok(())
}
