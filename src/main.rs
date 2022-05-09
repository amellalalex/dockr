use std::{
    error::Error,
    fs::File,
    io::BufReader,
    ops::Deref,
    process::{Child, Command},
};

use serde::{Deserialize, Serialize};

fn main() -> Result<(), Box<dyn Error>> {
    let acs = DockrModule::open("acs.json").expect("Failed to open module config file");
    let pay = DockrModule::open("pay.json").expect("Failed to open module config file");

    // Start modules
    let mut modules = vec![acs, pay];
    for x in &mut modules {
        x.start()?;
    }

    // Print modules
    for x in &mut modules {
        x.print();
    }

    // Stop modules
    for x in &mut modules {
        x.stop()?;
    }

    Ok(())
}

// type Result<T> = std::result::Result<T, DockrError>;

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

    fn open(path: &str) -> Result<DockrModule, Box<dyn Error>> {
        let file = File::open(path)?;
        let rdr = BufReader::new(file);
        let json: DockrJson = serde_json::from_reader(rdr)?;
        Ok(DockrModule::from(json))
    }

    fn start(&mut self) -> Result<(), Box<dyn Error>> {
        println!("Starting module...");
        self.proc = Some(
            Command::new(self.cmd.as_str())
                .args(self.args.deref())
                .spawn()?,
        );
        Ok(())
    }

    // fn start(&mut self) -> Result<(), Box<dyn Error>> {
    //     let child = Command::new(self.cmd.as_str())
    //         .args(self.args.deref())
    //         .spawn()?;
    //     Ok(())
    // }

    fn stop(&mut self) -> Result<(), Box<dyn Error>> {
        // self.proc.wait().expect("Failed to wait on child");
        match self.proc.as_ref() {
            Some(_) => println!("got some"),
            None => println!("got none"),
        }
        self.proc.as_mut().unwrap().wait()?;
        println!("Stopped module successfully!");
        Ok(())
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
