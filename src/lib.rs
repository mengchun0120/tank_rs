pub mod mygame;
pub mod myjsonutils;
pub mod myopengl;
pub mod myrenderer;
pub mod mytemplates;
mod mytests;
pub mod mytypes;

use cgmath::Vector2;
use glfw::{Action, Context, Glfw, GlfwReceiver, Key, PWindow, WindowEvent};
use log::info;
use mygame::*;
use mytemplates::*;
use mytypes::*;

pub struct App {
    settings: Settings,
    map: GameMap,
    lib: GameLib,
    viewport_origin: Vector2<f32>,
    viewport_size: Vector2<f32>,
    glfw: Glfw,
    window: PWindow,
    events: GlfwReceiver<(f64, WindowEvent)>,
}

impl App {
    pub fn new(settings: Settings) -> Result<Self, MyError> {
        info!("Initializing app");

        let title = settings.get_str("title")?;
        let mut glfw = Self::init_glfw()?;
        let (window, events) =
            Self::init_window(&mut glfw, WINDOW_WIDTH as u32, WINDOW_HEIGHT as u32, title)?;

        let lib = GameLib::load(&settings)?;

        let viewport_origin = Vector2 {
            x: WINDOW_WIDTH as f32 / 2.0,
            y: WINDOW_HEIGHT as f32 / 2.0,
        };
        let viewport_size = Vector2 {
            x: WINDOW_WIDTH as f32,
            y: WINDOW_HEIGHT as f32,
        };

        let map = GameMap::from_file("res/map/1.json", &lib)?;

        info!("App initialized successfully");

        Ok(Self {
            settings,
            map,
            lib,
            viewport_origin,
            viewport_size,
            glfw,
            window,
            events,
        })
    }

    pub fn run(&mut self) {
        self.init_opengl();
        while !self.window.should_close() {
            self.process_events();
            self.render();
            self.post_update();
        }
    }

    fn init_glfw() -> Result<Glfw, MyError> {
        glfw::init(glfw::fail_on_errors).map_err(|_| "Failed to initialize".into())
    }

    fn init_window(
        glfw: &mut Glfw,
        width: u32,
        height: u32,
        title: &str,
    ) -> Result<(PWindow, GlfwReceiver<(f64, WindowEvent)>), MyError> {
        info!("Initializing window: width={width} height={height} title={title}");

        glfw.window_hint(glfw::WindowHint::ContextVersion(3, 3));
        glfw.window_hint(glfw::WindowHint::OpenGlProfile(
            glfw::OpenGlProfileHint::Core,
        ));
        glfw.window_hint(glfw::WindowHint::Resizable(false));

        let (mut window, events) = glfw
            .create_window(width, height, title, glfw::WindowMode::Windowed)
            .ok_or("Failed to create GLFW window")?;

        window.make_current();
        window.set_key_polling(true);
        window.set_framebuffer_size_polling(true);

        gl::load_with(|symbol| window.get_proc_address(symbol) as *const _);

        info!("Window initialized successfully");

        Ok((window, events))
    }

    fn init_opengl(&self) {
        unsafe {
            gl::ClearColor(0.2, 0.3, 0.3, 1.0);
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
            gl::Enable(gl::DEPTH_TEST);
            gl::Viewport(
                0,
                0,
                self.viewport_size.x as i32,
                self.viewport_size.y as i32,
            );
        }

        let renderer = self.lib.simple_renderer();
        renderer.apply();
        renderer.set_viewport_origin(&self.viewport_origin);
        renderer.set_viewport_size(&self.viewport_size);
    }

    fn process_events(&mut self) {
        for (_, event) in glfw::flush_messages(&self.events) {
            match event {
                glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                    self.window.set_should_close(true);
                }
                glfw::WindowEvent::Key(Key::Up, _, action, _) => {
                    info!("Up action={:?}", action);
                    self.map.set_player_direction(Direction::Up);
                }
                glfw::WindowEvent::Key(Key::Down, _, Action::Press, _) => {
                    self.map.set_player_direction(Direction::Down);
                }
                glfw::WindowEvent::Key(Key::Left, _, Action::Press, _) => {
                    self.map.set_player_direction(Direction::Left);
                }
                glfw::WindowEvent::Key(Key::Right, _, Action::Press, _) => {
                    self.map.set_player_direction(Direction::Right);
                }
                _ => {}
            }
        }
    }

    fn render(&mut self) {
        self.clear_window();
        let renderer = self.lib.simple_renderer();
        renderer.apply();
        self.map.draw(renderer);
    }

    fn post_update(&mut self) {
        self.window.swap_buffers();
        self.glfw.poll_events();
    }

    fn clear_window(&mut self) {
        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }
    }
}
