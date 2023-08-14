# Dunge

<div align="center">
    <p>
        Simple and portable 3d render based on <a href="https://github.com/gfx-rs/wgpu">WGPU</a>.
    </p>
    <p>
        <a href="https://crates.io/crates/dunge"><img src="https://img.shields.io/crates/v/dunge.svg"></img></a>
        <a href="https://docs.rs/dunge"><img src="https://docs.rs/dunge/badge.svg"></img></a>
        <a href="https://github.com/nanoqsh/dunge/actions"><img src="https://github.com/nanoqsh/dunge/workflows/ci/badge.svg"></img></a>
    </p>
</div>

## Features
* Simple but flexible API
* Desktop, WASM and Android support
* Customizable vertices and shaders
* Pixel perfect render with custom layers
* Light sources and light spaces

## Application area
The library is for personal use only. I use it to create my applications and I make API suitable exclusively for my problems. Perhaps in the future API will settle down and the library will be self-sufficient for other applications.

## Getting Started
Let's render a colorful triangle for example. First, we need to add the dependency of dunge in the `Cargo.toml`:
```sh
cargo add dunge
```

Then, let's create a new window to draw something in it:
```rust,ignore
use dunge::{CanvasConfig, InitialState};

fn main() -> ! {
    dunge::make_window(InitialState::default())
        .run_blocking(CanvasConfig::default(), App::new)
        .into_panic()
}
```

`make_window` creates a new instance of `Canvas` type and sets up window properties, it allows us to handle an input from users. `run_blocking` runs our application by calling the constructor of it and passes the `Context` object there. Context uses for creation and updating of meshes, textures, instances etc.

To be able to draw something, you need to define a vertex type with the `Vertex` trait implementation and a shader type with the `Shader` trait implementation:
```rust,ignore
use dunge::{Shader, Vertex};

// Instead of manually implementing the trait, use a derive macro.
// Note the struct must have the `repr(C)` attribute
#[repr(C)]
#[derive(Vertex)]
struct Vert(#[position] [f32; 2], #[color] [f32; 3]);

struct TriangleShader;
impl Shader for TriangleShader {
    type Vertex = Vert; // Specify the vertex type 
}
```

The `App` is our application type, we need to create it:
```rust,ignore
use dunge::{Context, Layer, Mesh, MeshData};

struct App {
    layer: Layer<TriangleShader>,
    mesh: Mesh<Vert>,
}

impl App {
    fn new(context: &mut Context) -> Self {
        // Create a layer
        let layer = context.create_layer();

        // Create a mesh
        let mesh = {
            let data = MeshData::from_verts(&[
                Vert([-0.5, -0.5], [1., 0., 0.]),
                Vert([ 0.5, -0.5], [0., 1., 0.]),
                Vert([ 0.,   0.5], [0., 0., 1.]),
            ]);

            context.create_mesh(&data)
        };

        Self { layer, mesh }
    }
}
```

To be able to pass the `App` in `run_blocking` we need to implement a `Loop` trait for it:
```rust,ignore
use dunge::{Input, Frame, Loop, Rgba};

impl Loop for App {
    // This calls once before every `render`
    fn update(&mut self, context: &mut Context, input: &Input) {
        // You can update the context here. For example create and delete meshes.
        // Also you may want to handle an user's input here.
    }

    // This calls every time the application needs to draw something in the window
    fn render(&self, frame: &mut Frame) {
        frame
            .layer(&self.layer)
            .with_clear_color(Rgba::from_bytes([0, 0, 0, u8::MAX]))
            .start()
            .draw(&self.mesh);
    }
}
```

Finally, let's run our code:
```sh
cargo run
```

Now you should see something like [this](https://github.com/nanoqsh/dunge/tree/main/examples/triangle/screen.png)

![triangle](./examples/triangle/screen.png)

## Examples
See [examples](https://github.com/nanoqsh/dunge/tree/main/examples) directory for more examples.
To build and run an example do:
```sh
cargo r -p <example_name>
```
