<div align="center">
    <h1>Dunge</h1>
    <p>
        Simple and portable 3d render based on <a href="https://github.com/gfx-rs/wgpu">WGPU</a>
    </p>
    <p>
        <a href="https://crates.io/crates/dunge"><img src="https://img.shields.io/crates/v/dunge.svg"></img></a>
        <a href="https://docs.rs/dunge"><img src="https://docs.rs/dunge/badge.svg"></img></a>
        <a href="https://github.com/nanoqsh/dunge/actions"><img src="https://github.com/nanoqsh/dunge/workflows/ci/badge.svg"></img></a>
    </p>
</div>

## Features
* Simple and flexible API
* Customizable vertices, groups and instances
* Shader code described as a single rust function
* High degree of typesafety with minimal runtime checks
* Desktop, WASM and (later) Android support
* Optional built-in window and event loop

## Application area
Currently the library is for personal use only. Although, over time I plan to stabilize API so that someone could use it for their tasks.

## Getting Started
To start using the library add it to your project:
```sh
cargo add dunge -F winit
```
Specify the `winit` feature if you need to create a windowed application. Although this is not necessary, for example, you can simply draw a scene directly to the image in RAM.

So what if you want to draw something on the screen? Let's say you want to draw a simple colored triangle. Then start by creating a vertex type. To do this, derive the `Vertex` trait for your struct:
```rust
use dunge::{
    glam::{Vec2, Vec3},
    prelude::*,
};

// Create a vertex type
#[repr(C)]
#[derive(Vertex)]
struct Vert {
    pos: Vec2,
    col: Vec3,
}
```

To render something on GPU you need to program a shader. In dunge you can do this via a normal (almost) rust function:
```rust
// Create a shader program
let triangle = |vert: sl::InVertex<Vert>| {
    // Describe the vertex position:
    // Take the vertex data as vec2 and expand it to vec4
    let place = sl::vec4_concat(vert.pos, sl::vec2(0., 1.));

    // Then describe the vertex color:
    // First you need to pass the color from
    // vertex shader stage to fragment shader stage
    let fragment_col = sl::fragment(vert.col);

    // Now create the final color by adding an alpha value
    let color = sl::vec4_with(fragment_col, 1.);

    // As a result, return a program that describes how to
    // compute the vertex position and the fragment color
    sl::Render { place, color }
};
```

As you can see from the snippet, the shader requires you to provide two things: the position of the vertex on the screen and the color of each fragment/pixel. The result is a `triangle` function, but if you ask for its type in the IDE you may notice that it is more complex than usual:

`impl Fn(InVertex<Vert>) -> Render<Ret<Compose<Ret<ReadVertex, Vec2<f32>>, Ret<NewVec<(f32, f32), Vs>, Vec2<f32>>>, Vec4<f32>>, Ret<Compose<Ret<Fragment<Ret<ReadVertex, Vec3<f32>>>, Vec3<f32>>, f32>, Vec4<f32>>>`

That's because this function doesn't actually compute anything. It is needed only to describe the method for computing what we need on GPU. During shader instantiation, this function is used to compile an actual shader. However, this saves us from having to write the shader in wgsl and allows to typecheck at compile time. For example, dunge checks that a vertex type in a shader matches with a mesh used during rendering. It also checks types inside the shader itself.

Now let's create the dunge context and other necessary things:
```rust
// Create the dunge context
let cx = dunge::context().await?;

// You can use the context to manage dunge objects.
// Create a shader instance
let shader = cx.make_shader(triangle);
```

You may notice the context creation requires async. This is WGPU specific, so you will have to add your favorite async runtime in the project.

Also create a triangle mesh that we're going to draw:
```rust
// Create a mesh from vertices
let mesh = {
    let data = const {
        MeshData::from_verts(&[
            Vert { pos: Vec2::new(-0.5, -0.5), col: Vec3::new(1., 0., 0.) },
            Vert { pos: Vec2::new( 0.5, -0.5), col: Vec3::new(0., 1., 0.) },
            Vert { pos: Vec2::new( 0. ,  0.5), col: Vec3::new(0., 0., 1.) },
        ])
    };

    cx.make_mesh(&data)
};
```

Now to run the application we need two last things: handlers. One `Update` that is called every time before rendering and is used to control the render objects and manage the main [event loop](https://en.wikipedia.org/wiki/Event_loop):
```rust
// Describe the `Update` handler
let upd = |ctrl: &Control| {
    for key in ctrl.pressed_keys() {
        // Exit by pressing escape key
        if key.code == KeyCode::Escape {
            return Then::Close;
        }
    }

    // Otherwise continue running
    Then::Run
};
```
We don't do anything special here, we just check is <kbd>Esc</kbd> pressed and end the main loop if necessary. Note that this handler is only needed to use a window with the `winit` feature.

Second `Draw` is used directly to draw something in the final frame:
```rust
// Create a layer for drawing a mesh on it
let layer = cx.make_layer(&shader, view.format());

// Describe the `Draw` handler
let draw = move |mut frame: Frame| {
    use dunge::color::Rgba;

    // Create a black RGBA background
    let bg = Rgba::from_bytes([0, 0, 0, !0]);

    frame
        // Set a layer to draw on it
        .set_layer(&layer, bg)
        // The shader has no bindings, so call empty bind
        .bind_empty()
        // And finally draw the mesh
        .draw(&mesh);
};
```

> **Note:** To create a layer we need to know the window format. It would be possible to guess it, but it is better to get it directly from a view object. You can get the view from a special `make` helper, which will call a closure when the handler is initialized and passes the necessary data to it.

Now you can join two steps in one hander and run the application and see the window:
```rust
let make_handler = |cx: &Context, view: &View| {
    let upd = |ctrl: &Control| {/***/};
    let draw = move |mut frame: Frame| {/***/};
    dunge::update(upd, draw)
};

// Run the window with handlers
dunge::window().run_local(cx, dunge::make(make_handler))?;
```

<div align="center">
    <img src="examples/window/s.png">
</div>

You can see full code from this example [here](https://github.com/nanoqsh/dunge/tree/main/examples/window) and run it using:
```sh
cargo run -p window
```

## Examples
For more examples using the window, see the [examples](https://github.com/nanoqsh/dunge/tree/main/examples) directory. To build and run an example do:
```sh
cargo run -p <example_name>
```

To build and run a wasm example:
```sh
cargo x build <example_name>
cargo x serve <example_name>
```

If [`wasm-pack`](https://github.com/rustwasm/wasm-pack) is already installed on the system, the build script will find it and use it to compile a wasm artifact. Otherwise, `wasm-pack` will be installed locally. To prevent this behavior add the `no-install` flag:
```sh
cargo x --no-install build <example_name>
```

Eventually it will start a local server and you can open http://localhost:3000 in your browser to see the application running. Only [WebGPU](https://gpuweb.github.io/gpuweb/) backend is supported for the web platform, so make sure your browser supports it.

Also see the [test](https://github.com/nanoqsh/dunge/tree/main/dunge/tests) directory for small examples of creation a single image.
