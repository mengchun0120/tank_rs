mod mygame;
mod myjsonutils;
mod myopengl;
mod myrender;
mod mytests;
mod mytypes;

use cgmath::Vector2;
use glfw::{Action, Context, Glfw, GlfwReceiver, Key, PWindow, WindowEvent};
use mygame::*;
use myopengl::*;
use myrender::*;
use mytypes::*;
use std::{collections::HashMap, fs, rc::Rc};

pub struct App {
    simple_render: SimpleRender,
    mesh_templates: HashMap<String, Rc<MeshTemplate>>,
    mesh: Mesh,
    viewport_origin: Vector2<f32>,
    viewport_size: Vector2<f32>,
    glfw: Glfw,
    window: PWindow,
    events: GlfwReceiver<(f64, WindowEvent)>,
}

impl App {
    pub fn new(width: u32, height: u32, title: &str) -> Result<Self, MyError> {
        let mut glfw = Self::init_glfw()?;
        let (window, events) = Self::init_window(&mut glfw, width, height, title)?;

        let simple_render = SimpleRender::new(
            "res/glsl/simple_vertex_shader.glsl",
            "res/glsl/simple_frag_shader.glsl",
        )?;

        let mesh_templates = Self::load_mesh_templates(simple_render.program())?;

        let template = mesh_templates
            .get("tile")
            .ok_or("Mesh template not found")?;

        let mesh = Mesh::new(
            template.clone(),
            Vector2 { x: 200.0, y: 200.0 },
            Vector2 { x: 1.0, y: 0.0 },
        );

        let viewport_origin = Vector2 {
            x: width as f32 / 2.0,
            y: height as f32 / 2.0,
        };
        let viewport_size = Vector2 {
            x: width as f32,
            y: height as f32,
        };

        Ok(Self {
            simple_render,
            mesh_templates,
            mesh,
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

        Ok((window, events))
    }

    fn load_mesh_templates(
        program: &ShaderProgram,
    ) -> Result<HashMap<String, Rc<MeshTemplate>>, MyError> {
        let contents = fs::read_to_string("res/mesh_templates.json")
            .map_err(|_| "Failed to read mesh template file")?;
        let json_value = json::parse(&contents).map_err(|_| "Failed to parse JSON")?;

        let mut templates = HashMap::new();

        for t in json_value.members() {
            let template = MeshTemplate::from_json(t, program)?;
            templates.insert(template.name.clone(), Rc::new(template));
        }

        Ok(templates)
    }

    fn init_opengl(&self) {
        unsafe {
            gl::ClearColor(0.2, 0.3, 0.3, 1.0);
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
            gl::Enable(gl::DEPTH_TEST);
        }

        self.simple_render.apply();
        self.simple_render
            .set_viewport_origin(&self.viewport_origin);
        self.simple_render.set_viewport_size(&self.viewport_size);
    }

    fn process_events(&mut self) {
        for (_, event) in glfw::flush_messages(&self.events) {
            match event {
                glfw::WindowEvent::FramebufferSize(width, height) => unsafe {
                    gl::Viewport(0, 0, width, height)
                },
                glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                    self.window.set_should_close(true)
                }
                _ => {}
            }
        }
    }

    fn render(&mut self) {
        self.clear_window();
        self.simple_render.apply();
        self.mesh.draw(&self.simple_render);
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
