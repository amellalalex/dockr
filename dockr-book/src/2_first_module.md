# Making your first module

Under `main.rs`, create a new `DockrModule` using the associated `create()` function:

```rust
use dockr::DockrModule;

fn main() {
    let mut my_module = DockrModule::create(
        "My Module",
        "echo",
        vec!["hello dockr!"]
    );
}
```

Then, launch the module using `.run()` method:

```rust
fn main() {
    let mut my_module = DockrModule::create(
        "My Module",
        "echo",
        vec!["hello dockr!"]
    );

    my_module.run();
}
```

That's it! You now have a working module.