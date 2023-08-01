use std::{collections::HashMap, path::PathBuf, fs};

use judge_core::{builder::PackageType, error::JudgeCoreError};
use serde_derive::{Serialize, Deserialize};

pub const PACKAGES_DISCRIPTION_FILE_NAME: &str = "judge-pd.json";

#[derive(Debug, Serialize, Deserialize)]
pub struct PackageDiscription {
    pub name: String,
    pub revision: u32,
    pub package_type: PackageType,
}

pub struct StoragedPackageDiscriptionMap {
    pub folder_path: PathBuf,
    pub package_discription_map: HashMap<String, PackageDiscription>,
}

impl StoragedPackageDiscriptionMap {
    pub fn init(folder_path: PathBuf) -> Result<Self, JudgeCoreError> {
        init_package_discription_file(&folder_path)?;
        let package_discription_map = HashMap::new();
        Ok(Self { folder_path, package_discription_map })
    }

    pub fn load(folder_path: PathBuf) -> Result<Self, JudgeCoreError> {
        let package_discription_map = load_package_discription_map(&folder_path)?;
        Ok(Self { folder_path, package_discription_map })
    }

    pub fn insert(&mut self, package_discription: PackageDiscription) -> Result<(), JudgeCoreError> {
        self.package_discription_map.insert(package_discription.name.clone(), package_discription);
        update_package_discription_file(&self.folder_path, &self.package_discription_map)?;
        Ok(())
    }
}

fn load_package_discription_map(folder: &PathBuf) -> Result<HashMap<String, PackageDiscription>, JudgeCoreError> {
    let discription_file_content = fs::read_to_string(folder.join(PACKAGES_DISCRIPTION_FILE_NAME))?;
    let package_discription_map: HashMap<String, PackageDiscription> = serde_json::from_str(&discription_file_content)?;
    Ok(package_discription_map)
}

fn init_package_discription_file(folder: &PathBuf) -> Result<(), JudgeCoreError> {
    let package_discription_map = HashMap::<String, PackageDiscription>::new();
    let package_discription_file_content = serde_json::to_string_pretty(&package_discription_map)?;

    fs::write(folder.join(PACKAGES_DISCRIPTION_FILE_NAME), package_discription_file_content)?;
    Ok(())
}

fn update_package_discription_file(folder: &PathBuf, package_discription_map: &HashMap<String, PackageDiscription>) -> Result<(), JudgeCoreError> {
    let package_discription_file_content = serde_json::to_string_pretty(package_discription_map)?;

    fs::write(folder.join(PACKAGES_DISCRIPTION_FILE_NAME), package_discription_file_content)?;
    Ok(())
}

#[cfg(test)]
pub mod package_discription_test {
    #[test]
    fn test_storaged_package_discription_map() {
        use std::path::PathBuf;
        use judge_core::builder::PackageType;
        use super::StoragedPackageDiscriptionMap;

        let folder = PathBuf::from("../tmp");
        let mut package_discription_map = StoragedPackageDiscriptionMap::init(folder.clone()).unwrap();

        let package_discription = super::PackageDiscription {
            name: "test".to_string(),
            revision: 1,
            package_type: PackageType::ICPC,
        };
        package_discription_map.insert(package_discription).unwrap();

        let package_discription_map = StoragedPackageDiscriptionMap::load(folder).unwrap();
        assert_eq!(package_discription_map.package_discription_map.len(), 1);
        assert_eq!(package_discription_map.package_discription_map.get("test").unwrap().name, "test");
        assert_eq!(package_discription_map.package_discription_map.get("test").unwrap().revision, 1);
        assert_eq!(package_discription_map.package_discription_map.get("test").unwrap().package_type, PackageType::ICPC);
        
    }
}