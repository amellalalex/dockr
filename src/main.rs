use std::process::{Child, Command};

// TODO: Stop having to manually convert all str to Strings

fn main() {
    let mut acs = DockrModule::new("ACS", "./acs.out", vec!["JSON", "North", "HIGH"]);

    acs.print();

    acs.start();

    acs.print();

    acs.stop();

    acs.print();
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
