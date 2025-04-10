use crate::myjsonutils::*;
use crate::myopengl::*;
use crate::myrender::*;
use crate::mytypes::*;
use cgmath::{Vector2, Vector3, Vector4};
use json::JsonValue;
use std::rc::Rc;

pub struct MeshTemplate {
    pub name: String,
    pub va: VertexArray,
    pub z: f32,
    pub color: Option<Vector3<f32>>,
    pub tex_color: Option<Vector4<f32>>,
    pub texture: Option<Texture>,
    pub alpha: f32,
}

impl MeshTemplate {
    pub fn from_json(value: &JsonValue, program: &ShaderProgram) -> Result<Self, MyError> {
        if !value.has_key("name") {
            return Err("Missing name".into());
        }

        if !value.has_key("va") {
            return Err("Missing va".into());
        }

        let name = value["name"].as_str().ok_or("Invalid name")?.to_string();
        let va = VertexArray::from_json(&value["va"], program)?;

        let z = if value.has_key("z") {
            value["z"].as_f32().ok_or("Invalid z")?
        } else {
            0.0
        };

        let color = if value.has_key("color") {
            Some(rgb_from_json(&value["color"])?)
        } else {
            None
        };

        let texture = if value.has_key("texture") {
            let path = value["texture"].as_str().ok_or("Invalid texture")?;
            Some(Texture::new(path)?)
        } else {
            None
        };

        let tex_color = if value.has_key("texColor") {
            Some(rgba_from_json(&value["texColor"])?)
        } else {
            None
        };

        let alpha = if value.has_key("alpha") {
            alpha_from_json(&value["alpha"])?
        } else {
            1.0
        };

        Ok(Self {
            name,
            va,
            z,
            color,
            tex_color,
            texture,
            alpha,
        })
    }
}

pub struct Mesh {
    pub template: Rc<MeshTemplate>,
    pub pos: Vector2<f32>,
    pub direction: Vector2<f32>,
}

impl Mesh {
    pub fn new(template: Rc<MeshTemplate>, pos: Vector2<f32>, direction: Vector2<f32>) -> Self {
        Self {
            template,
            pos,
            direction,
        }
    }

    pub fn draw(&self, renderer: &SimpleRender) {
        let template = self.template.as_ref();

        renderer.set_use_obj_ref(true);
        renderer.set_obj_ref(&self.pos);
        renderer.set_use_direction(true);
        renderer.set_direction(&self.direction);
        renderer.set_z(template.z);
        if let Some(c) = &template.color {
            renderer.set_use_color(true);
            renderer.set_color(c);
        } else {
            renderer.set_use_color(false);
        }
        if let Some(tc) = &template.tex_color {
            renderer.set_use_tex_color(true);
            renderer.set_tex_color(tc);
        } else {
            renderer.set_use_tex_color(false);
        }
        if let Some(t) = &template.texture {
            renderer.set_tex_unit(0);
            t.bind(gl::TEXTURE0);
        }
        renderer.set_alpha(template.alpha);
        template.va.bind();
        draw_elemens(DrawMode::Triangles, 0, template.va.vertice_count() as u32);
    }
}
