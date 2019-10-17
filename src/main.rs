use luminance::context::GraphicsContext as _;
use luminance::render_state::RenderState;
use luminance::shader::program::{Program, Uniform};
use luminance::tess::{Mode, TessBuilder};
use luminance_derive::{Semantics, UniformInterface, Vertex};
use luminance_glfw::{Action, GlfwSurface, Key, Surface, WindowDim, WindowEvent, WindowOpt};
use rosc::{encoder, OscMessage, OscPacket};
use std::mem::drop;
use std::net::{SocketAddr, UdpSocket};
use std::str::FromStr;
use std::sync::mpsc;
use std::thread;
use std::time::Instant;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Semantics)]
pub enum Semantics {
    // reference vertex positions with the co variable in vertex shaders
    #[sem(name = "pos", repr = "[f32; 2]", wrapper = "VertexPosition")]
    Position,
    // reference vertex colors with the color variable in vertex shaders
    #[sem(name = "uv", repr = "[f32; 2]", wrapper = "VertexUV")]
    UV,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Vertex)]
#[vertex(sem = "Semantics")]
pub struct Vertex {
    pub pos: VertexPosition,
    pub uv: VertexUV,
}

const VS: &str = include_str!("shader.vert");
const FS: &str = include_str!("shader.frag");

// Only one triangle this time.
const TRI_VERTICES: [Vertex; 4] = [
    Vertex {
        pos: VertexPosition::new([-1.0, -1.0]),
        uv: VertexUV::new([0., 0.]),
    },
    Vertex {
        pos: VertexPosition::new([1.0, -1.0]),
        uv: VertexUV::new([1.0, 0.0]),
    },
    Vertex {
        pos: VertexPosition::new([1.0, 1.0]),
        uv: VertexUV::new([1.0, 1.0]),
    },
    Vertex {
        pos: VertexPosition::new([-1.0, 1.0]),
        uv: VertexUV::new([0.0, 1.0]),
    },
];

// Create a uniform interface. This is a type that will be used to customize the shader. In our
// case, we just want to pass the time and the position of the triangle, for instance.
//
// This macro only supports structs for now; you cannot use enums as uniform interfaces.
#[derive(Debug, UniformInterface)]
struct ShaderInterface {
    time: Uniform<f32>,
    trigger_time: Uniform<f32>,
    resolution: Uniform<[f32; 2]>,
}

fn main() {
    let socket = UdpSocket::bind("127.0.0.1:0").expect("Could not listen");

    let (tx_outgoing_messages, rx_outgoing_messages) = mpsc::channel();

    let network_outgoing_hanndle = thread::spawn(move || {
        let addr = SocketAddr::from_str("127.0.0.1:57120").expect("Bad address");

        while let Ok(message) = rx_outgoing_messages.recv() {
            let buf = encoder::encode(&OscPacket::Message(message)).unwrap();
            socket.send_to(&buf, addr).unwrap();
        }
    });

    let mut surface =
        GlfwSurface::new(WindowDim::Windowed(960, 540), "Trill", WindowOpt::default())
            .expect("GLFW surface creation");

    // see the use of our uniform interface here as thirds type variable
    let program = Program::<Semantics, (), ShaderInterface>::from_strings(None, VS, None, FS)
        .expect("program creation")
        .ignore_warnings();

    let triangle = TessBuilder::new(&mut surface)
        .add_vertices(TRI_VERTICES)
        .set_mode(Mode::TriangleFan)
        .build()
        .unwrap();

    let mut back_buffer = surface.back_buffer().unwrap();

    // position of the triangle
    let mut trigger_time = -100.0;

    // reference time
    let start_t = Instant::now();
    let mut resize = false;

    'app: loop {
        // get the current monotonic time
        let elapsed = start_t.elapsed();
        let t64 = elapsed.as_secs() as f64 + f64::from(elapsed.subsec_millis()) * 1e-3;
        let time = t64 as f32;

        for event in surface.poll_events() {
            match event {
                WindowEvent::Close | WindowEvent::Key(Key::Escape, _, Action::Release, _) => break 'app,

                WindowEvent::Key(Key::Space, _, Action::Press, _) => {
                    trigger_time = time;

                    let message = OscMessage {
                        addr: "/".to_string(),
                        args: None,
                    };
                    tx_outgoing_messages
                        .send(message)
                        .expect("Failed to send message.");
                }

                WindowEvent::FramebufferSize(..) => {
                    resize = true;
                }

                _ => (),
            }
        }

        if resize {
            back_buffer = surface.back_buffer().unwrap();
            resize = false;
        }

        let width = surface.width() as f32;
        let height = surface.height() as f32;

        surface
            .pipeline_builder()
            .pipeline(&back_buffer, [0., 0., 0., 0.], |_, mut shd_gate| {
                // notice the iface free variable, which type is &ShaderInterface
                shd_gate.shade(&program, |iface, mut rdr_gate| {
                    // update the time and triangle position on the GPU shader program
                    iface.time.update(time);
                    iface.trigger_time.update(trigger_time);
                    iface.resolution.update([width, height]);

                    rdr_gate.render(RenderState::default(), |mut tess_gate| {
                        // render the dynamically selected slice
                        tess_gate.render(&triangle);
                    });
                });
            });

        surface.swap_buffers();
    }

    drop(tx_outgoing_messages);

    network_outgoing_hanndle
        .join()
        .expect("Failed to join network thread.");
}
