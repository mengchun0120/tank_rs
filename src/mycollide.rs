use cgmath::Vector2;

pub fn check_collide_nonpass(
    pos1: &Vector2<f32>,
    span1: f32,
    direction1: &Vector2<f32>,
    pos2: &Vector2<f32>,
    span2: f32,
) -> Option<Vector2<f32>> {
    let not_collide = pos1.x - span1 >= pos2.x + span2
        || pos1.x + span1 <= pos2.x - span2
        || pos1.y - span1 >= pos2.y + span2
        || pos1.y + span1 <= pos2.y - span2;

    if not_collide {
        return None;
    }

    if direction1.x == 0.0 && direction1.y == 0.0 {
        return Some(Vector2::new(0.0, 0.0));
    }

    let span_sum = span1 + span2;
    let d = Vector2 {
        x: direction1.x.abs(),
        y: direction1.y.abs(),
    };
    let delta = if d.x > 0.0 && d.y > 0.0 {
        let wx = direction1.x.signum() * (pos1.x - pos2.x) + span_sum;
        let wy = direction1.y.signum() * (pos1.y - pos2.y) + span_sum;

        if wx * d.y <= wy * d.x {
            Vector2::new(wx, wx * d.y / d.x)
        } else {
            Vector2::new(wy * d.x / d.y, wy)
        }
    } else if d.y == 0.0 {
        let wx = direction1.x.signum() * (pos1.x - pos2.x) + span_sum;
        Vector2::new(wx, 0.0)
    } else {
        let wy = direction1.y.signum() * (pos1.y - pos2.y) + span_sum;
        Vector2::new(0.0, wy)
    };

    Some(delta)
}

pub fn check_collide_bound(
    pos: &mut Vector2<f32>,
    span: f32,
    direction: &Vector2<f32>,
    bound: &Vector2<f32>,
) -> bool {
    let not_collide = pos.x - span >= 0.0
        && pos.x + span <= bound.x
        && pos.y - span >= 0.0
        && pos.y + span <= bound.y;

    if not_collide {
        return false;
    }

    if direction.x == 0.0 && direction.y == 0.0 {
        return true;
    }

    let wx = if pos.x - span < 0.0 {
        span - pos.x
    } else if pos.x + span > bound.x {
        pos.x + span - bound.x
    } else {
        0.0
    };
    let wy = if pos.y - span < 0.0 {
        span - pos.y
    } else if pos.y + span > bound.y {
        pos.y + span - bound.y
    } else {
        0.0
    };

    let d = Vector2 {
        x: direction.x.abs(),
        y: direction.y.abs(),
    };

    if d.x > 0.0 && d.y > 0.0 {
        if wx * d.y <= wy * d.x {
            pos.y -= direction.y.signum() * wy;
            pos.x -= direction.x.signum() * wy * d.x / d.y;
        } else {
            pos.x -= direction.x.signum() * wx;
            pos.y -= direction.y.signum() * wx * d.y / d.x;
        }
    } else if d.y == 0.0 {
        pos.x -= direction.x.signum() * wx;
    } else {
        pos.y -= direction.y.signum() * wy;
    }

    true
}
