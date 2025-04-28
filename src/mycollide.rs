use cgmath::Vector2;

pub fn check_obj_collide(
    pos1: &mut Vector2<f32>,
    region1: &Vector2<f32>,
    pass1: bool,
    direction1: &Vector2<f32>,
    pos2: &Vector2<f32>,
    region2: &Vector2<f32>,
    pass2: bool,
) -> bool {
    if pass1 && pass2 {
        return false;
    }

    let collide = pos1.x - region1.x >= pos2.x + region2.x
        || pos1.x + region1.x <= pos2.x - region2.x
        || pos1.y - region1.y >= pos2.y + region2.y
        || pos1.y + region1.y <= pos2.y - region2.y;

    if !collide {
        return false;
    }

    if pass1 || pass2 || (direction1.x == 0.0 && direction1.y == 0.0) {
        return true;
    }

    let wx = if direction1.x > 0.0 {
        pos1.x + region1.x - pos2.x + region2.x
    } else if direction1.x < 0.0 {
        pos2.x + region2.x - pos1.x + region1.x
    } else {
        0.0
    };

    let wy = if direction1.y > 0.0 {
        pos1.y + region1.y - pos2.y + region2.y
    } else if direction1.y < 0.0 {
        pos2.y + region2.y - pos1.y + region1.y
    } else {
        0.0
    };

    let d = Vector2 {
        x: direction1.x.abs(),
        y: direction1.y.abs(),
    };
    if wx * d.y <= wy * d.x {
        pos1.x -= direction1.x.signum() * wx;
        pos1.y -= direction1.y * wx / d.x;
    } else {
        pos1.y -= direction1.y.signum() * wy;
        pos1.x -= direction1.x * wy / d.y;
    }

    true
}
