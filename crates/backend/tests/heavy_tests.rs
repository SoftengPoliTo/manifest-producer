mod common;

#[test]
fn test_c() {
    common::run_analysis_test("./tests/binaries/ffmpeg", "heavy_test_c");
}
