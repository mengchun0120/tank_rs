use crate::mytypes::MyError;
use cgmath::{Vector2, Vector3, prelude::*};
use gl::types::*;
use image::EncodableLayout;
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

    #[inline]
    pub fn use_program(&self) {
        unsafe {
            gl::UseProgram(self.id);
        }
    }

    #[inline]
    pub fn set_uniform_int(&self, loc: i32, value: i32) {
        unsafe {
            gl::Uniform1i(loc, value);
        }
    }

    #[inline]
    pub fn set_uniform_bool(&self, loc: i32, value: bool) {
        unsafe {
            gl::Uniform1i(loc, value as i32);
        }
    }

    #[inline]
    pub fn set_uniform_1i(&self, loc: i32, value: i32) {
        unsafe {
            gl::Uniform1i(loc, value);
        }
    }

    #[inline]
    pub fn set_uniform_1f(&self, loc: i32, value: f32) {
        unsafe {
            gl::Uniform1f(loc, value);
        }
    }

    #[inline]
    pub fn set_uniform_2fv(&self, loc: i32, value: &Vector2<f32>) {
        unsafe {
            gl::Uniform2fv(loc, 1, value.as_ptr());
        }
    }

    #[inline]
    pub fn set_uniform_3fv(&self, loc: i32, value: &Vector3<f32>) {
        unsafe {
            gl::Uniform3fv(loc, 1, value.as_ptr());
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
    vbo: u32,
    ebo: Option<u32>,
    vao: u32,
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

            let mut ebo: Option<u32> = None;
            if let Some(i) = indices {
                let mut b = 0;
                gl::GenBuffers(1, &mut b);
                gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, b);
                gl::BufferData(
                    gl::ELEMENT_ARRAY_BUFFER,
                    (i.len() * mem::size_of::<GLfloat>()) as GLsizeiptr,
                    &i[0] as *const i32 as *const c_void,
                    gl::STATIC_DRAW,
                );
                ebo = Some(b);
            }

            Self::set_pointers(pointers);

            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
            Self::unbind();

            Self { vao, vbo, ebo }
        }
    }

    pub fn bind(&self) {
        unsafe {
            gl::BindVertexArray(self.vao);
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

impl Drop for VertexArray {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteBuffers(1, &self.vbo);
            if let Some(i) = self.ebo {
                gl::DeleteBuffers(1, &i);
            }
            gl::DeleteVertexArrays(1, &self.vao);
        }
    }
}

pub fn draw_elemens(mode: GLenum, first: u32, count: u32) {
    unsafe {
        gl::DrawElements(
            mode,
            count as GLsizei,
            gl::UNSIGNED_INT,
            (first as usize) as *const c_void,
        );
    }
}

pub fn draw_arrays(mode: GLenum, first: u32, count: u32) {
    unsafe {
        gl::DrawArrays(mode, first as GLint, count as GLsizei);
    }
}

pub struct Texture {
    width: u32,
    height: u32,
    id: u32,
}

impl Texture {
    pub fn new(file_path: &str) -> Result<Self, MyError> {
        let img = image::open(file_path)
            .map_err(|e| format!("Failed to open image {}: {}", file_path, e))?;
        let img = img.flipv();
        let img = img.into_rgba8();

        unsafe {
            let mut id = 0;
            gl::GenTextures(1, &mut id);

            gl::BindTexture(gl::TEXTURE_2D, id);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as GLint);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as GLint);

            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as GLint);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as GLint);

            let data = img.as_bytes();
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGBA as i32,
                img.width() as i32,
                img.height() as i32,
                0,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                &data[0] as *const u8 as *const c_void,
            );

            gl::GenerateMipmap(gl::TEXTURE_2D);

            Ok(Self {
                width: img.width(),
                height: img.height(),
                id,
            })
        }
    }

    pub fn bind(&self, unit: GLenum) {
        unsafe {
            gl::ActiveTexture(unit);
            gl::BindTexture(gl::TEXTURE_2D, self.id);
        }
    }
}

impl Drop for Texture {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteTextures(1, &self.id);
        }
    }
}

pub struct VertexDataBlock<'a> {
    vertex_size: usize,
    data: &'a [f32],
}

impl<'a> VertexDataBlock<'a> {
    pub fn new(vertex_size: usize, data: &'a [f32]) -> Result<Self, MyError> {
        if vertex_size == 0 {
            return Err("vertex_size is zero".into());
        }

        if data.len() == 0 || data.len() % vertex_size != 0 {
            return Err("Invalid data size".into());
        }

        Ok(Self {
            vertex_size,
            data: data,
        })
    }

    #[inline]
    pub fn vertex_size(&self) -> usize {
        self.vertex_size
    }

    #[inline]
    pub fn data(&self) -> &[f32] {
        &self.data
    }

    #[inline]
    pub fn num_of_vertices(&self) -> usize {
        self.data.len() / self.vertex_size
    }

    #[inline]
    pub fn get_slice(&self, start: usize) -> Result<&[f32], MyError> {
        if start + self.vertex_size > self.data.len() {
            return Err("Invalid start".into());
        }

        Ok(&self.data[start..(start + self.vertex_size)])
    }
}

pub fn interleave_vertex_data(blocks: &[VertexDataBlock]) -> Result<Vec<f32>, MyError> {
    if blocks.len() == 0 {
        return Err("blocks is empty".into());
    }

    let num_of_vertices = blocks[0].num_of_vertices();
    let valid = blocks
        .iter()
        .skip(1)
        .all(|v| v.num_of_vertices() == num_of_vertices);

    if !valid {
        return Err("blocks don't have the same number of vertices".into());
    }

    let total_len: usize = blocks.iter().map(|v| v.data().len()).sum();
    let mut result: Vec<f32> = vec![0_f32; total_len];
    let mut block_offsets = vec![0_usize; blocks.len()];
    let mut offset: usize = 0;

    while offset < total_len {
        for (i, b) in blocks.iter().enumerate() {
            let vertex_size = b.vertex_size();
            let src = b.get_slice(block_offsets[i])?;
            let dst = &mut result[offset..(offset + vertex_size)];
            dst.copy_from_slice(src);
            block_offsets[i] += vertex_size;
            offset += vertex_size;
        }
    }

    Ok(result)
}
