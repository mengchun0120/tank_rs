use cgmath::Vector2;

pub fn check_obj_collide(
    pos1: &mut Vector2<f32>,
    span1: f32,
    pass1: bool,
    direction1: &Vector2<f32>,
    pos2: &Vector2<f32>,
    span2: f32,
    pass2: bool,
) -> bool {
    if pass1 && pass2 {
        return false;
    }

    let not_collide = pos1.x - span1 >= pos2.x + span2
        || pos1.x + span1 <= pos2.x - span2
        || pos1.y - span1 >= pos2.y + span2
        || pos1.y + span1 <= pos2.y - span2;

    if not_collide {
        return false;
    }

    if pass1 || pass2 || (direction1.x == 0.0 && direction1.y == 0.0) {
        return true;
    }

    let span_sum = span1 + span2;
    let d = Vector2 {
        x: direction1.x.abs(),
        y: direction1.y.abs(),
    };
    if d.x > 0.0 && d.y > 0.0 {
        let wx = direction1.x.signum() * (pos1.x - pos2.x) + span_sum;
        let wy = direction1.y.signum() * (pos1.y - pos2.y) + span_sum;

        if wx * d.y <= wy * d.x {
            pos1.x -= direction1.x.signum() * wx;
            pos1.y -= direction1.y * wx / d.x;
        } else {
            pos1.y -= direction1.y.signum() * wy;
            pos1.x -= direction1.x * wy / d.y;
        }
    } else if d.y == 0.0 {
        pos1.x = pos2.x - direction1.x.signum() * span_sum;
    } else {
        pos1.y = pos2.y - direction1.y.signum() * span_sum;
    }

    true
}
