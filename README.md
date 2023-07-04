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
```
cargo add dunge
```

Then, let's create a new window to draw something in it:
```rust
// Import some types
use dunge::{
    handles::*,
    input::{Input, Key},
    shader::Shader,
    CanvasConfig, Context, Error, Frame, InitialState, Loop, MeshData, Model, Rgba, Vertex,
    WindowMode,
};

fn main() {
    dunge::make_window(InitialState::default())
        .run_blocking(CanvasConfig::default(), App::new)
        .into_panic();
}
```

`make_window` creates a new instance of `Canvas` type and sets up window properties, it allows us to handle an input from users. `run_blocking` runs our application by calling the constructor of it and passes the `Context` object there. Context uses for creation and updating of meshes, textures, instances etc.

To be able to draw something, you need to define a vertex type with the `Vertex` trait implementation and a shader type with the `Shader` trait implementation:
```rust
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
```rust
struct App {
    layer: LayerHandle<TriangleShader>,
    mesh: MeshHandle<Vert>,
    instance: InstanceHandle,
}

impl App {
    fn new(context: &mut Context) -> Self {
        // Create shader and layer
        let layer = {
            let shader: ShaderHandle<TriangleShader> = context.create_shader();
            context.create_layer(shader).expect("create layer")
        };

        // Create a mesh
        let mesh = {
            const VERTICES: [Vert; 3] = [
                Vert([-0.5, -0.5], [1., 0., 0.]),
                Vert([ 0.5, -0.5], [0., 1., 0.]),
                Vert([ 0.,   0.5], [0., 0., 1.]),
            ];
            let data = MeshData::from_verts(&VERTICES);
            context.create_mesh(&data)
        };

        // Create a model instance
        let instance = context.create_instances(&[Model::default()]);
        Self { layer, mesh, instance }
    }
}
```

To be able to pass the `App` in `run_blocking` we need to implement a `Loop` trait for it:
```rust
impl Loop for App {
    type Error = Error; // Define the error type

    // This calls once before every `render`
    fn update(&mut self, context: &mut Context, input: &Input) -> Result<(), Self::Error> {
        // You can update the context here. For example create and delete meshes.
        // Also you may want to handle an user's input here.
        Ok(())
    }

    // This calls every time the application needs to draw something in the window
    fn render(&self, frame: &mut Frame) -> Result<(), Self::Error> {
        frame
            .layer(self.layer)?
            .with_clear_color(Rgba::from_bytes([0, 0, 0, u8::MAX]))
            .with_clear_depth()
            .start()
            .draw(self.mesh, self.instance)
    }
}
```

Finally, let's run our code:
```
cargo run
```

Now you should see something like this:

![the triangle](./examples/triangle/screen.png)

## Examples
See [examples](https://github.com/nanoqsh/dunge/tree/main/examples) directory for more examples.
To build and run an example do:
```
cargo r -p <example_name>
```
