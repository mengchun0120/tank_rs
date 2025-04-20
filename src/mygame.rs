use crate::myjsonutils::json_from_file;
use crate::myjsonutils::vector2_from_json;
use crate::myopengl::*;
use crate::myrenderer::*;
use crate::mytemplates::*;
use crate::mytypes::*;
use cgmath::Vector2;
use json::JsonValue;
use log::{info, warn};
use std::fmt::Display;
use std::rc::Rc;

#[derive(Debug, PartialEq)]
pub enum GameObjectState {
    Active,
    Dead,
}

#[derive(Debug, PartialEq)]
pub enum Side {
    AI,
    Player,
    Neutral,
}

impl TryFrom<&str> for Side {
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

#[derive(Debug, PartialEq, Clone)]
pub enum Direction {
    Up = 0,
    Down = 1,
    Left = 2,
    Right = 3,
}

const DIRECTIONS: &[Vector2<f32>] = &[
    Vector2 { x: 0.0, y: 1.0 },
    Vector2 { x: 0.0, y: -1.0 },
    Vector2 { x: -1.0, y: 0.0 },
    Vector2 { x: 1.0, y: 0.0 },
];

impl Direction {
    pub fn to_vec(&self) -> &Vector2<f32> {
        &DIRECTIONS[self.clone() as usize]
    }

}

impl Default for Direction {
    fn default() -> Self {
        Self::Left
    }
}

impl TryFrom<&str> for Direction {
    type Error = MyError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "Up" => Ok(Self::Up),
            "Down" => Ok(Self::Down),
            "Left" => Ok(Self::Left),
            "Right" => Ok(Self::Right),
            _ => Err(format!("Invalid direction: {}", value).into()),
        }
    }
}

pub struct GameObject {
    template: Rc<GameObjectTemplate>,
    state: GameObjectState,
    pos: Vector2<f32>,
    direction: Direction,
    side: Side,
    hp: Option<u32>,
    moving: bool,
}

impl GameObject {
    pub fn new(
        template: Rc<GameObjectTemplate>,
        pos: Vector2<f32>,
        direction: Direction,
        side: Side,
        hp: Option<u32>,
    ) -> Self {
        Self {
            state: GameObjectState::Active,
            template,
            pos,
            direction,
            side,
            hp,
            moving: false,
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
            obj["direction"]
                .as_str()
                .ok_or("Invalid direction")?
                .try_into()?
        } else {
            Direction::default()
        };
        let side = if obj.has_key("side") {
            let side_str = obj["side"].as_str().ok_or("Invalid side")?;
            info!("side_str={}", side_str);
            side_str.try_into()?
        } else {
            Side::Neutral
        };

        Ok(Self::new(
            template.clone(),
            pos,
            direction,
            side,
            template.hp,
        ))
    }

    pub fn draw(&self, renderer: &SimpleRenderer) {
        let game_obj_template = self.template.as_ref();
        let mesh_template = game_obj_template.mesh_template.as_ref();

        renderer.set_use_obj_ref(true);
        renderer.set_obj_ref(&self.pos);
        renderer.set_use_direction(true);
        renderer.set_direction(self.direction.to_vec());
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
    pub fn direction(&self) -> Direction {
        self.direction.clone()
    }

    #[inline]
    pub fn set_direction(&mut self, d: Direction) {
        self.direction = d;
    }
}

pub struct GameMap {
    map: Vec<Vec<GameObject>>,
    player_idx: Option<usize>,
}

pub const GAME_MAP_ROWS: usize = 20;
pub const GAME_MAP_COLS: usize = 30;
pub const GAME_CELL_SIZE: usize = 40;
pub const GAME_CELL_COUNT: usize = GAME_MAP_COLS * GAME_MAP_ROWS;
pub const WINDOW_WIDTH: usize = GAME_CELL_SIZE * GAME_MAP_COLS;
pub const WINDOW_HEIGHT: usize = GAME_CELL_SIZE * GAME_MAP_ROWS;

impl GameMap {
    pub fn new() -> Self {
        let mut map = Vec::new();

        for _ in 0..GAME_CELL_COUNT {
            let cell = Vec::new();
            map.push(cell);
        }

        Self {
            map,
            player_idx: None,
        }
    }

    pub fn from_file(file: &str, lib: &GameLib) -> Result<Self, MyError> {
        info!("Initializing GameMap from {file}");

        let mut map = Self::new();
        let obj = json_from_file(file)?;

        for m in obj.members() {
            let game_obj = GameObject::from_json(m, lib)?;
            map.add(game_obj);
        }

        info!("Initialized GameMap successfully");
        Ok(map)
    }

    pub fn add(&mut self, obj: GameObject) -> bool {
        let (col, row) = Self::get_cell_pos(&obj);

        if row < 0 || row >= GAME_MAP_ROWS as i32 {
            warn!("Invalid row {}", row);
            return false;
        }

        if col < 0 || col >= GAME_MAP_COLS as i32 {
            warn!("Invalid col {}", col);
            return false;
        }

        let idx = Self::get_cell_idx(row, col);

        if obj.side == Side::Player {
            if self.player_idx.is_some() {
                warn!("More than one player");
                return false;
            }

            info!("Player at idx {idx}");
            self.player_idx = Some(idx);
        }

        info!("add obj idx={idx} row={row} col={col} side={:?}", obj.side);

        self.map[idx].push(obj);

        true
    }

    pub fn draw(&self, renderer: &SimpleRenderer) {
        for cell in self.map.iter() {
            for obj in cell.iter() {
                obj.draw(renderer);
            }
        }
    }

    pub fn get_cell(&self, row: i32, col: i32) -> Option<&[GameObject]> {
        if row < 0 || row >= GAME_MAP_ROWS as i32 {
            warn!("Invalid row {row}");
            return None;
        }

        if col < 0 || col >= GAME_MAP_COLS as i32 {
            warn!("Invalid col {col}");
            return None;
        }

        let idx = Self::get_cell_idx(row, col);
        Some(&self.map[idx])
    }

    pub fn get_player(&self) -> Option<&GameObject> {
        match self.player_idx {
            Some(idx) => self.map[idx].iter().find(|o| o.side == Side::Player),
            None => None,
        }
    }

    pub fn get_player_mut(&mut self) -> Option<&mut GameObject> {
        match self.player_idx {
            Some(idx) => self.map[idx].iter_mut().find(|o| o.side == Side::Player),
            None => None,
        }
    }

    pub fn set_player_direction(&mut self, d: Direction) {
        if let Some(player) = self.get_player_mut() {
            player.set_direction(d);
        }
    }

    #[inline]
    pub fn get_cell_pos(obj: &GameObject) -> (i32, i32) {
        (
            (obj.pos.x / GAME_CELL_SIZE as f32).floor() as i32,
            (obj.pos.y / GAME_CELL_SIZE as f32).floor() as i32,
        )
    }

    #[inline]
    pub fn get_cell_idx(row: i32, col: i32) -> usize {
        row as usize * GAME_MAP_COLS + col as usize
    }
}
