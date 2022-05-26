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
type DockrError = Box<dyn std::error::Error>;

#[derive(Debug)]
pub struct Module {
    name: String,
    pwd: Option<String>,
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
    /// Returns a blank module.
    pub fn new() -> Module {
        Module {
            name: String::new(),
            pwd: None,
            cmd: String::new(),
            args: Vec::new(),
            proc: None,
        }
    }

    pub fn create(name: &str, pwd: &str, cmd: &str, args: Vec<&str>) -> Module {
        Module {
            name: name.to_string(),
            pwd: Some(pwd.to_string()),
            cmd: cmd.to_string(),
            args: args.into_iter().map(|arg| arg.to_string()).collect(),
            proc: None,
        }
    }

    pub fn open(path: &str) -> Result<Module, DockrError> {
        let file = File::open(path)?;
        let rdr = BufReader::new(file);
        let json: DockrJson = serde_json::from_reader(rdr)?;
        let mut module = Module::from(json);
        if let Some(parent) = Path::new(path).parent() {
            if let Some(workdir) = parent.to_str() {
                module.pwd = Some(workdir.to_string());
            }
        }
        Ok(module)
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
    pub fn open_dir(path: &str) -> Result<Option<Module>, DockrError> {
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
                    .current_dir(self.pwd.as_deref().unwrap_or("."))
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
        self.stop_in(STOP_TIMEOUT)
    }
    pub fn stop_in(&mut self, timeout: u128) -> DockrResult {
        let start = Instant::now();
        if let Some(proc) = self.proc.as_mut() {
            log::debug!("Waiting on {} with intent to kill soon...", self.name);
            while let None = proc.try_wait()? {
                if start.elapsed().as_millis() >= timeout {
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

    /// Perform a one-shot `.start()` and `.wait()` on the module.
    /// Ideal for simple one-time executions.
    ///
    /// # Example
    /// ```no_run
    /// use dockr::Module;
    /// let mut module = Module::open("mod.json")?;
    /// module.run()?;
    /// ```
    ///
    /// # See also
    /// If you are interested in running a batch of modules, consider making a `Collection` and using `.run_all()`.
    pub fn run(&mut self) -> DockrResult {
        self.start()?;
        self.wait()?;
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
            pwd: None,
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

    /// Recursively search the specified directory for folders containing JSON config files.
    /// Returns a `Collection` containing the discovered modules.
    ///
    /// # Examples
    /// ```no_run
    /// use dockr::Collection;
    /// let mut coll = Collection::open_dir(".")?;
    /// ```
    ///
    /// # See also
    /// Use `Module::open_dir()` to only search one folder for a module.
    pub fn open_dir(path: &str) -> Result<Collection, DockrError> {
        log::debug!("Recursively searching {} directory...", path);
        let mut coll = Collection::new();
        let path = Path::new(path);
        for direntry in path.read_dir()? {
            if let Ok(entry) = direntry {
                if let (true, Some(dirpath)) = (entry.path().is_dir(), entry.path().to_str()) {
                    log::debug!("Found {} directory, attempting to open...", dirpath);
                    if let Ok(Some(module)) = Module::open_dir(dirpath) {
                        log::debug!(
                            "Adding {} to collection from {} directory!",
                            module.name,
                            dirpath
                        );

                        coll.push(module);
                    }
                }
            }
        }
        Ok(coll)
    }

    pub fn start_all(&mut self) -> DockrResult {
        if self.modules.len() == 0 {
            log::warn!("Attempting to .start_all() on an empty collection. Was this intentional?");
        }
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

    /// Run every module contained in the `Collection`. Ideal for one-shot executions of a batch of modules.
    ///
    /// # Example
    /// ```no_run
    /// use dockr::Collection;
    /// let mut coll = Collection::open_dir("modules")?;
    /// coll.run_all()?;
    /// ```
    ///
    /// # See also
    /// The `.run()` method in `Module` performs a `.start()` and a `.wait()`.
    /// The `.run_all()` method simply applies this to every module in a `Collection`.
    pub fn run_all(&mut self) -> DockrResult {
        self.start_all()?;
        self.wait_all()?;
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
