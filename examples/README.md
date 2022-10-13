This directory contains examples of dunge applications.

You can build and run any example by:
```
cd examples
cargo r -p <example>
```

To run an wasm example do:
```
cd examples
cargo r -p serve -- <wasm_example>
```

It builds a web server and runs the wasm example in it. Then you can open http://127.0.0.1:3000 in a browser to show the example.
