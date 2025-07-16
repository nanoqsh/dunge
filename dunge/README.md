<div align="center">
    <h1>dunge</h1>
    <p>
        Typesafe and portable 3d render based on <a href="https://github.com/gfx-rs/wgpu">wgpu</a>
    </p>
    <p>
        <a href="https://crates.io/crates/dunge"><img src="https://img.shields.io/crates/v/dunge.svg"></img></a>
        <a href="https://docs.rs/dunge"><img src="https://docs.rs/dunge/badge.svg"></img></a>
        <a href="https://github.com/nanoqsh/dunge/actions"><img src="https://github.com/nanoqsh/dunge/workflows/ci/badge.svg"></img></a>
    </p>
</div>

## Features

* Typesafe and flexible API
* Customizable vertices, groups and instances
* Render and compute shaders described as a single rust function
* High degree of typesafety with minimal runtime checks
* Desktop and WASM support
* Optional windowing extension

## Application area

Currently the library is for personal use only. Although, over time I plan to stabilize API so that someone could use it for their tasks

## Getting Started

To start using the library, add the `dunge` crate to your project's dependencies. If you need the windowing extension, add the `dunge_winit` crate only - it directly re-exports everything from the `dunge` crate, so there's no need to include both

```sh
cargo add dunge_winit
```

You can also opt out of window system support and render the scene directly into an image in RAM

So what if you want to draw something on the screen? Let's say you want to draw a simple colored triangle. Then start by creating a vertex type. To do this, derive the `Vertex` trait for your struct

```rust
use dunge_winit::{
    glam::{Vec2, Vec3},
    prelude::*,
};

// create a vertex type
#[repr(C)]
#[derive(Vertex)]
struct Vert {
    pos: Vec2,
    col: Vec3,
}
```

To render something on GPU you need to program a shader. In dunge you can do this via a normal (almost) rust function

```rust
// create a shader program
let triangle = |vert: sl::InVertex<Vert>| {
    // describe the vertex position:
    // take the vertex data as vec2 and expand it to vec4
    let place = sl::vec4_concat(vert.pos, sl::vec2(0., 1.));

    // then describe the vertex color:
    // first you need to pass the color from
    // vertex shader stage to fragment shader stage
    let fragment_col = sl::fragment(vert.col);

    // now create the final color by adding an alpha value
    let color = sl::vec4_with(fragment_col, 1.);

    // as a result, return a program that describes how to
    // compute the vertex position and the fragment color
    sl::Render { place, color }
};
```

As you can see from the snippet, the shader requires you to provide two things: the position of the vertex on the screen and the color of each fragment/pixel. The result is a `triangle` function, but if you ask for its type in the IDE you may notice that it is more complex than usual

`impl Fn(InVertex<Vert>) -> Render<Ret<Compose<Ret<ReadVertex, Vec2<f32>>, Ret<NewVec<(f32, f32), Vs>, Vec2<f32>>>, Vec4<f32>>, Ret<Compose<Ret<Fragment<Ret<ReadVertex, Vec3<f32>>>, Vec3<f32>>, f32>, Vec4<f32>>>`

That's because this function doesn't actually compute anything. It is needed only to describe the method for computing what we need on GPU. During shader instantiation, this function is used to compile an actual shader. However, this saves us from having to write the shader in wgsl and allows to typecheck at compile time. For example, dunge checks that a vertex type in a shader matches with a mesh used during rendering. It also checks types inside the shader itself

Now let's create the dunge context and other necessary things

```rust
// create the dunge context
let cx = dunge::context().await?;

// you can use the context to manage dunge objects.
// create a shader instance
let shader = cx.make_shader(triangle);
```

You may notice that context creation requires async. Indeed, dunge is fundamentally **async**: scheduling GPU workloads, managing windows, handling real-time IO and working with timings - all of these are inherently asynchronous operations. This API also makes it easy to integrate existing ecosystem components into your project. For example, you can effortlessly add asynchronous network IO handling - whether you're targeting a desktop system or a browser runtime

That's why dunge includes its own asynchronous runtime. If you're not using the `dunge_winit` windowing extension and simply want to work with the GPU, you can use the `dunge::block_on` function - it allows you to run an async routine on desktop platforms. For windowed applications, use `dunge_winit::winit::block_on` or `dunge_winit::winit::try_block_on`, which handle the event loop of a windowed app. A minimal usage example with error handling might look like this:

```rust
async fn run(control: Control) -> Result<(), Error> {
    // full the application logic here
}

fn main() {
    if let Err(e) = dunge_winit::winit::try_block_on(run) {
        eprintln!("error: {e}");
    }
}
```

Also create a triangle mesh that we're going to draw

```rust
// create a mesh from vertices
let mesh = {
    const DATA: MeshData<'static, Vert> = MeshData::from_verts(&[
        Vert { pos: Vec2::new(-0.5, -0.5), col: Vec3::new(1., 0., 0.) },
        Vert { pos: Vec2::new(0.5, -0.5),  col: Vec3::new(0., 1., 0.) },
        Vert { pos: Vec2::new(0., 0.5),    col: Vec3::new(0., 0., 1.) },
    ]);

    cx.make_mesh(&DATA)
};
```

We need to create the application window and a layer - the surface onto which the final scene will be rendered. The layer must use the same color format as the window, so we'll query the required format directly. Additionally, the layer needs to know which shader to use for rendering, so we'll specify our shader as well

```rust
// the control object is created from the `(try_)block_on` function
let window = control.make_window(&cx, Attributes::default()).await?;
let layer = cx.make_layer(&shader, window.format());
```

Now we can create the render loop. It's described in a simple and straightforward way: it's literally a loop where we wait for the window's redraw event, schedule the rendering of the layer with a triangle mesh, and present the final result

```rust
// specify a color of render background, it will be black
let bg = layer.format().rgb_from_bytes([0; 3]);
let render = async {
    loop {
        // wait for window is going to redraw
        let redraw = window.redraw().await;

        // schedule the render
        cx.shed(|s| {
            s.render(&redraw, bg).layer(&layer).draw(&mesh);
        })
        .await;

        // present rendered image on the window
        redraw.present();
    }
};

// render is an infinite future, so we can await on it
render.await;
```

That's it - you can now run the program and see a beautiful colorful triangle on the screen!

However, there's one issue you may have noticed earlier: our render future runs indefinitely, which means there's currently no way to gracefully shut down the application. What happens if a user closes the window? Nothing - because we arent tracking that event

Fortunately, this is easy to fix. We'll need to use one of the async utility libraries: `futures`, `futures-lite` or `futures-concurrency` - feel free to pick whichever you prefer. For this example, we'll use `futures-concurrency`, which provides a convenient [`race`](https://docs.rs/futures-concurrency/latest/futures_concurrency/future/trait.Race.html#tymethod.race) function that allows you to concurrently await multiple futures - exactly what we need:

```rust
use futures_concurrency::prelude::*;

let render = async {/**/};

// wait for close requested event
let close = window.close_requested();

// race two futures
// since render will never finish, this race will finish
// as soon as close requested event will be emitted
(render, close).race().await;
```

You can see full code (with additions) from this example [here](https://github.com/nanoqsh/dunge/tree/main/examples/window/src/main.rs) and run it using:
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
