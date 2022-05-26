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

pub type DockrResult = Result<(), Box<dyn std::error::Error>>;
pub type DockrError = Box<dyn std::error::Error>;

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

    /// Create a module using user-defined function arguments.
    ///
    /// # Example
    /// ```no_run
    /// use dockr::Module;
    /// let mut module = Module::create(
    ///     "My Module",            // name
    ///     "modules/my_module",    // working directory
    ///     "./my_script.sh",       // command
    ///     vec!["arg1", "arg2"]    // args
    /// );
    /// ```
    ///
    /// # See also
    /// It is recommended to use JSON config files to load modules into Dockr.
    /// Check out `::open()`, `::open_dir()` and `Collection` for more.
    pub fn create(name: &str, pwd: &str, cmd: &str, args: Vec<&str>) -> Module {
        Module {
            name: name.to_string(),
            pwd: Some(pwd.to_string()),
            cmd: cmd.to_string(),
            args: args.into_iter().map(|arg| arg.to_string()).collect(),
            proc: None,
        }
    }

    /// Loads a module from a JSON config file.
    ///
    /// # Formatting
    /// Basic JSON formatting and structure applies.
    ///
    /// Every config file requires the following entries to be valid:
    /// 1. "name": the human-readable name of the module.
    /// 2. "cmd": the command to be executed upon `.start()`. *NOTE*: do not add arguments here.
    /// 3. "args": array of arguments to be passed to cmd. See example below.
    ///
    /// # Example
    /// ```json
    /// {
    ///     "name": "my module",
    ///     "cmd": "./my_script.sh",
    ///     "args": ["arg1", "arg2"]
    /// }
    /// ```
    ///
    /// # Additional notes
    /// - ALL paths are relative to the directory containing the JSON config file.
    /// - The `args` are passed to the child (if any) but are NOT directly pasted into
    /// the shell (as `system()` or `eval()` would in C/JS respectively).
    ///
    /// # See also
    /// The `::open_dir()` function operates very similarly to `::open()`, but
    /// instead accepts a directory containing a JSON config file.
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

    /// Searches for a valid JSON config file within the specified directory.
    ///
    /// Only expects to find 1 JSON config file. If multiple are present, only 1 module
    /// will be returned (likely to be the highest ranking filename alphabetically speaking).
    ///
    /// Although `::open()` would return an error in case of an invalid config file, `::open_dir()`
    /// cannot assume that a valid directory contains a valid config file. As such, the possibility
    /// of finding no such config file is handled by a return of `None`.
    ///
    /// The complications relating to dealing with `Some(Module)` return type are avoided when
    /// preferencially working with the `Collection::open_dir()` function. See the docs thereof
    /// for more info.
    ///
    /// # Examples
    /// ```no_run
    /// use dockr::Module;
    /// match Module::open_dir(".")? {
    ///     Some(module) => println!("Found module {} in dir", module.name),
    ///     None => println!("Could not locate module in dir"),
    /// };
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

    /// Launches the module `cmd` entry as a child process of the current binary.
    ///
    /// The `args` are passed to the child (if any) but are NOT directly pasted into
    /// the shell (as `system()` or `eval()` would in C/JS respectively).
    ///
    /// Later on, the module may be called to `.stop()` or be `.wait()` upon. See them
    /// for further elaboration.
    ///
    /// # Example
    /// ```no_run
    /// use dockr::Module;
    /// let mut module = Module::open("mod.json");
    /// module.start()?;
    /// ```
    ///
    /// # See also
    /// The way in which `args` are passed to the child can be explored in more detail under
    /// the Rust std::process documentation.
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

    /// Wait upon active module until termination.
    ///
    /// This method is BLOCKING in nature and will stop the current thread until execution
    /// is complete. For batch waiting, see `Collection.wait_all()`.
    ///
    /// # Example
    /// ```no_run
    /// use dockr::Module;
    /// let mut module = Module::open("mod.json")?;
    /// module.start()?;
    /// module.wait()?;
    /// ```
    ///
    /// # See also
    /// In most simple one-shot type of executions, the `.run()` method is
    /// conveniently composed of back-to-back `.start()` and `.wait()` methods.
    pub fn wait(&mut self) -> DockrResult {
        if let Some(proc) = self.proc.as_mut() {
            log::debug!("Waiting on {} ...", self.name);
            proc.wait()?;
            log::debug!("Done waiting on {} !", self.name);
        }
        Ok(())
    }

    /// Waits `STOP_TIMEOUT` ms for active module to terminate before killing the child process.
    ///
    /// Only to be used when a module must be terminated within a pre-determined timeframe.
    /// For a more graceful/passive closing, `.wait()` is available.
    ///
    /// # Example
    /// ```no_run
    /// use dockr::Module;
    /// let mut module = Module::open("mod.json")?;
    /// module.start()?;
    /// module.stop()?;
    /// ```
    ///
    /// # See also
    /// The `.stop_in()` method is available if a custom termination period is desired.
    pub fn stop(&mut self) -> DockrResult {
        self.stop_in(STOP_TIMEOUT)
    }

    /// Waits `timeout` ms for active module to terminate before killing the child process.
    ///
    /// Only to be used when a module must be terminated within a pre-determined timeframe.
    /// For a more graceful/passive closing, `.wait()` is available.
    ///
    /// # Example
    /// ```no_run
    /// use dockr::Module;
    /// let mut module = Module::open("mod.json")?;
    /// module.start()?;
    /// module.stop_in(1000)?; // in ms
    /// ```
    ///
    /// # See also
    /// The `.stop()` method is available if a standard termination period is desired.
    pub fn stop_in(&mut self, timeout: u128) -> DockrResult {
        let start = Instant::now();
        if let Some(proc) = self.proc.as_mut() {
            log::debug!("Waiting on {} with intent to kill soon...", self.name);
            while let None = proc.try_wait()? {
                if start.elapsed().as_millis() >= timeout {
                    // Force stop
                    log::debug!("Timeout elapsed, killing {} .", self.name);
                    proc.kill()?;
                    self.proc = None;
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
    /// Returns an empty collection
    pub fn new() -> Collection {
        Collection { modules: vec![] }
    }

    /// Creates a collection using user-defined arguments (not recommended).
    ///
    /// The `dockr::collection!()` macro is preferentially used.
    ///
    /// # Example
    /// ```no_run
    /// use dockr::Module;
    /// use dockr::Collection;
    ///
    /// let module1 = Module::open("my_mod1/conf.json")?;
    /// let module2 = Module::open("my_mod2/conf.json")?;
    /// let mut coll = Collection::create(vec![module1, module2]);
    /// ```
    ///
    /// # See also
    /// See the `dockr::collection!()` for user-defined collections.
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

    /// Starts all of the modules contained within the collection.
    ///
    /// This process occurs sequentially; first-added-first-ran basis.
    ///
    /// # Example
    /// ```no_run
    /// use dockr::Collection;
    /// let mut coll = Collection::open_dir("modules/")?;
    /// coll.start_all()?;
    /// ```
    ///
    /// # See also
    /// Other methods such as `.wait_all()`, `.run_all()` and `.stop_all()` pair nicely
    /// with each other.
    pub fn start_all(&mut self) -> DockrResult {
        if self.modules.len() == 0 {
            log::warn!("Attempting to .start_all() on an empty collection. Was this intentional?");
        }
        for x in self.modules.iter_mut() {
            x.start()?;
        }
        Ok(())
    }

    /// Sequentially waits for each module to terminate within `STOP_TIMEOUT` ms before killing.
    ///
    /// Every process is given the same timeout opportunity respectively; as such the maximum delay
    /// for complete termination may be expected to occur within `Collection.len() * STOP_TIMEOUT` ms.
    ///
    /// For a custom timeout duration to be specified, see the `.stop_all_in()` method.
    ///
    /// # Example
    /// ```no_stop
    /// use dockr::Collection;
    /// let mut coll = Collection::open_dir("modules/")?;
    /// coll.start_all()?;
    /// coll.stop_all()?;
    /// ```
    ///
    /// # See also
    /// For a more graceful termination, the `.wait_all()` method is available.
    pub fn stop_all(&mut self) -> DockrResult {
        self.stop_all_in(STOP_TIMEOUT)
    }

    /// Sequentially waits for each module to terminate within `timeout` ms before killing.
    ///
    /// Every process is given the same timeout opportunity respectively; as such the maximum delay
    /// for complete termination may be expected to occur within `Collection.len() * STOP_TIMEOUT` ms.
    ///
    /// For a standard timeout duration to be specified, see the `.stop_all()` method.
    ///
    /// # Example
    /// ```no_stop
    /// use dockr::Collection;
    /// let mut coll = Collection::open_dir("modules/")?;
    /// coll.start_all()?;
    /// coll.stop_all_in(500)?; // in ms
    /// ```
    ///
    /// # See also
    /// For a more graceful termination, the `.wait_all()` method is available.
    pub fn stop_all_in(&mut self, timeout: u128) -> DockrResult {
        for module in self.modules.iter_mut() {
            module.stop_in(timeout)?;
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

    /// Wait upon active modules until termination.
    ///
    /// This is a BLOCKING method and will stop the current thread until
    /// ALL modules have terminated on their own.
    ///
    /// Modules are sequentially waited on (via `.wait()`) until every active module
    /// has been found to have terminated.
    ///
    /// # Example
    /// ```no_run
    /// use dockr::Collection;
    /// let mut coll = Collection::open_dir("modules")?;
    /// coll.start_all()?;
    /// coll.wait_all()?;
    /// ```
    ///
    /// # See also
    /// If a limited timeout period for termination is desired, the `.stop_all()` method is
    /// available for this purpose.
    pub fn wait_all(&mut self) -> DockrResult {
        for module in self.modules.iter_mut() {
            module.wait()?;
        }
        Ok(())
    }

    /// Append a module to the collection (like a vec).
    pub fn push(&mut self, module: Module) {
        self.modules.push(module);
    }
}

/// Creates a user-defined collection of modules.
///
/// # Example
/// ```no_run
/// use dockr::Module;
/// let module1 = Module::open("my_mod1/conf.json")?;
/// let module2 = Module::open("my_mod2/conf.json")?;
/// let mut coll = dockr::collection!(module1, module2);
///```
///
/// # See also
/// It is possible to recursively scan directories for modules to create collections.
/// See `Collection::open_dir()` for more info.
#[macro_export]
macro_rules! collection {
    ($x:expr) => {
        Collection::create(vec![$x])
    };
    ($x:expr, $($y:expr),+) => (
        dockr::Collection::create(vec![$x, $($y),+])
    )
}
