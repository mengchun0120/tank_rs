use cgmath::{Vector3, Vector4};
use json::JsonValue;

use crate::mytypes::MyError;

pub fn rgb_from_json(value: &JsonValue) -> Result<Vector3<f32>, MyError> {
    let mut color_parts = Vec::new();
    for i in value.members() {
        let part = i.as_u8().ok_or("Invalid color part")?;
        color_parts.push(part as f32 / 255.0);
    }

    if color_parts.len() != 3 {
        return Err("Invalid color length".into());
    }

    Ok(Vector3::new(
        color_parts[0],
        color_parts[1],
        color_parts[2],
    ))
}

pub fn rgba_from_json(value: &JsonValue) -> Result<Vector4<f32>, MyError> {
    let mut color_parts = Vec::new();
    for i in value.members() {
        let part = i.as_u8().ok_or("Invalid color part")?;
        color_parts.push(part as f32 / 255.0);
    }

    if color_parts.len() != 4 {
        return Err("Invalid color length".into());
    }

    Ok(Vector4::new(
        color_parts[0],
        color_parts[1],
        color_parts[2],
        color_parts[3],
    ))
}