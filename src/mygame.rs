use crate::myopengl::*;
use crate::myrender::*;
use crate::mytypes::*;
use cgmath::{Vector2, Vector3};
use json::JsonValue;
use std::rc::Rc;

pub struct MeshTemplate {
    pub name: String,
    pub va: VertexArray,
    pub z: f32,
    pub color: Option<Vector3<f32>>,
    pub tex: Option<Texture>,
    pub alpha: f32,
}

impl MeshTemplate {
    pub fn from_json(value: &JsonValue, program: &ShaderProgram) -> Result<Self, MyError> {
        let name = value["name"].as_str().ok_or("Invalid name")?.to_string();
        let z = value["z"].as_f32().ok_or("Invalid z")?;

        todo!()
    }
}

pub struct Mesh {
    pub template: Rc<MeshTemplate>,
    pub pos: Vector2<f32>,
    pub direction: Vector2<f32>,
}

impl Mesh {
    fn draw(&self, renderer: &SimpleRender) {
        let template = self.template.as_ref();

        renderer.set_use_obj_ref(true);
        renderer.set_obj_ref(&self.pos);
        renderer.set_use_direction(true);
        renderer.set_direction(&self.direction);
        renderer.set_z(template.z);
        if let Some(c) = &template.color {
            renderer.set_use_color(true);
            renderer.set_color(c);
        }
        if let Some(t) = &template.tex {
            renderer.set_tex_unit(0);
            t.bind(gl::TEXTURE0);
        }
        renderer.set_alpha(template.alpha);
        template.va.bind();
        draw_arrays(gl::TRIANGLES, 0, template.va.vertice_count() as u32);
    }
}
