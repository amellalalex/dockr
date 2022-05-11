use dockr::DockrModule;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let acs = DockrModule::open("acs.json").unwrap_or(DockrModule::new());
    if acs == DockrModule::new() {
        println!("failed to load acs module");
    }
    Ok(())
}

// type Result<T> = std::result::Result<T, DockrError>;

mod dockr {
    use serde::{Deserialize, Serialize};
    use std::{
        fs::File,
        io::BufReader,
        ops::Deref,
        process::{Child, Command},
    };

    pub struct DockrModule {
        name: String,
        cmd: String,
        args: Vec<String>,
        proc: Option<Child>,
    }

    impl PartialEq for DockrModule {
        fn eq(&self, other: &Self) -> bool {
            self.name == other.name && self.cmd == other.cmd && self.args == other.args
        }
    }

    impl Eq for DockrModule {}

    impl DockrModule {
        pub fn new() -> DockrModule {
            DockrModule {
                name: String::new(),
                cmd: String::new(),
                args: Vec::new(),
                proc: None,
            }
        }

        pub fn open(path: &str) -> Result<DockrModule, Box<dyn std::error::Error>> {
            let file = File::open(path)?;
            let rdr = BufReader::new(file);
            let json: DockrJson = serde_json::from_reader(rdr)?;
            Ok(DockrModule::from(json))
        }

        pub fn start(&mut self) -> Result<(), Box<dyn std::error::Error>> {
            if self.is_blank() {}
            println!("Starting {} ...", self.name);
            self.proc = Some(
                Command::new(self.cmd.as_str())
                    // .args(self.args.deref())
                    .args(self.args.deref())
                    .spawn()?,
            );
            Ok(())
        }

        pub fn stop(&mut self) -> Result<(), Box<dyn std::error::Error>> {
            // self.proc.wait().expect("Failed to wait on child");
            match self.proc.as_ref() {
                Some(_) => println!("got some"),
                None => println!("got none"),
            }
            self.proc.as_mut().unwrap().wait()?;
            println!("Stopped module successfully!");
            Ok(())
        }

        pub fn print(&self) {
            println!("Module: {}", self.name);
            println!("Module Command: {}", self.cmd);
            print!("Module Args: ");
            for x in self.args.iter() {
                print!("{}, ", x);
            }
            print!("\n");
            println!("Module Process Running: {}", self.proc.is_some());
        }

        pub fn is_running(&self) -> bool {
            match self.proc {
                Some(_) => true,
                None => false,
            }
        }

        pub fn is_blank(&self) -> bool {
            DockrModule::new() == *self
        }
    }

    impl From<DockrJson> for DockrModule {
        fn from(json: DockrJson) -> Self {
            DockrModule {
                name: json.name,
                cmd: json.cmd,
                args: json.args,
                proc: None,
            }
        }
    }

    #[derive(Debug, Deserialize, Serialize)]
    struct DockrJson {
        name: String,
        cmd: String,
        args: Vec<String>,
    }
}
