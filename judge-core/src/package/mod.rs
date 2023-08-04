pub mod icpc;

use std::{path::PathBuf, str::FromStr};

use serde_derive::{Deserialize, Serialize};

use self::icpc::ICPCPackageAgent;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PackageType {
    ICPC,
}

impl FromStr for PackageType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "icpc" => Ok(Self::ICPC),
            _ => Err(anyhow::anyhow!("PackageType not found: {}", s)),
        }
    }
}

impl PackageType {
    pub fn get_package_agent(&self) -> Box<dyn PackageAgent> {
        match self {
            Self::ICPC => Box::new(ICPCPackageAgent),
        }
    }
}

pub trait PackageAgent {
    fn validate(&self, package_path: PathBuf) -> bool;
}
