use crate::myjsonutils::*;
use crate::myopengl::*;
use crate::myrenderer::*;
use crate::mytypes::*;
use cgmath::{Vector3, Vector4};
use json::JsonValue;
use log::info;
use std::{collections::HashMap, fs, rc::Rc};

pub struct Settings {
    settings: JsonValue,
}

impl Settings {
    pub fn new(file: &str) -> Result<Self, MyError> {
        let settings = json_from_file(file)?;
        Ok(Self { settings })
    }

    pub fn get_str(&self, name: &str) -> Result<&str, MyError> {
        if !self.settings.has_key(name) {
            return Err(format!("Cannot find {} in settings", name).into());
        }

        self.settings[name]
            .as_str()
            .ok_or(format!("Invalid str {} in settings", name).into())
    }

    pub fn get_u32(&self, name: &str) -> Result<u32, MyError> {
        if !self.settings.has_key(name) {
            return Err(format!("Cannot find {} in settings", name).into());
        }

        self.settings[name]
            .as_u32()
            .ok_or(format!("Invalid u32 {} in settings", name).into())
    }
}

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

type GameObjTemplateLib = HashMap<String, Rc<GameObjectTemplate>>;

pub struct GameLib {
    simple_renderer: SimpleRenderer,
    comp_template_lib: ComponentTemplateLib,
    game_obj_template_lib: GameObjTemplateLib,
}

impl GameLib {
    pub fn load(settings: &Settings) -> Result<Self, MyError> {
        info!("Loading GameLib");

        let simple_renderer = SimpleRenderer::new(
            settings.get_str("simple_vertex_shader")?,
            settings.get_str("simple_frag_shader")?,
        )?;

        let comp_template_lib = ComponentTemplateLib::load(settings, &simple_renderer)?;
        let game_obj_template_lib = Self::load_game_obj_template_lib(settings, &comp_template_lib)?;

        info!("GameLib loaded successfully");

        Ok(Self {
            simple_renderer,
            comp_template_lib,
            game_obj_template_lib,
        })
    }

    #[inline]
    pub fn simple_renderer(&self) -> &SimpleRenderer {
        &self.simple_renderer
    }

    #[inline]
    pub fn comp_template_lib(&self) -> &ComponentTemplateLib {
        &self.comp_template_lib
    }

    pub fn find_game_obj_template(&self, name: &str) -> Result<&Rc<GameObjectTemplate>, MyError> {
        self.game_obj_template_lib
            .get(name)
            .ok_or(format!("Cannot find GameObjectTemplate {}", name).into())
    }

    fn load_game_obj_template_lib(
        settings: &Settings,
        lib: &ComponentTemplateLib,
    ) -> Result<GameObjTemplateLib, MyError> {
        let file = settings.get_str("game_object_templates")?;

        info!("Loading GameObjTemplateLib from {file}");

        let obj = json_from_file(file)?;
        let mut game_obj_lib = GameObjTemplateLib::new();

        for t in obj.members() {
            let template = GameObjectTemplate::from_json(t, lib)?;
            game_obj_lib.insert(template.name.clone(), Rc::new(template));
        }

        info!("GameObjTemplateLib loaded successfully");

        Ok(game_obj_lib)
    }
}

pub struct ComponentTemplateLib {
    mesh_templates: HashMap<String, Rc<MeshTemplate>>,
}

impl ComponentTemplateLib {
    pub fn load(settings: &Settings, renderer: &SimpleRenderer) -> Result<Self, MyError> {
        let mesh_templates = Self::load_mesh_templates(settings, renderer.program())?;

        Ok(Self { mesh_templates })
    }

    pub fn find_mesh_template(&self, name: &str) -> Result<&Rc<MeshTemplate>, MyError> {
        self.mesh_templates
            .get(name)
            .ok_or(format!("Failed to find MeshTemplate {}", name).into())
    }

    fn load_mesh_templates(
        settings: &Settings,
        program: &ShaderProgram,
    ) -> Result<HashMap<String, Rc<MeshTemplate>>, MyError> {
        let file = settings.get_str("mesh_templates")?;
        let contents = fs::read_to_string(file).map_err(|_| "Failed to read mesh template file")?;
        let json_value = json::parse(&contents).map_err(|_| "Failed to parse JSON")?;

        let mut templates = HashMap::new();

        for t in json_value.members() {
            let template = MeshTemplate::from_json(t, program)?;
            templates.insert(template.name.clone(), Rc::new(template));
        }

        Ok(templates)
    }
}

pub struct GameObjectTemplate {
    name: String,
    mesh_template: Rc<MeshTemplate>,
}

impl GameObjectTemplate {
    pub fn from_json(obj: &JsonValue, lib: &ComponentTemplateLib) -> Result<Self, MyError> {
        if !obj.has_key("name") {
            return Err("name is missing".into());
        }

        if !obj.has_key("mesh") {
            return Err("mesh is missing".into());
        }

        let name = obj["name"].as_str().ok_or("Invalid name")?.to_string();
        let mesh_str = obj["mesh"].as_str().ok_or("Invalid mesh")?;
        let mesh_template = lib.find_mesh_template(mesh_str)?;

        Ok(Self {
            name,
            mesh_template: mesh_template.clone(),
        })
    }

    #[inline]
    pub fn mesh_template(&self) -> &Rc<MeshTemplate> {
        &self.mesh_template
    }
}
