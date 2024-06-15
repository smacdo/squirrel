# Squirrel
Squirrel is a project is to facilitate experimentation with WGPU and how gameplay
could be implemented in Rust. It's messy, adhoc, full of TODO notes and not in any way
ready for production use.

If you see any obvious errors _please_ let me know by opening an issue!

# Windows/Mac/Linux desktop
Nothing special is required for running on traditional desktop machines. Simply
run the follow commands to view the demo scene:

```
$ cargo build
$ cargo run
```

# Web assembly
To build this project for web assembly perform the following steps:
```
$ cargo install wasm-pack
$ wasm-pack build . --target web
```

Once built you will need to set up a web server that is pointed to the `pkg`
subfolder. Here is a fast way of accomplishing that with microserver:

```
$ cargo install microserver 
$ microserver
```

Open up your web browser to `http://localhost:9090` and you should see the
demo scene!

# TODO
- CI profile to test windows, linux, mac builds
- CD profile to build webasm and then publish somewhere for viewing on the web