use crate::mytypes::MyError;
use gl::types::*;
use std::os::raw::c_void;
use std::{ffi::CString, fmt::Display, fs, mem, ptr, str};

pub enum ShaderType {
    Vertex,
    Fragment,
}

impl ShaderType {
    pub fn to_gl_enum(&self) -> GLenum {
        match self {
            Self::Vertex => gl::VERTEX_SHADER,
            Self::Fragment => gl::FRAGMENT_SHADER,
        }
    }
}

impl Display for ShaderType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Vertex => write!(f, "VertexShader"),
            Self::Fragment => write!(f, "FragmentShader"),
        }
    }
}

pub struct Shader {
    id: u32,
}

impl Shader {
    pub fn new(shader_type: ShaderType, source: &str) -> Result<Self, MyError> {
        unsafe {
            let id = gl::CreateShader(shader_type.to_gl_enum());
            if id == 0 {
                return Err(format!("Failed to create shader {}", shader_type).into());
            }

            let c_str_vert = CString::new(source.as_bytes()).map_err(|_| "Invalid source")?;

            gl::ShaderSource(id, 1, &c_str_vert.as_ptr(), ptr::null());
            gl::CompileShader(id);

            let mut success = gl::FALSE as GLint;
            gl::GetShaderiv(id, gl::COMPILE_STATUS, &mut success);
            if success != gl::TRUE as GLint {
                let mut info_log = Vec::with_capacity(512);
                info_log.set_len(512 - 1);

                gl::GetShaderInfoLog(
                    id,
                    512,
                    ptr::null_mut(),
                    info_log.as_mut_ptr() as *mut GLchar,
                );

                let msg = str::from_utf8(&info_log).map_err(|_| "Failed to get complie info")?;

                return Err(format!("Failed to complie {}: {}", shader_type, msg).into());
            }

            Ok(Self { id })
        }
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteShader(self.id);
        }
    }
}

pub struct ShaderProgram {
    id: u32,
}

impl ShaderProgram {
    pub fn new(vertex_shader_file: &str, frag_shader_file: &str) -> Result<Self, MyError> {
        let vertex_shader_source = fs::read_to_string(vertex_shader_file)?;
        let vertex_shader = Shader::new(ShaderType::Vertex, &vertex_shader_source)?;

        let frag_shader_source = fs::read_to_string(frag_shader_file)?;
        let frag_shader = Shader::new(ShaderType::Fragment, &frag_shader_source)?;

        unsafe {
            let id = gl::CreateProgram();
            if id == 0 {
                return Err(format!("Failed to create program").into());
            }

            gl::AttachShader(id, vertex_shader.id);
            gl::AttachShader(id, frag_shader.id);
            gl::LinkProgram(id);

            let mut success = gl::FALSE as GLint;
            gl::GetProgramiv(id, gl::LINK_STATUS, &mut success);
            if success != gl::TRUE as GLint {
                let mut info_log = Vec::with_capacity(512);
                info_log.set_len(512 - 1);

                gl::GetProgramInfoLog(
                    id,
                    512,
                    ptr::null_mut(),
                    info_log.as_mut_ptr() as *mut GLchar,
                );
                let msg = str::from_utf8(&info_log).map_err(|_| "Failed to get link info")?;

                return Err(format!("Failed to link shader program: {}", msg).into());
            }

            Ok(Self { id })
        }
    }

    pub fn get_attrib_loc(&self, name: &str) -> Result<i32, MyError> {
        unsafe {
            let name_cstr = CString::new(name.as_bytes()).map_err(|_| "Invalid name")?;
            let loc = gl::GetAttribLocation(self.id, name_cstr.as_ptr());
            if loc == -1 {
                return Err(format!("Failed to find attrib {}", name).into());
            }

            Ok(loc)
        }
    }

    pub fn get_uniform_loc(&self, name: &str) -> Result<i32, MyError> {
        unsafe {
            let name_cstr = CString::new(name.as_bytes()).map_err(|_| "Invalid name")?;
            let loc = gl::GetUniformLocation(self.id, name_cstr.as_ptr());
            if loc == -1 {
                return Err(format!("Failed to find uniform {}", name).into());
            }

            Ok(loc)
        }
    }

    pub fn use_program(&self) {
        unsafe {
            gl::UseProgram(self.id);
        }
    }
}

impl Drop for ShaderProgram {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteProgram(self.id);
        }
    }
}

pub struct VertexAttribPointer {
    index: u32,
    size: u32,
    stride: usize,
    offset: usize,
}

impl VertexAttribPointer {
    pub fn new(index: u32, size: u32, stride: usize, offset: usize) -> Self {
        Self {
            index,
            size,
            stride,
            offset,
        }
    }
}

pub struct VertexArray {
    id: u32,
}

impl VertexArray {
    pub fn new(
        vertices: &[f32],
        indices: Option<&[i32]>,
        pointers: &[VertexAttribPointer],
    ) -> Self {
        unsafe {
            let (mut vbo, mut vao) = (0, 0);
            gl::GenVertexArrays(1, &mut vao);
            gl::GenBuffers(1, &mut vbo);

            gl::BindVertexArray(vao);

            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (vertices.len() * mem::size_of::<GLfloat>()) as GLsizeiptr,
                &vertices[0] as *const f32 as *const c_void,
                gl::STATIC_DRAW,
            );

            if let Some(i) = indices {
                let mut ebo = 0;
                gl::GenBuffers(1, &mut ebo);
                gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
                gl::BufferData(
                    gl::ELEMENT_ARRAY_BUFFER,
                    (i.len() * mem::size_of::<GLfloat>()) as GLsizeiptr,
                    &i[0] as *const i32 as *const c_void,
                    gl::STATIC_DRAW,
                );
            }

            Self::set_pointers(pointers);

            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
            Self::unbind();

            Self { id: vao }
        }
    }

    pub fn bind(&self) {
        unsafe {
            gl::BindVertexArray(self.id);
        }
    }

    pub fn unbind() {
        unsafe {
            gl::BindVertexArray(0);
        }
    }

    fn set_pointers(pointers: &[VertexAttribPointer]) {
        unsafe {
            for pointer in pointers {
                gl::VertexAttribPointer(
                    pointer.index,
                    pointer.size as i32,
                    gl::FLOAT,
                    gl::FALSE,
                    (pointer.stride * mem::size_of::<GLfloat>()) as GLsizei,
                    (pointer.offset * mem::size_of::<GLfloat>()) as *const c_void,
                );
                gl::EnableVertexAttribArray(pointer.index);
            }
        }
    }
}

pub enum DrawMode {
    TRIANGLES = gl::TRIANGLES as isize,
}

pub struct Render {}

impl Render {
    pub fn draw_elemens(mode: DrawMode, count: u32) {
        unsafe {
            gl::DrawElements(mode as u32, count as GLsizei, gl::UNSIGNED_INT, ptr::null());
        }
    }
}
