mod common;
use common::run_analysis_test;

#[test]
fn test_c() {
    run_analysis_test(
        "./tests/binaries/minimal-fake-firmware-c-static",
        "light_test_c",
    );
}

#[test]
fn test_cpp() {
    run_analysis_test(
        "./tests/binaries/minimal-fake-firmware-cpp-static",
        "light_test_cpp",
    );
}

#[test]
fn test_rust() {
    run_analysis_test("./tests/binaries/fridge", "light_test_rust");
}
