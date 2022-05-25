fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let mut acs = dockr::Module::open("acs.json")?;
    // let mut acs2 = dockr::Module::open("acs.json")?;
    // let mut pay = dockr::Module::open("pay.json")?;

    acs.start()?;
    acs.wait()?;

    Ok(())
}
