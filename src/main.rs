#![feature(map_values_mut)]

#[macro_use]
extern crate glium;
extern crate nalgebra as na;

use na::{Vec1, Vec2, Norm, Absolute, Rotate};
use glium::{DisplayBuild, Surface, VertexBuffer};
use glium::glutin;
use glium::glutin::{Event, ElementState, VirtualKeyCode};

use std::f32::consts::PI;
use std::collections::{HashMap, HashSet};

const ACCEL: f32 = 0.01;
const STEP: f32 = 0.02;

#[derive(Clone, Debug)]
struct Board {
    ship: Ship,
    mooks: HashMap<u32, Mook>,
    bullets: HashMap<u32, Bullet>,
    bulletct: u32
}

impl Board {
    fn new() -> Board {
        Board {
            ship: Ship::new(),
            mooks: HashMap::new(),
            bullets: HashMap::new(),
            bulletct: 0
        }
    }

    fn step(&mut self) {
        self.ship.step();
        for b in self.bullets.values_mut() {
            b.step()
        };
        for m in self.mooks.values_mut() {
            m.step()
        };
    }

    fn fire(&mut self) {
        let bullet = self.ship.fire();
        self.bullets.insert(self.bulletct, bullet);
        self.bulletct += 1;
    }

    fn tidy(&mut self) {
        let mut to_rm = Vec::new();
        for (&id, b) in &self.bullets {
            if b.pos.sqnorm() > 2.0 {
                to_rm.push(id)
            }
        }
        for id in to_rm {
            self.bullets.remove(&id).unwrap();
        }
    }
}

#[derive(Copy, Clone, Debug)]
struct Mook {
    pos: Vec2<f32>,
    vel: Vec2<f32>,
    health: f32
}

impl Mook {
    fn step(&mut self) {
    }
}

#[derive(Copy, Clone, Debug)]
struct Ship {
    pos: Vec2<f32>,
    vel: Vec2<f32>,
    orient: f32,
    health: f32
}

#[derive(Copy, Clone, Debug)]
struct Bullet {
    pos: Vec2<f32>,
    vel: Vec2<f32>,
}

impl Bullet {
    fn new(pos: Vec2<f32>, vel: Vec2<f32>) -> Bullet {
        Bullet {
            pos: pos,
            vel: vel
        }
    }

    fn vertices() -> Vec<Vertex> {
        vec![
            Vertex::new(-0.1, -1.0),
            Vertex::new(-0.1,  1.0),
            Vertex::new( 0.1,  1.0),
            Vertex::new( 0.1, -1.0),
        ]
    }

    fn step(&mut self) {
        self.pos = self.pos + self.vel * STEP
    }

    fn orient(&self) -> f32 {
        na::angle_between(&Vec2::x(), &self.vel)
    }
}

impl Ship {
    fn new() -> Ship {
        Ship {
            pos: Vec2::new(0.0, 0.0),
            vel: Vec2::new(0.0, 0.0),
            orient: 0.0,
            health: 1.0
        }
    }

    fn vertices() -> Vec<Vertex> {
        vec![
            Vertex::new(-0.3, -0.3),
            Vertex::new( 0.0,  0.5),
            Vertex::new( 0.3, -0.3),

            Vertex::new(-0.3, -0.3),
            Vertex::new( 0.0,  0.25),
            Vertex::new( 0.3, -0.3),

            Vertex::new(-0.3, -0.3),
            Vertex::new( 0.0,  0.0),
            Vertex::new( 0.3, -0.3),
        ]
    }

    fn thrust_vertices() -> Vec<Vertex> {
        vec![
            Vertex::new(-0.3, -0.3),
            Vertex::new(-0.15, -0.45),
            Vertex::new( 0.0, -0.3),

            Vertex::new(-0.0, -0.3),
            Vertex::new( 0.15, -0.45),
            Vertex::new( 0.3, -0.3),
        ]
    }

    fn step(&mut self) {
        self.pos = self.pos + self.vel * STEP;
        wrap(&mut self.pos)
    }

    fn accelerate(&mut self, accel: f32) {
        self.vel = self.vel + self.orient_vec() * accel;
        if self.vel.norm() > 1.0 {
            self.vel.normalize_mut();
        }
    }

    fn accel(&mut self) {
        self.accelerate(ACCEL)
    }

    fn decel(&mut self) {
        self.accelerate(-1.0 * ACCEL)
    }

    fn fire(&self) -> Bullet {
        Bullet::new(self.pos, self.orient_vec() * 1.1)
    }

    fn orient_vec(&self) -> Vec2<f32> {
        na::Rot2::new(Vec1::new(self.orient)).rotate(&Vec2::x())
    }
}

