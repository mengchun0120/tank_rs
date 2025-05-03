use std::f32::INFINITY;

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
        let msg = format!(
            "check_collide_nonpass: Invalid parameter: time_delta: {}, span1: {}, span2: {}, v: {:?}",
            time_delta, span1, span2, v
        );
        error!("{}", msg);
        return Err(msg.into());
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

pub fn check_collide_bound_nonpass(
    pos: &Vector2<f32>,
    span: f32,
    v: &Vector2<f32>,
    bound: &Vector2<f32>,
    time_delta: f32,
) -> Result<Option<f32>, MyError> {
    if span <= 0.0  || v.is_zero() || bound.x <= 0.0 || bound.y <= 0.0 || time_delta <= 0.0 {
        let msg = format!(
            "check_collide_bound: Invalid parameters: span={} v={:?} bound={:?} time_delta={}",
            span, v, bound, time_delta
        );
        error!("{}", msg);
        return Err(msg.into());
    }

    if pos.x - span <= 0.0
        || pos.x + span >= bound.x
        || pos.y - span <= 0.0
        || pos.y + span >= bound.y
    {
        return Err("check_collide_bound_nonpass: object is already outside bound".into());
    }

    let tx = if v.x < 0.0 {
        (pos.x - span) / v.x.abs()
    } else if v.x > 0.0 {
        (bound.x - pos.x - span) / v.x
    } else {
        INFINITY
    };

    let ty = if v.y < 0.0 {
        (pos.y - span) / v.y.abs()
    } else if v.y > 0.0 {
        (bound.y - pos.y - span) / v.y
    } else {
        INFINITY
    };

    let t = tx.min(ty);
    if t < time_delta {
        Ok(Some(t))
    } else {
        Ok(None)
    }
}

