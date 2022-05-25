use dockr::Module;

#[test]
fn load_module() {
    let test = Module::open("tests/test.json").unwrap();
    let expected = Module::create(
        "test module",
        "./run_test_module.sh",
        vec!["arg1", "arg2", "arg3"],
    );
    assert_eq!(test, expected);
}
