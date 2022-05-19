use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    io::BufReader,
    ops::Deref,
    process::{Child, Command},
};

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

    pub fn stop(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // self.proc.wait().expect("Failed to wait on child");
        match self.proc.as_ref() {
            Some(_) => println!("got some"),
            None => println!("got none"),
        }
        self.proc.as_mut().unwrap().wait()?;
        println!("Stopped {} successfully!", self.name);
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
