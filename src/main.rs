use dockr::DockrModule;

fn main() {
    let mut acs = DockrModule::create("My Module", "echo", vec!["hello dockr!"]);
    acs.run().expect("failed to run acs module");
}
