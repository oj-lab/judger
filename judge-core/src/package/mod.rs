pub mod icpc;

use std::{path::PathBuf, str::FromStr};

use serde_derive::{Deserialize, Serialize};

use crate::{
    error::JudgeCoreError,
    judge::{CheckerConfig, TestdataConfig},
    run::RlimitConfigs,
};

use self::icpc::ICPCPackageAgent;

pub trait PackageAgent {
    fn init(package_path: PathBuf) -> Result<Self, JudgeCoreError>
    where
        Self: Sized;
    fn validate(&self) -> bool;
    fn get_rlimit_configs(&self) -> Result<RlimitConfigs, JudgeCoreError>;
    fn load_testdata(&self, dest: PathBuf) -> Result<Vec<TestdataConfig>, JudgeCoreError>;
    fn load_checker(&self, dest: PathBuf) -> Result<CheckerConfig, JudgeCoreError>;
}

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
    pub fn get_package_agent(
        &self,
        package_path: PathBuf,
    ) -> Result<Box<dyn PackageAgent>, JudgeCoreError> {
        match self {
            Self::ICPC => Ok(Box::new(ICPCPackageAgent::init(package_path)?)),
        }
    }
}
