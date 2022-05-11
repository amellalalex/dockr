use dockr::DockrModule;

#[test]
fn load_module() {
    let test = DockrModule::open("tests/test.json").unwrap();
    let expected = DockrModule::create(
        "test module",
        "./run_test_module.sh",
        vec!["arg1", "arg2", "arg3"],
    );
    assert_eq!(test, expected);
}
