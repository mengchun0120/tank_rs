mod myopengl;
mod mytypes;

use glfw::{Action, Context, Glfw, GlfwReceiver, Key, PWindow, WindowEvent};
use myopengl::{Render, ShaderProgram, Texture, VertexArray, VertexAttribPointer};
use mytypes::MyError;

pub struct App {
    glfw: Glfw,
    window: PWindow,
    events: GlfwReceiver<(f64, WindowEvent)>,
    shader_program: ShaderProgram,
    va: VertexArray,
    texture: Texture,
    texture_loc: i32,
}

impl App {
    pub fn new(width: u32, height: u32, title: &str) -> Result<Self, MyError> {
        let mut glfw = Self::init_glfw()?;
        let (window, events) = Self::init_window(&mut glfw, width, height, title)?;
        let shader_program = ShaderProgram::new("res/vertex_shader.glsl", "res/frag_shader.glsl")?;
        let va = Self::init_vertex_array(&shader_program)?;
        let texture = Texture::new("res/container.jpg")?;
        let texture_loc = shader_program.get_uniform_loc("texture1")?;

        Ok(Self {
            glfw,
            window,
            events,
            shader_program,
            va,
            texture,
            texture_loc,
        })
    }

    pub fn run(&mut self) {
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

        let (mut window, events) = glfw
            .create_window(width, height, title, glfw::WindowMode::Windowed)
            .ok_or("Failed to create GLFW window")?;

        window.make_current();
        window.set_key_polling(true);
        window.set_framebuffer_size_polling(true);

        gl::load_with(|symbol| window.get_proc_address(symbol) as *const _);

        Ok((window, events))
    }

    fn init_vertex_array(shader_program: &ShaderProgram) -> Result<VertexArray, MyError> {
        let vertices = [
            0.5, 0.5, 0.0, 1.0, 1.0, 0.5, -0.5, 0.0, 1.0, 0.0, -0.5, -0.5, 0.0, 0.0, 0.0, -0.5,
            0.5, 0.0, 0.0, 1.0,
        ];
        let indices = [0, 1, 3, 1, 2, 3];
        let pointers = [
            VertexAttribPointer::new(shader_program.get_attrib_loc("pos")? as u32, 3, 5, 0),
            VertexAttribPointer::new(shader_program.get_attrib_loc("tex_coord")? as u32, 2, 5, 3),
        ];

        Ok(VertexArray::new(&vertices, Some(&indices), &pointers))
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
        self.shader_program.use_program();
        self.va.bind();
        self.shader_program.set_uniform_int(self.texture_loc, 0);
        self.texture.bind(gl::TEXTURE0);
        Render::draw_elemens(gl::TRIANGLES, 0, 6);
    }

    fn post_update(&mut self) {
        self.window.swap_buffers();
        self.glfw.poll_events();
    }

    fn clear_window(&mut self) {
        unsafe {
            gl::ClearColor(0.2, 0.3, 0.3, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }
    }
}
