use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    io::BufReader,
    ops::Deref,
    process::{Child, Command},
};

// TODO: Put settings into JSON file
// Settings
const STOP_TIMEOUT: u32 = 3; // in seconds
const STOP_REFRESH_DELAY_SECS: u32 = 1;
const STOP_REFRESH_DELAY_NANO: u32 = 0;

#[derive(Debug)]
pub struct Module {
    name: String,
    cmd: String,
    args: Vec<String>,
    proc: Option<Child>,
}

impl PartialEq for Module {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.cmd == other.cmd && self.args == other.args
    }
}

impl Eq for Module {}

impl Module {
    pub fn new() -> Module {
        Module {
            name: String::new(),
            cmd: String::new(),
            args: Vec::new(),
            proc: None,
        }
    }

    pub fn create(name: &str, cmd: &str, args: Vec<&str>) -> Module {
        Module {
            name: name.to_string(),
            cmd: cmd.to_string(),
            args: args.into_iter().map(|arg| arg.to_string()).collect(),
            proc: None,
        }
    }

    pub fn open(path: &str) -> Result<Module, Box<dyn std::error::Error>> {
        let file = File::open(path)?;
        let rdr = BufReader::new(file);
        let json: DockrJson = serde_json::from_reader(rdr)?;
        Ok(Module::from(json))
    }

    pub fn start(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if self.is_blank() {}
        println!("Starting {} ...", self.name);
        self.proc = Some(
            Command::new(self.cmd.as_str())
                .args(self.args.deref())
                .spawn()?,
        );
        Ok(())
    }

    pub fn wait(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(proc) = self.proc.as_mut() {
            proc.wait()?;
        }
        println!("Done waiting on {} !", self.name);
        Ok(())
    }

    // TODO: Finish stop func
    pub fn stop(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(proc) = self.proc.as_mut() {
            if let Some(status) = proc.try_wait()? {}
        }
        Ok(())
    }

    pub fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.start()?;
        self.stop()?;
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
        Module::new() == *self
    }
}

impl From<DockrJson> for Module {
    fn from(json: DockrJson) -> Self {
        Module {
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

pub struct Collection {
    modules: Vec<Module>,
}

impl Collection {
    pub fn new(modules: Vec<Module>) -> Collection {
        Collection { modules }
    }

    pub fn start_all(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        for x in self.modules.iter_mut() {
            x.start()?;
        }
        Ok(())
    }

    pub fn stop_all(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        for module in self.modules.iter_mut() {
            if let Some(child) = module.proc.as_mut() {
                if let Some(status) = child.try_wait()? {
                    println!("module exited with status {status}");
                    module.proc = None;
                } else {
                    println!("Did not stop yet");
                }
            }

            match module {
                Module { proc: None, .. } => println!("great"),
                _ => println!("nothin ado"),
            }
        }

        Ok(())
    }

    pub fn run_all(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        for x in self.modules.iter_mut() {
            x.start()?;
        }
        for x in self.modules.iter_mut() {
            x.stop()?;
        }
        Ok(())
    }
}

#[macro_export]
macro_rules! collection {
    ($x:expr) => {
        Collection::new(vec![$x])
    };
    ($x:expr, $($y:expr),+) => (
        dockr::Collection::new(vec![$x, $($y),+])
    )
}
