use std::process::{Command, Child};

// TODO: Stop having to manually convert all str to Strings

fn main() {
    let mut acs = DockrModule::new(
        123, 
        "./acs.out".to_string(), 
        vec!["12:00:00".to_string(), "JSON".to_string(), "North".to_string()]
    );
    acs.start();
    acs.stop();
}

struct DockrModule {
    id: u64,
    cmd: String,
    args: Vec<String>,
    proc: Option<Child>,
}

impl DockrModule {
    fn new(id: u64, cmd: String, args: Vec<String>) -> DockrModule {
        DockrModule { 
            id, 
            cmd, 
            args, 
            proc: None, 
        }
    }
    fn start(&mut self) {
        println!("Starting module...");
        let args = self.args.clone();
        self.proc = Some(Command::new(self.cmd.as_str())
                .args(args)
                .spawn()
                .expect("failed to start module"));
    }
    fn stop(&mut self) {
        // self.proc.wait().expect("Failed to wait on child");
        self.proc.as_mut().unwrap().wait().expect("Failed to wait on child");
        println!("Stopped module successfully!");
    }
}
