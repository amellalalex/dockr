use std::future;

// fn main() {
//     let mut acs = Module::create("My Module", "echo", vec!["hello dockr!"]);
//     acs.run().expect("failed to run acs module");
// }

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let acs = dockr::Module::open("acs.json")?;
    let acs2 = dockr::Module::open("acs.json")?;
    let pay = dockr::Module::open("pay.json")?;
    let mut mods = dockr::collection!(acs, pay, acs2);

    mods.run_all()?;

    Ok(())
}
