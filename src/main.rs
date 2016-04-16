#[macro_use]
extern crate glium;

use glium::glutin::Event;
use std::f32::consts::PI;

#[derive(Copy, Clone, Debug)]
struct Vel(f32, f32);
#[derive(Copy, Clone, Debug)]
struct Pos(f32, f32);


const DIFF: f32 = 0.01;

#[derive(Clone, Debug)]
struct Board {
    ship: Ship,
    mooks: Vec<Mook>
}

#[derive(Copy, Clone, Debug)]
struct Mook {
    pos: Pos,
    vel: Vel,
    health: f32
}

#[derive(Copy, Clone, Debug)]
struct Ship {
    pos: Pos,
    vel: Vel,
    orient: f32,
    health: f32
}

impl Ship {
    fn new() -> Ship {
        Ship {
            pos: Pos(0.0, 0.0),
            vel: Vel(0.0, 0.0),
            orient: 0.0,
            health: 1.0
        }
    }

    fn vertices() -> Vec<Vertex> {
        vec![Vertex::new(-0.3, -0.3),
             Vertex::new( 0.0,  0.5),
             Vertex::new( 0.3, -0.3)]
    }

    fn step(&mut self) {
        let stepamt = 0.05;
        self.pos.0 += self.vel.0 * stepamt;
        self.pos.1 += self.vel.1 * stepamt;
    }

    fn change_vel(&mut self, diff: f32) {
        self.vel.0 += diff * self.orient.cos();
        self.vel.1 -= diff * self.orient.sin();
    }

    fn incr_vel(&mut self) {
        self.change_vel(DIFF)
    }

    fn decr_vel(&mut self) {
        self.change_vel(-1.0 * DIFF)
    }
}

const VERTEX_SHADER : &'static str = r#"
    #version 140

    in vec2 position;

    uniform mat3 scale;
    uniform mat3 rot;
    uniform mat3 rot_offset;
    uniform mat4 trans;

    void main() {
        mat4 model = trans * mat4(rot_offset * rot * scale);
        gl_Position = model * vec4(position, 0.0, 1.0);
    }
"#;

const FRAGMENT_SHADER : &'static str = r#"
    #version 140

    out vec4 color;

    void main() {
        color = vec4(0.9, 0.9, 0.9, 1.0);
    }
"#;

#[derive(Copy, Clone, Debug)]
struct Vertex {
    position: [f32; 2]
}

impl Vertex {
    fn new(x: f32, y: f32) -> Vertex {
        Vertex {
            position: [x, y]
        }
    }
}

implement_vertex!(Vertex, position);

#[inline]
fn scale_mat(s: f32) -> [[f32; 3]; 3] {
    [[ s, 0., 0.],
     [0.,  s, 0.],
     [0., 0.,  s]]
}

#[inline]
fn rot_mat(t: f32) -> [[f32; 3]; 3] {
    [[t.cos(), -t.sin(),   0.],
     [t.sin(),  t.cos(),   0.],
     [0.,             0.,  1.0]]
}

#[inline]
fn trans_mat(x: f32, y:f32) -> [[f32; 4]; 4] {
    [[1.0, 0.0, 0.0, 0.0],
     [0.0, 1.0, 0.0, 0.0],
     [0.0, 0.0, 1.0, 0.0],
     [  x,   y, 0.0, 1.0]]
}



fn main() {
    let mut ship = Ship::new();

    use glium::{DisplayBuild, Surface};
    let display = glium::glutin::WindowBuilder::new()
        .build_glium()
        .unwrap();

    let program = match glium::Program::from_source(&display, VERTEX_SHADER,
                                                    FRAGMENT_SHADER, None) {
        Ok(p) => p,
        Err(e) => {
            println!("{}", e);
            panic!()
        }
    };

    let shipvx = Ship::vertices();
    let vertex_buffer = glium::VertexBuffer::new(&display, &shipvx).unwrap();
    let indices = glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList);

    let draw_params = glium::DrawParameters {
        polygon_mode: glium::draw_parameters::PolygonMode::Line,
        .. Default::default()
    };


    loop {
        ship.step();
        let mut target = display.draw();
        target.clear_color(0.0, 0.0, 0.0, 1.0);
        target.draw(&vertex_buffer,
                    &indices,
                    &program,
                    &uniform!(
                        scale: scale_mat(0.1),
                        rot: rot_mat(ship.orient),
                        rot_offset: rot_mat(PI / 2.0),
                        trans: trans_mat(ship.pos.0, ship.pos.1)
                    ),
                    &draw_params).unwrap();
        target.finish().unwrap();

        for ev in display.poll_events() {
            use glium::glutin::VirtualKeyCode::*;
            match ev {
                Event::Closed => return,
                Event::KeyboardInput(_, _, opt) => {
                    println!("{:?}", opt);
                    match opt {
                        Some(Left)  => ship.orient -= 10.0 * PI / 180.,
                        Some(Right)  => ship.orient += 10.0 * PI / 180.,
                        Some(Up)  => ship.incr_vel(),
                        Some(Down)  => ship.decr_vel(),
                        Some(Q)  => return,
                        _ => ()
                    }
                }
                _ => ()
            }
        }

    }
}
