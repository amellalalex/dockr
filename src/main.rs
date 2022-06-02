use dockr::Collection;
use dockr::Module;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let mut coll = Collection::open_dir("modules")?;

    coll.run_all()?;

    Ok(())
}
