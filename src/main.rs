extern crate piston;
extern crate piston_window;
extern crate glutin_window;
extern crate graphics;
extern crate opengl_graphics;
extern crate gfx_graphics;
extern crate gfx_device_gl;
//extern crate image;

extern crate vector;
extern crate perlin_noise;

use std::sync::atomic::AtomicPtr;

use piston_window::*;
use piston::window::WindowSettings;
use opengl_graphics::OpenGL;
// use image::{Rgb, RgbImage};

use perlin_noise::noise::*;
use vector::vector::{VectorS, VectorD};

mod gui;

use gui::{App, AppGraphics, Data};

const PERLIN_WIDTH: u32 = 600;
const PERLIN_HEIGHT: u32 = 600;
const TILE_WIDTH: u32 = 1;
const TILE_HEIGHT: u32 = 1;

fn main() {
    println!("Hello, world!");

    let noise = PerlinNoise::new(NoiseType::Barycentric, 2, 3, VectorS::from(vec![10, 10]));
    
    let noise = match noise {
        Ok(n) => n,
        Err(msg) => panic!(msg)
    };

    let opengl = OpenGL::V4_5;

    // Create an Glutin window.
    let window: PistonWindow = WindowSettings::new(
            "perlin noise visualizer",
            [PERLIN_WIDTH * TILE_WIDTH, PERLIN_HEIGHT * TILE_HEIGHT]
        )
        .resizable(false)
        .samples(8)
        .opengl(opengl)
        .exit_on_esc(true)
        .build()
        .unwrap();
    
    gui::start(window, PerlinVis::new(noise));
}

#[derive(Clone, Debug)]
enum StateData {
    Gen, Working, Render
}

#[derive(Clone)]
struct State {
    data: StateData
}

struct PerlinVis {
    data: AtomicPtr<Data>,
    window: AtomicPtr<PistonWindow>,
    noise: Arc<Mutex<PerlinNoise>>, cache: Arc<Mutex<Vec<Vec<VectorD>>>>,
    state: Arc<Mutex<State>>
}

use std::sync::{Mutex, Arc};

impl PerlinVis {
    pub fn new(noise: PerlinNoise) -> Self {
        Self {
            data: AtomicPtr::default(),
            window: AtomicPtr::default(),
            noise: Arc::new(Mutex::new(noise)), state: Arc::new(Mutex::new(State { data: StateData::Gen })),
            cache: Arc::new(Mutex::new(vec![vec![VectorD::new(0); PERLIN_WIDTH as usize];PERLIN_HEIGHT as usize]))
        }
    }
}

impl App for PerlinVis {
    fn render(&self, c: Context, g: &mut AppGraphics) {
        use graphics::*;

        let tol_h = 0.01;
        let tol_l = 0.05;

        let default = VectorD::new(3);

        let clr_h = |c1, c2| {
            let d = c1 - c2;
            if d < tol_h {
                1.0
            } else if d < tol_l{
                0.5
            } else {
                0.0
            }
        };

        let clr = |c1, c2, c3| {
            (1.0, clr_h(c1, c2), clr_h(c1, c3))
        };
        
        if let StateData::Render = self.state.lock().unwrap().data {
            if let Ok(cache) = self.cache.lock() {
                for i in 0..PERLIN_WIDTH as usize {
                    for j in 0..PERLIN_HEIGHT as usize {
                        let mut dat = &cache[i][j];
                        if dat.dim() == 0 {
                            dat = &default;
                        }
                        let dat = dat.shift(1.0) / 2.0;
                        
                        let (cr, cg, cb) = (dat[0] as f32, dat[1] as f32, dat[2] as f32);
                        let (mut cr, mut cg, mut cb) = (cg + cb, cr + cb, cr + cg);
                        let s = f32::max(f32::max(cr, cg), cb);
                        
                        if cr == s {
                            let c = clr(cr, cg, cb);
                            cr = c.0;
                            cg = c.1;
                            cb = c.2;
                        } else if cg == s {
                            let c = clr(cg, cr, cb);
                            cr = c.1;
                            cg = c.0;
                            cb = c.2;
                        } else if cb == s {
                            let c = clr(cb, cg, cr);
                            cr = c.2;
                            cg = c.1;
                            cb = c.0;
                        }
                        
                        // let mut image = ImageBuffer::<image::Rgb<f32>>::new(100, 100);
                        let color: [f32; 4] = [1.0 - cr, 1.0 - cg, 1.0 - cb, 1.0];
                        let sq = rectangle::square(i as f64, j as f64, 1.0);
                        let t = c.transform.scale(TILE_WIDTH as f64, TILE_HEIGHT as f64);
                        
                        rectangle(color, sq, t, g);
                    }
                }
            }
        }
    }
    
    fn update(&mut self, _args: &UpdateArgs) {
        use std::thread;
        if let StateData::Gen = self.state.lock().unwrap().data {
            let (noise, cache, state) = (self.noise.clone(), self.cache.clone(), self.state.clone());
            
            thread::spawn(move || {
                state.lock().unwrap().data = StateData::Working;
                let noise = noise.lock().unwrap();

                for i in 0..PERLIN_WIDTH as usize {
                    let mut cache = cache.lock().unwrap();
                    for j in 0..PERLIN_HEIGHT as usize {
                        let (x, y) = (i as f64 * 10.0 / PERLIN_WIDTH as f64, 
                                    j as f64 * 10.0 / PERLIN_HEIGHT as f64);
                        cache[i][j] = noise.eval(x, y);
                    }
                    println!("gen = {:.2}%%", 100.0 * (i + 1) as f64 / PERLIN_WIDTH as f64);
                }

                state.lock().unwrap().data = StateData::Render;
            }); 
        }
    }
    
    fn set_data(&mut self, data: AtomicPtr<Data>) {
        self.data = data;
    }

    fn set_window(&mut self, window: AtomicPtr<PistonWindow>) {
        self.window = window;
    }

    fn handle_mouse(&mut self, _: MouseButton, _: f64, _: f64) {
        if let Ok(mut noise) = self.noise.lock() {
            if let Err(msg) = noise.regen() {
                panic!(msg);
            }
            self.state.lock().unwrap().data = StateData::Gen;
        }
    }
}