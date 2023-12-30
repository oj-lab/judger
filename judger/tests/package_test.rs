use std::path::PathBuf;

use judge_core::package::PackageType;
use judger::service::package_manager::discription::{
    PackageDiscription, StoragedPackageDiscriptionMap,
};

const TEST_TEMP_PATH: &str = "tests/temp";

#[test]
fn test_storaged_package_discription_map() {
    let folder = PathBuf::from(TEST_TEMP_PATH);
    let mut package_discription_map = StoragedPackageDiscriptionMap::init(folder.clone()).unwrap();

    let package_discription = PackageDiscription {
        name: "test".to_string(),
        revision: 1,
        package_type: PackageType::ICPC,
    };
    package_discription_map.insert(package_discription).unwrap();

    let package_discription_map = StoragedPackageDiscriptionMap::load(folder).unwrap();
    assert_eq!(package_discription_map.package_discription_map.len(), 1);
    assert_eq!(
        package_discription_map
            .package_discription_map
            .get("test")
            .unwrap()
            .name,
        "test"
    );
    assert_eq!(
        package_discription_map
            .package_discription_map
            .get("test")
            .unwrap()
            .revision,
        1
    );
    assert_eq!(
        package_discription_map
            .package_discription_map
            .get("test")
            .unwrap()
            .package_type,
        PackageType::ICPC
    );
}
