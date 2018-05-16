use std::sync::atomic::{AtomicPtr, Ordering};

use piston::input::*;
use piston_window::*;
use gfx_graphics::GfxGraphics;
use gfx_device_gl::{Resources, CommandBuffer};

pub type AppGraphics<'a> = GfxGraphics<'a, Resources, CommandBuffer>;

pub trait App {
    fn render(&self, _args: Context, _gl: &mut AppGraphics);
    fn update(&mut self, _args: &UpdateArgs);
    fn set_data(&mut self, _data: AtomicPtr<Data>);
    fn set_window(&mut self, window: AtomicPtr<PistonWindow>);

    // called after rendering
    fn post_render(&self, _args: &AfterRenderArgs) {}

    fn idle(&self, _args: &IdleArgs) {}

    fn handle_key(&mut self, _key: Key) {}
    
    fn handle_mouse(&mut self, _mouse_button: MouseButton, _mouse_x: f64, _mouse_y: f64) {}

    fn handle_controller(&mut self, _controller_button: ControllerButton) {}
    
    fn handle_key_held(&mut self, _key: Key) {}
    
    fn handle_mouse_held(&mut self, _mouse_button: MouseButton) {}

    fn handle_controller_held(&mut self, _controller_button: ControllerButton) {}

    // handle mouse movement
    fn mouse_moved(&mut self, _args: &Motion) {}
    
    // handle cursor going on and off screen
    fn handle_cursor(&mut self, _cursor: bool) {}

    // handle window focus going on and off
    fn handle_focus(&mut self, _focus: bool) {}

    // handle window resizing
    fn handle_resize(&mut self, _width: u32, _height: u32) {}

    fn on_close(&mut self, _args: &CloseArgs) {}
}

pub struct Data {
    pub is_cursor_on: bool,
    pub is_window_focus: bool,
    pub screen_width: u32,
    pub screen_height: u32,
    pub mouse_x: f64,
    pub mouse_y: f64,
    pub button_held: Vec<Button>
}

impl Data {
    fn new(window: &PistonWindow) -> Data {
        let size = window.size();
        let (screen_width, screen_height) = (size.width, size.height);

        Data {
            is_cursor_on: false,
            is_window_focus: false,
            screen_width, screen_height,
            mouse_x: 0.0,
            mouse_y: 0.0,
            button_held: Vec::new()
        }
    }
}

pub fn unwrap<T>(t: &AtomicPtr<T>) -> &T {
    unsafe {
        &*t.load(Ordering::Relaxed)
    }
}

pub fn unwrap_mut<T>(t: &mut AtomicPtr<T>) -> &mut T {
    unsafe {
        &mut *t.load(Ordering::Relaxed)
    }
}

pub fn start<T>(mut window: PistonWindow, mut app: T)
    where T: App {
    // let mut events = Events::new(EventSettings::new());
    let mut _found = false;

    let mut data = Data::new(&window);

    app.set_data(AtomicPtr::new(&mut data));
    app.set_window(AtomicPtr::new(&mut window));

    let mut data = AtomicPtr::new(&mut data);
    let mut window = AtomicPtr::new(&mut window);

    loop {
        let e = unwrap_mut(&mut window).next();

        if let None = e {
            break;
        }

        let e = e.unwrap();

        match e {
            Event::Custom(_a, _b) => {

            },
            Event::Loop(l) => {
                match l {
                    Loop::Render(_r) => {
                        unwrap_mut(&mut window).draw_2d(&e, |c, g| app.render(c, g));
                    },
                    Loop::Update(u) => {
                        let d = unwrap(&data);

                        for button in &d.button_held {
                            match button {
                                &Button::Keyboard(key) => 
                                    app.handle_key_held(key),
                                &Button::Mouse(mouse_button) => 
                                    app.handle_mouse_held(mouse_button),
                                &Button::Controller(controller_button) => 
                                    app.handle_controller_held(controller_button)
                            }
                        }

                        app.update(&u);
                    },
                    Loop::AfterRender(ar) => {
                        app.post_render(&ar);
                    },
                    Loop::Idle(i) => {
                        // println!("idle time {:?}ms", _a.dt * 1000.0);
                        app.idle(&i);
                    }
                }
            }
            Event::Input(i) => {
                match i {
                    Input::Button(b) => {
                        let d = unwrap_mut(&mut data);
                        let contains = d.button_held.contains(&b.button);
                        
                        if !contains {
                            match b.button {
                                Button::Keyboard(key) => 
                                    app.handle_key(key),
                                Button::Mouse(mouse_button) => 
                                    app.handle_mouse(mouse_button, d.mouse_x, d.mouse_y),
                                Button::Controller(controller_button) => 
                                    app.handle_controller(controller_button)
                            }
                        }

                        match b.state {
                            ButtonState::Press => {
                                if !contains {
                                    d.button_held.push(b.button);
                                }
                            },
                            ButtonState::Release => {
                                if contains {
                                    let index = d.button_held.iter().position(|x| *x == b.button).unwrap();
                                    d.button_held.remove(index);
                                }
                            }
                        }
                    },
                    Input::Move(m) => {
                        let d = unwrap_mut(&mut data);
                        if let Motion::MouseCursor(x, y) = m {
                            d.mouse_x = x;
                            d.mouse_y = y;
                        }
                    },
                    Input::Resize(w, h) => {
                        let d = unwrap_mut(&mut data);

                        d.screen_width = w;
                        d.screen_height = h;
                    },
                    Input::Text(_t) => {

                    },
                    Input::Cursor(c) => {
                        let d = unwrap_mut(&mut data);

                        d.is_cursor_on = c;
                        app.handle_cursor(c);
                    },
                    Input::Focus(f) => {
                        let d = unwrap_mut(&mut data);

                        d.is_window_focus = f;
                        app.handle_focus(f);
                    },
                    Input::Close(c) => {
                        app.on_close(&c);
                    }
                }
            }
        }
    }
}
