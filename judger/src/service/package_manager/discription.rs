use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

use judge_core::{error::JudgeCoreError, package::PackageType};
use serde_derive::{Deserialize, Serialize};

use crate::service::error::JudgeServiceError;

pub const PACKAGES_DISCRIPTION_FILE_NAME: &str = "judge-pd.json";

#[derive(Debug, Serialize, Deserialize)]
pub struct PackageDiscription {
    pub name: String,
    pub revision: u32,
    pub package_type: PackageType,
}

impl PackageDiscription {
    pub fn new(name: String, package_type: PackageType) -> Result<Self, JudgeServiceError> {
        Ok(Self {
            name,
            revision: 0,
            package_type,
        })
    }
}

pub struct StoragedPackageDiscriptionMap {
    pub folder_path: PathBuf,
    pub package_discription_map: HashMap<String, PackageDiscription>,
}

impl StoragedPackageDiscriptionMap {
    pub fn init(folder_path: PathBuf) -> Result<Self, JudgeServiceError> {
        init_package_discription_file(&folder_path)?;
        let package_discription_map = HashMap::new();
        Ok(Self {
            folder_path,
            package_discription_map,
        })
    }

    pub fn load(folder_path: PathBuf) -> Result<Self, JudgeServiceError> {
        let package_discription_map = load_package_discription_map(&folder_path)?;
        Ok(Self {
            folder_path,
            package_discription_map,
        })
    }

    pub fn insert(
        &mut self,
        package_discription: PackageDiscription,
    ) -> Result<(), JudgeCoreError> {
        self.package_discription_map
            .insert(package_discription.name.clone(), package_discription);
        update_package_discription_file(&self.folder_path, &self.package_discription_map)?;
        Ok(())
    }

    pub fn get(&self, package_name: &str) -> Option<&PackageDiscription> {
        self.package_discription_map.get(package_name)
    }
}

fn load_package_discription_map(
    folder: &Path,
) -> Result<HashMap<String, PackageDiscription>, JudgeCoreError> {
    let discription_file_content = fs::read_to_string(folder.join(PACKAGES_DISCRIPTION_FILE_NAME))?;
    let package_discription_map: HashMap<String, PackageDiscription> =
        serde_json::from_str(&discription_file_content)?;
    Ok(package_discription_map)
}

fn init_package_discription_file(folder: &Path) -> Result<(), JudgeCoreError> {
    let package_discription_map = HashMap::<String, PackageDiscription>::new();
    let package_discription_file_content = serde_json::to_string_pretty(&package_discription_map)?;

    fs::write(
        folder.join(PACKAGES_DISCRIPTION_FILE_NAME),
        package_discription_file_content,
    )?;
    Ok(())
}

fn update_package_discription_file(
    folder: &Path,
    package_discription_map: &HashMap<String, PackageDiscription>,
) -> Result<(), JudgeCoreError> {
    let package_discription_file_content = serde_json::to_string_pretty(package_discription_map)?;

    fs::write(
        folder.join(PACKAGES_DISCRIPTION_FILE_NAME),
        package_discription_file_content,
    )?;
    Ok(())
}
