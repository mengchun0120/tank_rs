use cgmath::{Vector2, Vector3, Vector4};
use json::JsonValue;
use std::{fs, path::Path};

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

    Ok(Vector3::new(color_parts[0], color_parts[1], color_parts[2]))
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

pub fn alpha_from_json(value: &JsonValue) -> Result<f32, MyError> {
    let a = value.as_u8().ok_or("Invalid alpha")?;
    Ok(a as f32 / 255.0)
}

pub fn json_from_file<T: AsRef<Path>>(path: T) -> Result<JsonValue, MyError> {
    let json_str = fs::read_to_string(path)?;
    let obj = json::parse(&json_str).map_err(|e| format!("Failed to parse JSON: {}", e))?;
    Ok(obj)
}

pub fn vector2_from_json(obj: &JsonValue) -> Result<Vector2<f32>, MyError> {
    let mut a = Vec::new();
    for m in obj.members() {
        let v = m.as_f32().ok_or("Invalid float")?;
        a.push(v);
    }

    if a.len() < 2 {
        return Err("Invalid JSON format".into());
    }

    Ok(Vector2 { x: a[0], y: a[1] })
}
