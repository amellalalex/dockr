use core::fmt;
use std::{
    fs::File,
    io::BufReader,
    process::{Child, Command},
};

use serde::{Deserialize, Serialize};

fn main() {
    // let acs = DockrModule::new();
    // let pay = DockrModule::open("pay.json")
    //     .expect("Unable to open module config file")
    //     .expect("msg");
}

type Result<T> = std::result::Result<T, DockrError>;

#[derive(Debug, Clone)]
struct DockrError {
    msg: String,
}

impl fmt::Display for DockrError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Generic Dockr Error")
    }
}

impl From<std::io::Error> for DockrError {
    fn from(err: std::io::Error) -> Self {
        match err {
            Err(err) => 
        }
    }
}

struct DockrModule {
    name: String,
    cmd: String,
    args: Vec<String>,
    proc: Option<Child>,
}

impl DockrModule {
    fn new() -> DockrModule {
        DockrModule {
            name: String::new(),
            cmd: String::new(),
            args: Vec::new(),
            proc: None,
        }
    }

    // fn open(path: &str) -> Result<Result<Option<DockrModule>, serde_json::Error>, std::io::Error> {
    //     File::open(path).map(|f| {
    //         let rdr = BufReader::new(f);
    //         serde_json::from_reader(rdr).map(|json| {
    //             let module: DockrJson = json;
    //             Some(DockrModule::from(module))
    //         })
    //     })
    // }

    fn open(path: &str) -> Result<DockrModule> {
        let file = File::open(path).map_err(|_| DockrError)?;
        let rdr = BufReader::new(file);
        let json: DockrJson = serde_json::from_reader(rdr).map_err(|_| DockrError)?;
        Ok(DockrModule::from(json))
    }

    fn start(&mut self) {
        println!("Starting module...");
        let args = self.args.clone();
        self.proc = Some(
            Command::new(self.cmd.as_str())
                .args(args)
                .spawn()
                .expect("failed to start module"),
        );
    }
    fn stop(&mut self) {
        // self.proc.wait().expect("Failed to wait on child");
        self.proc
            .as_mut()
            .unwrap()
            .wait()
            .expect("Failed to wait on child");
        println!("Stopped module successfully!");
    }
    fn print(&self) {
        println!("Module: {}", self.name);
        println!("Module Command: {}", self.cmd);
        print!("Module Args: ");
        for x in self.args.iter() {
            print!("{}, ", x);
        }
        print!("\n");
        println!("Module Process Running: {}", self.proc.is_some());
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
