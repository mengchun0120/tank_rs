use crate::myopengl::*;
use crate::myrenderer::*;
use crate::mytemplates::*;
use cgmath::Vector2;
use std::rc::Rc;

pub enum GameObjectState {
    ACTIVE,
    DEAD,
}

pub struct GameObject {
    template: Rc<GameObjectTemplate>,
    state: GameObjectState,
    pos: Vector2<f32>,
    direction: Vector2<f32>,
}

impl GameObject {
    pub fn new(
        template: Rc<GameObjectTemplate>,
        pos: Vector2<f32>,
        direction: Vector2<f32>,
    ) -> Self {
        Self {
            state: GameObjectState::ACTIVE,
            template,
            pos,
            direction,
        }
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
