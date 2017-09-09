#[macro_use]
extern crate gfx;
extern crate gfx_window_glutin;
extern crate glutin;
extern crate clap;
extern crate math;


use gfx::traits::FactoryExt;
use gfx::Device;
use glutin::GlContext;

use std::fs::File;
use std::io::prelude::*;
use clap::{Arg, App};

use math::builder::Builder;
use math::vm::glsl::glsl;

pub type ColorFormat = gfx::format::Srgba8;
pub type DepthFormat = gfx::format::DepthStencil;

gfx_defines!{
    vertex Vertex {
        pos: [f32; 2] = "v_pos",
    }
    constant Time {
        time: f32 = "u_time",
    }

    pipeline pipe {
        vbuf: gfx::VertexBuffer<Vertex> = (),
        time: gfx::Global<f32> = "u_time",
        out: gfx::RenderTarget<ColorFormat> = "final_col",
    }
}

const QUAD: [Vertex; 6] = [
    Vertex { pos: [ -1.0, -1.0 ]},
    Vertex { pos: [  1.0, -1.0 ]},
    Vertex { pos: [  1.0,  1.0 ]},

    Vertex { pos: [ -1.0, -1.0 ]},
    Vertex { pos: [  1.0,  1.0 ]},
    Vertex { pos: [ -1.0,  1.0 ]},
];

const CLEAR_COLOR: [f32; 4] = [0.1, 0.2, 0.3, 1.0];

fn get_file_content(path: &str) -> String {
    let mut s = String::new();
    File::open(path).unwrap().read_to_string(&mut s).unwrap();
    s
}

pub fn main() {
    // Parse expression from command line args
    let options = App::new("Math")
        .arg(Arg::with_name("EXPR")
            .help("Mathematical expression to display")
            .required(true)
            .index(1))
        .get_matches();
    let expr = options.value_of("EXPR").unwrap();
    let builder = Builder::new();
    let tokens = builder.parse(expr).unwrap();
    let (vert, frag) = glsl(tokens);
    println!("Vert:\n{}\n\nFrag:\n{}\n", vert, frag);

    // Set up the rest

    let mut elapsed_time = 0.0;

    let mut events_loop = glutin::EventsLoop::new();
    let window_config = glutin::WindowBuilder::new()
        .with_title("Math".to_string())
        .with_dimensions(1024, 768);
    let context = glutin::ContextBuilder::new()
        .with_vsync(true);
    let (window, mut device, mut factory, main_color, mut main_depth) =
        gfx_window_glutin::init::<ColorFormat, DepthFormat>(window_config, context, &events_loop);
    let mut encoder: gfx::Encoder<_, _> = factory.create_command_buffer().into();
    let pso = factory.create_pipeline_simple(
        vert.as_bytes(),
        frag.as_bytes(),
        pipe::new()
    ).unwrap();
    let (vertex_buffer, slice) = factory.create_vertex_buffer_with_slice(&QUAD, ());
    let mut data = pipe::Data {
        vbuf: vertex_buffer,
        time: 0.0,
        out: main_color
    };

    let mut running = true;
    while running {
        // fetch events
        events_loop.poll_events(|event| {
            if let glutin::Event::WindowEvent { event, .. } = event {
                match event {
                    glutin::WindowEvent::KeyboardInput {
                        input: glutin::KeyboardInput {
                            virtual_keycode: Some(glutin::VirtualKeyCode::Escape), ..
                        }, ..
                    }
                    | glutin::WindowEvent::Closed
                        => running = false,
                    glutin::WindowEvent::Resized(width, height) => {
                        window.resize(width, height);
                        gfx_window_glutin::update_views(&window, &mut data.out, &mut main_depth);
                    },
                    _ => (),
                }
            }
        });

        // Update uniforms
        elapsed_time += 10.0;
        data.time = elapsed_time;

        // draw a frame
        encoder.clear(&data.out, CLEAR_COLOR);
        encoder.draw(&slice, &pso, &data);
        encoder.flush(&mut device);
        window.swap_buffers().unwrap();
        device.cleanup();
    }
}
