use crate::mytypes::MyError;
use cgmath::{Vector2, Zero};
use log::error;

pub fn check_collide_nonpass(
    pos1: &Vector2<f32>,
    span1: f32,
    v: &Vector2<f32>,
    pos2: &Vector2<f32>,
    span2: f32,
    time_delta: f32,
) -> Result<Option<f32>, MyError> {
    if time_delta <= 0.0 || span1 <= 0.0 || span2 <= 0.0 || v.is_zero() {
        error!(
            "check_collide_nonpass: Invalid parameters: time_delta: {}, span1: {}, span2: {}, v: {:?}",
            time_delta, span1, span2, v
        );
        return Err("check_collide_nonpass: Invalid parameter".into());
    }

    let v1 = Vector2::new(v.x.abs(), v.y.abs());
    let span_sum = span1 + span2;

    let wx = pos2.x - pos1.x;
    let dx = wx.abs() - span_sum;
    let tx = if dx >= 0.0 {
        if v.x.signum() != wx.signum() {
            return Ok(None);
        }
        dx / v1.x
    } else {
        -1.0
    };

    let wy = pos2.y - pos1.y;
    let dy = wy.abs() - span_sum;
    let ty = if dy >= 0.0 {
        if v.y.signum() != wy.signum() {
            return Ok(None);
        }
        dy / v1.y
    } else {
        -1.0
    };

    if tx >= 0.0 || ty >= 0.0 {
        if tx > ty {
            if tx < time_delta && v1.y * tx < v.y.signum() * wy + span_sum {
                Ok(Some(tx))
            } else {
                Ok(None)
            }
        } else if tx < ty {
            if ty < time_delta && v1.x * ty < v.x.signum() * wx + span_sum {
                Ok(Some(ty))
            } else {
                Ok(None)
            }
        } else {
            if tx < time_delta {
                Ok(Some(tx))
            } else {
                Ok(None)
            }
        }
    } else {
        Err("check_collide_nonpass: objects are already colliding".into())
    }
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
