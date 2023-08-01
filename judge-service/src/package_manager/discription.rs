use judge_core::builder::PackageType;
use serde_derive::{Serialize, Deserialize};

pub const PACKAGES_DISCRIPTION_FILE_NAME: &str = "judge-pd.json";

#[derive(Debug, Serialize, Deserialize)]
pub struct PackageDiscription {
    pub name: String,
    pub revision: u32,
    pub package_type: PackageType,
}