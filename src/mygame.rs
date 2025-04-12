use crate::myjsonutils::*;
use crate::myopengl::*;
use crate::myrenderer::*;
use crate::mytypes::*;
use cgmath::{Vector2, Vector3, Vector4};
use json::JsonValue;
use std::{collections::HashMap, fs, rc::Rc};

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

pub struct GameLib {
    simple_renderer: SimpleRenderer,
    mesh_templates: HashMap<String, Rc<MeshTemplate>>,
}

impl GameLib {
    pub fn load() -> Result<Self, MyError> {
        let simple_renderer = SimpleRenderer::new(
            "res/glsl/simple_vertex_shader.glsl",
            "res/glsl/simple_frag_shader.glsl",
        )?;

        let mesh_templates = Self::load_mesh_templates(simple_renderer.program())?;

        Ok(Self {
            simple_renderer,
            mesh_templates,
        })
    }

    pub fn find_mesh_template(&self, name: &str) -> Result<&Rc<MeshTemplate>, MyError> {
        self.mesh_templates
            .get(name)
            .ok_or(format!("Failed to find MeshTemplate {}", name).into())
    }

    #[inline]
    pub fn simple_renderer(&self) -> &SimpleRenderer {
        &self.simple_renderer
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

    pub fn draw(&self, renderer: &SimpleRenderer) {
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
