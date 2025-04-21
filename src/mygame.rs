use crate::myjsonutils::json_from_file;
use crate::myjsonutils::vector2_from_json;
use crate::myopengl::*;
use crate::myrenderer::*;
use crate::mytemplates::*;
use crate::mytypes::*;
use cgmath::Vector2;
use json::JsonValue;
use log::{info, warn};
use std::mem;
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

#[derive(Debug, PartialEq)]
enum ObjAction {
    Move,
    Attack,
    Idle,
}

pub struct GameObject {
    template: Rc<GameObjectTemplate>,
    state: GameObjectState,
    pos: Vector2<f32>,
    direction: Direction,
    side: Side,
    hp: Option<u32>,
    action: ObjAction,
    last_act_dur: f32,
    updated: bool,
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
            action: ObjAction::Idle,
            last_act_dur: 0.0,
            updated: false,
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
    max_collide_span: f32,
}

pub const GAME_MAP_ROWS: usize = 20;
pub const GAME_MAP_COLS: usize = 30;
pub const GAME_CELL_SIZE: usize = 40;
pub const GAME_CELL_COUNT: usize = GAME_MAP_COLS * GAME_MAP_ROWS;
pub const WINDOW_WIDTH: usize = GAME_CELL_SIZE * GAME_MAP_COLS;
pub const WINDOW_HEIGHT: usize = GAME_CELL_SIZE * GAME_MAP_ROWS;
pub const MAX_OBJ_X: f32 = WINDOW_WIDTH as f32 - 0.1;
pub const MAX_OBJ_Y: f32 = WINDOW_HEIGHT as f32 - 0.1;
pub const MOVE_ACT_DUR: f32 = 1000.0 / 60.0;

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
            max_collide_span: 0.0,
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
        let idx = match Self::get_cell_idx(&obj.pos) {
            Some(i) => i,
            None => {
                return false;
            }
        };

        if obj.side == Side::Player {
            if self.player_idx.is_some() {
                warn!("More than one player");
                return false;
            }

            self.player_idx = Some(idx);
        }

        if self.max_collide_span < obj.template.collide_span {
            self.max_collide_span = obj.template.collide_span;
        }

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

    pub fn move_player(&mut self, d: Direction, action: glfw::Action) {
        if let Some(player) = self.get_player_mut() {
            player.action = ObjAction::Move;
            player.set_direction(d);
        }
    }

    pub fn update(&mut self, time_delta: f32) {
        self.map
            .iter_mut()
            .flatten()
            .for_each(|obj| obj.updated = false);

        for cell_idx in 0..GAME_CELL_COUNT {
            for i in (0..self.map[cell_idx].len()).rev() {
                let mut obj = self.map[cell_idx].pop().unwrap();
                let new_cell_idx = self.update_obj(&mut obj, cell_idx, time_delta);
                self.map[new_cell_idx].push(obj);

                if i >= 1 {
                    let last_idx = self.map[cell_idx].len() - 1;
                    self.map[cell_idx].swap(i - 1, last_idx);
                }
            }
        }
    }

    pub fn update_obj(&mut self, obj: &mut GameObject, cell_idx: usize, time_delta: f32) -> usize {
        if obj.updated {
            return cell_idx;
        }

        match obj.action {
            ObjAction::Move => {
                let cell_idx = self.move_obj(obj, cell_idx, time_delta);
                obj.action = ObjAction::Idle;
                return cell_idx;
            }
            ObjAction::Attack => {}
            ObjAction::Idle => {}
        }

        cell_idx
    }

    pub fn move_obj(&mut self, obj: &mut GameObject, cell_idx: usize, time_delta: f32) -> usize {
        obj.last_act_dur += time_delta;
        if obj.last_act_dur < MOVE_ACT_DUR {
            return cell_idx;
        }

        while obj.last_act_dur >= MOVE_ACT_DUR {
            let disp = obj.direction.to_vec() * obj.template.speed * MOVE_ACT_DUR;
            obj.pos += disp;
            obj.last_act_dur -= MOVE_ACT_DUR;
        }

        obj.pos.x = obj.pos.x.clamp(0.0, MAX_OBJ_X);
        obj.pos.y = obj.pos.y.clamp(0.0, MAX_OBJ_Y);

        obj.last_act_dur = 0.0;
        let new_cell_idx = Self::get_cell_idx(&obj.pos).unwrap();

        if obj.side == Side::Player {
            self.player_idx = Some(new_cell_idx);
        }
        
        new_cell_idx
    }

    #[inline]
    pub fn get_cell_idx(pos: &Vector2<f32>) -> Option<usize> {
        let col = (pos.x / GAME_CELL_SIZE as f32).floor() as i32;
        if col < 0 || col >= GAME_MAP_COLS as i32 {
            warn!("Invalid pos {:?}", pos);
            return None;
        }

        let row = (pos.y / GAME_CELL_SIZE as f32).floor() as i32;
        if row < 0 || row >= GAME_MAP_ROWS as i32 {
            warn!("Invalid pos {:?}", pos);
            return None;
        }

        Some(row as usize * GAME_MAP_COLS + col as usize)
    }
}
