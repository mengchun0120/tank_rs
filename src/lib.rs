mod mygame;
mod myopengl;
mod myrender;
mod mytests;
mod mytypes;

use cgmath::Vector2;
use glfw::{Action, Context, Glfw, GlfwReceiver, Key, PWindow, WindowEvent};
use myopengl::*;
use myrender::*;
use mytypes::*;

pub struct App {
    simple_render: SimpleRender,
    va: VertexArray,
    texture: Texture,
    viewport_origin: Vector2<f32>,
    viewport_size: Vector2<f32>,
    obj_ref: Vector2<f32>,
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
        let va = Self::init_vertex_array(&simple_render)?;
        let texture = Texture::new("res/container.jpg")?;

        Ok(Self {
            simple_render,
            va,
            texture,
            viewport_origin: Vector2 {
                x: width as f32 / 2.0,
                y: height as f32 / 2.0,
            },
            viewport_size: Vector2 {
                x: width as f32,
                y: height as f32,
            },
            obj_ref: Vector2 {
                x: width as f32 / 2.0,
                y: height as f32 / 2.0,
            },
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

    fn init_vertex_array(simple_renderer: &SimpleRender) -> Result<VertexArray, MyError> {
        let positions = &[-50.0, -50.0, 50.0, -50.0, 50.0, 50.0, -50.0, 50.0];
        let tex_coords = &[0.0, 0.0, 1.0, 0.0, 1.0, 1.0, 0.0, 1.0];
        let blocks = vec![
            VertexDataBlock::new(2, positions)?,
            VertexDataBlock::new(2, tex_coords)?,
        ];
        let vertices = interleave_vertex_data(&blocks)?;
        let indices = [0, 1, 3, 1, 2, 3];
        let pointers = [
            VertexAttribPointer::new(simple_renderer.position_loc() as u32, 2, 4, 0),
            VertexAttribPointer::new(simple_renderer.tex_pos_loc() as u32, 2, 4, 2),
        ];

        VertexArray::new(&vertices, &indices, &pointers)
    }

    fn init_opengl(&self) {
        unsafe {
            gl::ClearColor(0.2, 0.3, 0.3, 1.0);
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
            gl::Enable(gl::DEPTH_TEST);
        }
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
        self.simple_render
            .set_viewport_origin(&self.viewport_origin);
        self.simple_render.set_viewport_size(&self.viewport_size);
        self.simple_render.set_use_direction(false);
        self.simple_render.set_use_obj_ref(true);
        self.simple_render.set_obj_ref(&self.obj_ref);
        self.simple_render.set_z(0.0);
        self.simple_render.set_alpha(1.0);
        self.simple_render.set_tex_unit(0);
        self.simple_render.set_use_color(false);
        self.va.bind();
        self.texture.bind(gl::TEXTURE0);
        draw_elemens(gl::TRIANGLES, 0, 6);
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
