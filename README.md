# Dunge
Simple and portable 3d render based on [WGPU](https://github.com/gfx-rs/wgpu).

## Getting Started
Let's render a colorful triangle for example. First, we need to add the dependency of dunge in the `Cargo.toml`:
```
cargo add dunge
```

Then, let's create a new window to draw something in it:
```rust
use dunge::*;

fn main() {
    dunge::make_window(InitialState::default())
        .run_blocking(App::new);
}
```

`make_window` creates a new instance of `Canvas` type and sets up window properties, it allows us to handle an input from users. `run_blocking` runs our application by calling the constructor of it and passes the `Context` object there. Context uses for creation and updating of meshes, textures, views, instances etc.

The `App` is our application type, we need to create it:
```rust
struct App {
    instance: InstanceHandle,
    mesh: MeshHandle<ColorVertex>,
    view: ViewHandle,
}

impl App {
    fn new(context: &mut Context) -> Self {
        // Create a model instance
        let instance = context.create_instances([Position::default()]);

        // Create a mesh
        let mesh = {
            // Vertex data describes a position in XYZ coordinates and color in RGB per vertex:
            const VERTICES: [ColorVertex; 3] = [
                ColorVertex { pos: [-0.5, -0.5, 0.], col: [1., 0., 0.] },
                ColorVertex { pos: [0.5,  -0.5, 0.], col: [0., 1., 0.] },
                ColorVertex { pos: [0.,    0.5, 0.], col: [0., 0., 1.] },
            ];
            // Indices of triangle vetrices:
            const INDICES: [u16; 3] = [0, 1, 2];

            let data = MeshData::new(&VERTICES, &[INDICES]).unwrap();
            context.create_mesh(data)
        };

        // Create the view
        let view = context.create_view::<Perspective>(View::default());
        Self { instance, mesh, view }
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
        // Draw a new layer. Use `color_layer` for this since our triangle has color vertices
        let mut layer = frame
            .color_layer()
            .with_clear_color(Srgba([0, 0, 0, 255]))
            .with_clear_depth()
            .start();

        layer.bind_view(self.view)?;
        layer.bind_instance(self.instance)?;
        layer.draw(self.mesh)
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
