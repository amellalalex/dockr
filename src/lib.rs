use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    io::BufReader,
    ops::Deref,
    path::Path,
    process::{Child, Command},
    time::Instant,
};

// TODO: Put settings into JSON file
// Settings
const STOP_TIMEOUT: u128 = 3000; // in milliseconds

type DockrResult = Result<(), Box<dyn std::error::Error>>;

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

    /// Searches for a valid module JSON config file within the specified directory.
    /// Only expects to find 1 JSON file. If multiple are present, only 1 module will be returned.
    ///
    /// # Examples
    /// ```no_run
    /// use dockr::Module;
    /// let mut module = Module::open_dir(".")?;
    /// ```
    ///
    /// # See also
    /// Use `Module::open()` if the JSON config file is known.
    pub fn open_dir(path: &str) -> Result<Option<Module>, Box<dyn std::error::Error>> {
        let dirpath = Path::new(path);
        if dirpath.is_dir() {
            for direntry in dirpath.read_dir()? {
                if let Ok(entry) = direntry {
                    if let Some(filepath) = entry.path().to_str() {
                        // Check for JSON extension
                        if filepath.contains(".json") {
                            // Try to open it
                            if let Ok(module) = Module::open(filepath) {
                                return Ok(Some(module));
                            }
                        }
                    }
                }
            }
        }

        Ok(None)
    }

    pub fn start(&mut self) -> DockrResult {
        if let None = self.proc {
            log::debug!("Starting {} ...", self.name);
            self.proc = Some(
                Command::new(self.cmd.as_str())
                    .args(self.args.deref())
                    .spawn()?,
            );
            log::debug!("Successfully started {} !", self.name);
        }
        Ok(())
    }

    pub fn wait(&mut self) -> DockrResult {
        if let Some(proc) = self.proc.as_mut() {
            log::debug!("Waiting on {} ...", self.name);
            proc.wait()?;
            log::debug!("Done waiting on {} !", self.name);
        }
        Ok(())
    }

    pub fn stop(&mut self) -> DockrResult {
        let start = Instant::now();
        if let Some(proc) = self.proc.as_mut() {
            log::debug!("Waiting on {} with intent to kill soon...", self.name);
            while let None = proc.try_wait()? {
                if start.elapsed().as_millis() >= STOP_TIMEOUT {
                    // Force stop
                    log::debug!("Timeout elapsed, killing {} .", self.name);
                    proc.kill()?;
                    break;
                }
            }
            log::debug!("Done stopping {} !", self.name);
        }
        Ok(())
    }

    pub fn run(&mut self) -> DockrResult {
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
    pub fn new() -> Collection {
        Collection { modules: vec![] }
    }

    pub fn create(modules: Vec<Module>) -> Collection {
        Collection { modules }
    }

    pub fn open(path: &str) -> Result<Collection, Box<dyn std::error::Error>> {
        let path = Path::new(path);
        let mut modules: Vec<Module> = Vec::new();
        for entry in path.read_dir()? {
            if let Ok(entry) = entry {
                // Found a folder
                if entry.path().is_dir() {
                    // Recurse one level down
                    for recursed_entry in entry.path().read_dir()? {
                        // Look for JSON file
                        if let Some(filepath) = recursed_entry?.path().to_str() {
                            if filepath.contains(".json") {
                                // Try to open it
                                if let Ok(potential_module) = Module::open(filepath) {
                                    modules.push(potential_module);
                                }
                            }
                        }
                    }
                // Found a file
                } else if let Some(filepath) = entry.path().to_str() {
                    if filepath.contains(".json") {
                        // Try to open it
                        if let Ok(potential_module) = Module::open(filepath) {
                            modules.push(potential_module);
                        }
                    }
                }
            }
        }
        Ok(Collection::new())
    }

    pub fn start_all(&mut self) -> DockrResult {
        for x in self.modules.iter_mut() {
            x.start()?;
        }
        Ok(())
    }

    pub fn stop_all(&mut self) -> DockrResult {
        for module in self.modules.iter_mut() {
            module.stop()?;
        }
        Ok(())
    }

    pub fn run_all(&mut self) -> DockrResult {
        self.start_all()?;
        self.stop_all()?;
        Ok(())
    }

    pub fn wait_all(&mut self) -> DockrResult {
        for module in self.modules.iter_mut() {
            module.wait()?;
        }
        Ok(())
    }

    pub fn push(&mut self, module: Module) {
        self.modules.push(module);
    }
}

#[macro_export]
macro_rules! collection {
    ($x:expr) => {
        Collection::create(vec![$x])
    };
    ($x:expr, $($y:expr),+) => (
        dockr::Collection::create(vec![$x, $($y),+])
    )
}
