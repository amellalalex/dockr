use std::future;

use dockr::DockrModule;

// fn main() {
//     let mut acs = DockrModule::create("My Module", "echo", vec!["hello dockr!"]);
//     acs.run().expect("failed to run acs module");
// }

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let acs = DockrModule::open("acs.json")?;
    let acs2 = DockrModule::open("acs.json")?;
    let pay = DockrModule::open("pay.json")?;
    let mut mods = vec![acs, pay, acs2];

    // for x in &mut mods {
    //     x.keep_alive();
    // }s

    Ok(())
}
