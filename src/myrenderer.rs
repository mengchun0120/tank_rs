use crate::myopengl::ShaderProgram;
use crate::mytypes::MyError;
use cgmath::{Vector2, Vector3, Vector4};

#[allow(unused)]
pub struct SimpleRenderer {
    program: ShaderProgram,
    position_loc: i32,
    tex_pos_loc: i32,
    use_obj_ref_loc: i32,
    obj_ref_loc: i32,
    viewport_size_loc: i32,
    viewport_origin_loc: i32,
    use_direction_loc: i32,
    direction_loc: i32,
    z_loc: i32,
    use_color_loc: i32,
    color_loc: i32,
    use_tex_color_loc: i32,
    tex_color_loc: i32,
    tex_unit_loc: i32,
    alpha_loc: i32,
}

#[allow(unused)]
impl SimpleRenderer {
    pub fn new(vertex_shader_file: &str, frag_shader_file: &str) -> Result<Self, MyError> {
        let program = ShaderProgram::new(vertex_shader_file, frag_shader_file)?;
        Ok(Self {
            position_loc: program.get_attrib_loc("position")?,
            tex_pos_loc: program.get_attrib_loc("texPos")?,
            use_obj_ref_loc: program.get_uniform_loc("useObjRef")?,
            obj_ref_loc: program.get_uniform_loc("objRef")?,
            viewport_size_loc: program.get_uniform_loc("viewportSize")?,
            viewport_origin_loc: program.get_uniform_loc("viewportOrigin")?,
            use_direction_loc: program.get_uniform_loc("useDirection")?,
            direction_loc: program.get_uniform_loc("direction")?,
            z_loc: program.get_uniform_loc("z")?,
            use_color_loc: program.get_uniform_loc("useColor")?,
            color_loc: program.get_uniform_loc("color")?,
            use_tex_color_loc: program.get_uniform_loc("useTexColor")?,
            tex_color_loc: program.get_uniform_loc("texColor")?,
            tex_unit_loc: program.get_uniform_loc("texUnit")?,
            alpha_loc: program.get_uniform_loc("alpha")?,
            program,
        })
    }

    #[inline]
    pub fn apply(&self) {
        self.program.use_program();
    }

    #[inline]
    pub fn program(&self) -> &ShaderProgram {
        &self.program
    }

    #[inline]
    pub fn position_loc(&self) -> i32 {
        self.position_loc
    }

    #[inline]
    pub fn tex_pos_loc(&self) -> i32 {
        self.tex_pos_loc
    }

    #[inline]
    pub fn set_use_obj_ref(&self, use_obj_ref: bool) {
        self.program
            .set_uniform_bool(self.use_obj_ref_loc, use_obj_ref);
    }

    #[inline]
    pub fn set_obj_ref(&self, obj_ref: &Vector2<f32>) {
        self.program.set_uniform_2fv(self.obj_ref_loc, obj_ref);
    }

    #[inline]
    pub fn set_viewport_size(&self, viewport_size: &Vector2<f32>) {
        self.program
            .set_uniform_2fv(self.viewport_size_loc, viewport_size);
    }

    #[inline]
    pub fn set_viewport_origin(&self, viewport_origin: &Vector2<f32>) {
        self.program
            .set_uniform_2fv(self.viewport_origin_loc, viewport_origin);
    }

    #[inline]
    pub fn set_use_direction(&self, use_direction: bool) {
        self.program
            .set_uniform_bool(self.use_direction_loc, use_direction);
    }

    #[inline]
    pub fn set_direction(&self, direction: &Vector2<f32>) {
        self.program.set_uniform_2fv(self.direction_loc, direction);
    }

    #[inline]
    pub fn set_z(&self, z: f32) {
        self.program.set_uniform_1f(self.z_loc, z);
    }

    #[inline]
    pub fn set_use_color(&self, use_color: bool) {
        self.program.set_uniform_bool(self.use_color_loc, use_color);
    }

    #[inline]
    pub fn set_color(&self, color: &Vector3<f32>) {
        self.program.set_uniform_3fv(self.color_loc, color);
    }

    #[inline]
    pub fn set_use_tex_color(&self, use_tex_color: bool) {
        self.program
            .set_uniform_bool(self.use_tex_color_loc, use_tex_color);
    }

    #[inline]
    pub fn set_tex_color(&self, tex_color: &Vector4<f32>) {
        self.program.set_uniform_4fv(self.tex_color_loc, tex_color);
    }

    #[inline]
    pub fn set_tex_unit(&self, tex_unit: i32) {
        self.program.set_uniform_1i(self.tex_unit_loc, tex_unit);
    }

    #[inline]
    pub fn set_alpha(&self, alpha: f32) {
        self.program.set_uniform_1f(self.alpha_loc, alpha);
    }
}
