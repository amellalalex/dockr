use std::{
    fs::File,
    io::BufReader,
    process::{Child, Command},
};

use serde::{Deserialize, Serialize};

// TODO: Stop having to manually convert all str to Strings

fn main() {
    let mut acs = DockrModule::new("ACS", "./acs.out", vec!["JSON", "North", "HIGH"]);
    acs.print();
    acs.start();
    acs.stop();

    let json = r#"
    {
        "name": "newAcs",
        "cmd": "./a.out",
        "args": [
            "JSON",
            "North",
            "High"
        ]
    }
    "#;

    // let acs_json: DockrJson = serde_json::from_str(json).unwrap();
    let file = File::open("acs.json").expect("Failed to open JSON file");
    let rdr = BufReader::new(file);
    let acs_json: DockrJson = serde_json::from_reader(rdr).expect("Failed to parse JSON from file");
    let new_acs = DockrModule::from(acs_json);
    new_acs.print();
}

struct DockrModule {
    name: String,
    cmd: String,
    args: Vec<String>,
    proc: Option<Child>,
}

impl DockrModule {
    fn new(name: &str, cmd: &str, args: Vec<&str>) -> DockrModule {
        DockrModule {
            name: String::from(name),
            cmd: String::from(cmd),
            args: args.into_iter().map(|s| String::from(s)).collect(),
            proc: None,
        }
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

// impl From<&str> for DockrModule {
//     fn from(path: &str) -> Self {
//         let file = File
//     }
// }

impl From<File> for DockrModule {
    fn from(file: File) -> Result<Self, serde_json::Error> {
        let rdr = BufReader::new(file);
        let json: DockrJson = serde_json::from_reader(rdr)?;
        DockrModule::from(json);
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct DockrJson {
    name: String,
    cmd: String,
    args: Vec<String>,
}
