#[test]
fn test_create_dir() {
    assert_wasi_output!(
        "../../wasitests/create_dir.wasm",
        "create_dir",
        vec![".".to_string(),],
        vec![],
        vec![],
        "../../wasitests/create_dir.out"
    );
}
