use std::path::PathBuf;

use judge_core::package::PackageType;
use judger::service::package_manager::description::{
    PackageDescription, StoragedPackageDescriptionMap,
};

const TEST_TEMP_PATH: &str = "tests/temp";

#[test]
fn test_storaged_package_description_map() {
    let folder = PathBuf::from(TEST_TEMP_PATH);
    let mut package_description_map = StoragedPackageDescriptionMap::init(folder.clone()).unwrap();

    let package_description = PackageDescription {
        name: "test".to_string(),
        revision: 1,
        package_type: PackageType::ICPC,
    };
    package_description_map.insert(package_description).unwrap();

    let package_description_map = StoragedPackageDescriptionMap::load(folder).unwrap();
    assert_eq!(package_description_map.package_description_map.len(), 1);
    assert_eq!(
        package_description_map
            .package_description_map
            .get("test")
            .unwrap()
            .name,
        "test"
    );
    assert_eq!(
        package_description_map
            .package_description_map
            .get("test")
            .unwrap()
            .revision,
        1
    );
    assert_eq!(
        package_description_map
            .package_description_map
            .get("test")
            .unwrap()
            .package_type,
        PackageType::ICPC
    );
}
