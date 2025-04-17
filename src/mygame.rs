use crate::myjsonutils::json_from_file;
use crate::myjsonutils::vector2_from_json;
use crate::myopengl::*;
use crate::myrenderer::*;
use crate::mytemplates::*;
use crate::mytypes::*;
use cgmath::Vector2;
use json::JsonValue;
use log::{info, warn};
use std::rc::Rc;

pub enum GameObjectState {
    Active,
    Dead,
}

pub enum GameObjectSide {
    AI,
    Player,
    Neutral,
}

impl TryFrom<&str> for GameObjectSide {
    type Error = MyError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "AI" => Ok(Self::AI),
            "Player" => Ok(Self::Player),
            "Neutral" => Ok(Self::Neutral),
            _ => Err(format!("Invalid side {}", value).into()),
        }
    }
}

pub struct GameObject {
    template: Rc<GameObjectTemplate>,
    state: GameObjectState,
    pos: Vector2<f32>,
    direction: Vector2<f32>,
    side: GameObjectSide,
}

impl GameObject {
    pub fn new(
        template: Rc<GameObjectTemplate>,
        pos: Vector2<f32>,
        direction: Vector2<f32>,
        side: GameObjectSide,
    ) -> Self {
        Self {
            state: GameObjectState::Active,
            template,
            pos,
            direction,
            side,
        }
    }

    pub fn from_json(obj: &JsonValue, lib: &GameLib) -> Result<Self, MyError> {
        if !obj.has_key("template") {
            return Err("template is missing".into());
        }

        if !obj.has_key("pos") {
            return Err("pos is missing".into());
        }

        let template_str = obj["template"].as_str().ok_or("Invalid template")?;
        let template = lib.find_game_obj_template(template_str)?;
        let pos = vector2_from_json(&obj["pos"])?;
        let direction = if obj.has_key("direction") {
            vector2_from_json(&obj["direction"])?
        } else {
            Vector2 { x: 1.0, y: 0.0 }
        };
        let side = if obj.has_key("side") {
            obj["side"].as_str().ok_or("Invalid side")?.try_into()?
        } else {
            GameObjectSide::Neutral
        };

        Ok(Self::new(template.clone(), pos, direction, side))
    }

    pub fn draw(&self, renderer: &SimpleRenderer) {
        let game_obj_template = self.template.as_ref();
        let mesh_template = game_obj_template.mesh_template().as_ref();

        renderer.set_use_obj_ref(true);
        renderer.set_obj_ref(&self.pos);
        renderer.set_use_direction(true);
        renderer.set_direction(&self.direction);
        renderer.set_z(mesh_template.z);
        if let Some(c) = &mesh_template.color {
            renderer.set_use_color(true);
            renderer.set_color(c);
        } else {
            renderer.set_use_color(false);
        }
        if let Some(tc) = &mesh_template.tex_color {
            renderer.set_use_tex_color(true);
            renderer.set_tex_color(tc);
        } else {
            renderer.set_use_tex_color(false);
        }
        if let Some(t) = &mesh_template.texture {
            renderer.set_tex_unit(0);
            t.bind(gl::TEXTURE0);
        }
        renderer.set_alpha(mesh_template.alpha);
        mesh_template.va.bind();
        draw_elemens(
            DrawMode::Triangles,
            0,
            mesh_template.va.vertice_count() as u32,
        );
    }

    #[inline]
    pub fn pos(&self) -> &Vector2<f32> {
        &self.pos
    }

    #[inline]
    pub fn direction(&self) -> &Vector2<f32> {
        &self.direction
    }
}

pub struct GameMap {
    map: Vec<Vec<Vec<GameObject>>>,
}

const GAME_MAP_ROWS: usize = 20;
const GAME_MAP_COLS: usize = 30;
const GAME_CELL_SIZE: usize = 40;
const WINDOW_WIDTH: usize = GAME_CELL_SIZE * GAME_MAP_COLS;
const WINDOW_HEIGHT: usize = GAME_CELL_SIZE * GAME_MAP_ROWS;

impl GameMap {
    fn new() -> Self {
        let mut map = Vec::new();

        for _ in 0..GAME_MAP_ROWS {
            let mut r = Vec::new();
            for _ in 0..GAME_MAP_COLS {
                let cell: Vec<GameObject> = Vec::new();
                r.push(cell);
            }
            map.push(r);
        }

        Self { map }
    }

    fn from_file(file: &str) -> Result<Self, MyError> {
        let mut map = Self::new();
        let obj = json_from_file(file)?;

        Ok(map)
    }

    fn add(&mut self, obj: GameObject) -> bool {
        let (row, col) = Self::get_cell_pos(&obj);

        if row < 0 || row >= GAME_MAP_ROWS as i32 {
            warn!("Invalid row {}", row);
            return false;
        }

        if col < 0 || col >= GAME_MAP_COLS as i32 {
            warn!("Invalid col {}", col);
            return false;
        }

        self.map[row as usize][col as usize].push(obj);

        true
    }

    #[inline]
    fn get_cell_pos(obj: &GameObject) -> (i32, i32) {
        (
            (obj.pos.x / GAME_CELL_SIZE as f32).round() as i32,
            (obj.pos.y / GAME_CELL_SIZE as f32).round() as i32,
        )
    }
}