fn wrap(v: &mut Vec2<f32>) {
    if v.x.abs() > 1.05 { v.x *= -1.0 }
    if v.y.abs() > 1.05 { v.y *= -1.0 }
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
    [[t.cos(),  t.sin(),   0.],
     [-t.sin(),  t.cos(),   0.],
     [0.,             0.,  1.0]]
}

#[inline]
fn trans_mat(x: f32, y:f32) -> [[f32; 4]; 4] {
    [[1.0, 0.0, 0.0, 0.0],
     [0.0, 1.0, 0.0, 0.0],
     [0.0, 0.0, 1.0, 0.0],
     [  x,   y, 0.0, 1.0]]
}



struct Window {
    display: glium::backend::glutin_backend::GlutinFacade,
    program: glium::Program,
    events: Events
}

struct Events(HashSet<VirtualKeyCode>);

impl Events {
    fn update_keysdown(&mut self, keystate: ElementState, key: VirtualKeyCode) {
        let _ = match keystate {
            ElementState::Pressed => self.0.insert(key),
            ElementState::Released => self.0.remove(&key)
        };
    }
}


impl Window {
    fn new() -> Window {
        let display = glutin::WindowBuilder::new()
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
        Window {
            display: display,
            program: program,
            events: Events(HashSet::new())
        }
    }

    fn render(&self, board: &Board, assets: &Assets) {
        let mut target = self.display.draw();
        target.clear_color(0.0, 0.0, 0.0, 1.0);
        target.draw(&assets.ship_vx_buf,
                    &glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList),
                    &self.program,
                    &uniform!(
                        scale: scale_mat(0.1),
                        rot: rot_mat(board.ship.orient),
                        rot_offset: rot_mat(PI / -2.0),
                        trans: trans_mat(board.ship.pos.x, board.ship.pos.y)
                    ),
                    &assets.draw_params).unwrap();
        if self.events.0.contains(&VirtualKeyCode::Up) {
            target.draw(&assets.thruster_vx_buf,
                        &glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList),
                        &self.program,
                        &uniform!(
                            scale: scale_mat(0.1),
                            rot: rot_mat(board.ship.orient),
                            rot_offset: rot_mat(PI / -2.0),
                            trans: trans_mat(board.ship.pos.x, board.ship.pos.y)
                        ),
                        &assets.draw_params).unwrap();
        }
        for b in board.bullets.values() {
            target.draw(&assets.bullet_vx_buf,
                        &glium::index::NoIndices(glium::index::PrimitiveType::LineLoop),
                        &self.program,
                        &uniform!(
                            scale: scale_mat(0.01),
                            rot: rot_mat(b.orient()),
                            rot_offset: rot_mat(PI / -2.0),
                            trans: trans_mat(b.pos.x, b.pos.y)
                        ),
                        &assets.draw_params).unwrap();
        }
        target.finish().unwrap();
    }
}

struct Assets {
    ship_vx_buf: VertexBuffer<Vertex>,
    thruster_vx_buf: VertexBuffer<Vertex>,
    bullet_vx_buf: VertexBuffer<Vertex>,
    draw_params: glium::DrawParameters<'static>
}

impl Assets {
    fn new(window: &Window) -> Assets {
        let draw_params = glium::DrawParameters {
            polygon_mode: glium::draw_parameters::PolygonMode::Line,
            .. Default::default()
        };

        Assets {
            ship_vx_buf: VertexBuffer::new(&window.display, &Ship::vertices()).unwrap(),
            thruster_vx_buf: VertexBuffer::new(&window.display, &Ship::thrust_vertices()).unwrap(),
            bullet_vx_buf: VertexBuffer::new(&window.display, &Bullet::vertices()).unwrap(),
            draw_params: draw_params
        }
    }
}

fn game_loop(mut board: Board, mut window: Window) {
    let assets = Assets::new(&window);
    loop {
        board.step();
        window.render(&board, &assets);
        for ev in window.display.poll_events() {
            match ev {
                Event::Closed => return,
                Event::KeyboardInput(keystate, _, Some(key)) => window.events.update_keysdown(keystate, key),
                _ => ()
            }
        }
        for key in &window.events.0 {
            use glium::glutin::VirtualKeyCode::*;
            match *key {
                Left => board.ship.orient += 10.0 * PI / 180.,
                Right => board.ship.orient -= 10.0 * PI / 180.,
                Up => board.ship.accel(),
                Down => board.ship.decel(),
                Space => board.fire(),
                Q => return
            }
        }
        board.tidy();
    }
}

fn main() {
    let board = Board::new();
    let window = Window::new();
    game_loop(board, window);
}
