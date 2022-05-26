# What just happened?

This page will go over a breakdown of the previous example.

## The DockrModule

In the first codeblock, a mutable variable containing a `DockrModule` was instantiated:
```rust
let mut my_module = DockrModule::create(
    "My Module",
    "echo",
    vec!["hello dockr!"]
);
```

We can make more sense of the arguments we passed to `create()` by taking a look at the inner components of the `DockrModule` struct:

```rust
pub struct DockrModule {
    name: String,
    cmd: String,
    args: Vec<String>,
    proc: Option<Child>,
}
```

That is, every module needs a name, a command to execute, a set of arguments and an optional `Child` placeholder (more on this later).

For now, the main takeaway is that the `create()` function simply returns a `DockrModule` object that was constructed using the provided hardcoded arguments.

## The arguments

The `name` argument is self-explanatory: it is a human-readable identifier for the module. In case of notice, warning and error logging, the `name` will be the identifier of choice.

The `cmd` argument accepts the command-line argument or program to be executed. Do note this is quite unlike running `os.system()` in python or `system()` in C whereby the provided string is directly pasted into the shell. Rather, `cmd` must only contain the very first segment of a command, be it the name of the script, program or bash command itself.

In case of our previous example, `echo` was the bash command of choice. 

Onto the `args` argument. Here a vector of arguments to be passed to the `cmd` script/program is needed. Although arguments technically optional, a vector *of some kind* must still be passed for the function to work. If no arguments are required, an empty `vec![]` would take the argument's place. 

Back to our example, the provided argument was `"hello dockr!"`. The combined `cmd` and argument would yield `echo hello\ dockr!`. 