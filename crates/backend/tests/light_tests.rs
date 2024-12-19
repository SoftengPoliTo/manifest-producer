mod common;

#[test]
fn test_c() {
    common::run_analysis_test(
        "./tests/binaries/minimal-fake-firmware-c-static",
        "light_test_c",
    );
}

#[test]
fn test_cpp() {
    common::run_analysis_test(
        "./tests/binaries/minimal-fake-firmware-cpp-static",
        "light_test_cpp",
    );
}

#[test]
fn test_rust() {
    common::run_analysis_test("./tests/binaries/fridge", "light_test_rust");
}
