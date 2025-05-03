#[cfg(test)]
mod test_vertex_data_block {
    use crate::myopengl::{VertexDataBlock, interleave_vertex_data};

    #[test]
    fn new_vertex_data_block_fails() {
        let data: &[f32] = &[];
        let block = VertexDataBlock::new("", 0, data);
        assert!(block.is_err());

        let block = VertexDataBlock::new("", 2, data);
        assert!(block.is_err());

        let data: &[f32] = &[1.0, 2.0, 3.0];
        let block = VertexDataBlock::new("", 2, data);
        assert!(block.is_err());
    }

    #[test]
    fn vertex_data_block_works() {
        let data: &[f32] = &[1.0, 2.0, 3.0, 4.0];
        let block = VertexDataBlock::new("", 2, data).unwrap();
        assert_eq!(block.vertex_size(), 2);
        assert_eq!(block.data(), vec![1.0, 2.0, 3.0, 4.0]);
        assert_eq!(block.num_of_vertices(), 2);
    }

    #[test]
    fn vertex_data_block_get_slice_works() {
        let data: &[f32] = &[1.0, 2.0, 3.0, 4.0];
        let block = VertexDataBlock::new("", 2, data).unwrap();
        let slice = block.get_slice(0).unwrap();
        assert_eq!(slice, &[1.0, 2.0]);
        let slice = block.get_slice(1).unwrap();
        assert_eq!(slice, &[2.0, 3.0]);
        let slice = block.get_slice(2).unwrap();
        assert_eq!(slice, &[3.0, 4.0]);
    }

    #[test]
    fn vertex_data_block_get_slice_fails() {
        let data: &[f32] = &[1.0, 2.0, 3.0, 4.0];
        let block = VertexDataBlock::new("", 2, data).unwrap();
        let slice = block.get_slice(3);
        assert!(slice.is_err());
    }

    #[test]
    fn vertex_data_block_from_json_works() {
        let json_data = r#"
        {
            "name": "test",
            "vertex_size": 2,
            "data": [1.0, 2.0, 3.0, 4.0]
        }
        "#;
        let json_value = json::parse(json_data).unwrap();
        let block = VertexDataBlock::from_json(&json_value).unwrap();
        assert_eq!(block.name(), "test");
        assert_eq!(block.vertex_size(), 2);
        assert_eq!(block.data(), vec![1.0, 2.0, 3.0, 4.0]);
    }

    #[test]
    fn vertex_data_block_from_json_fails() {
        let json_data = r#"
        {
            "vertex_size": 0,
            "data": [1.0, 2.0, 3.0]
        }
        "#;
        let json_value = json::parse(json_data).unwrap();
        let block = VertexDataBlock::from_json(&json_value);
        assert!(block.is_err());

        let json_data = r#"
        {
            "name": "test",
            "vertex_size": 0,
            "data": [1.0, 2.0, 3.0]
        }
        "#;
        let json_value = json::parse(json_data).unwrap();
        let block = VertexDataBlock::from_json(&json_value);
        assert!(block.is_err());

        let json_data = r#"
        {
            "name": "test",
            "vertex_size": 2,
            "data": [1.0, 2.0, 3.0]
        }
        "#;
        let json_value = json::parse(json_data).unwrap();
        let block = VertexDataBlock::from_json(&json_value);
        assert!(block.is_err());

        let json_data = r#"
        {
            "name": "test",
            "vertex_size": 2,
            "data": []
        }
        "#;
        let json_value = json::parse(json_data).unwrap();
        let block = VertexDataBlock::from_json(&json_value);
        assert!(block.is_err());
    }

    #[test]
    fn interleave_vertex_data_works() {
        let data1: &[f32] = &[1.0, 2.0, 3.0, 4.0];
        let blocks = vec![VertexDataBlock::new("", 2, data1).unwrap()];
        let result = interleave_vertex_data(&blocks).unwrap();
        assert_eq!(result, vec![1.0, 2.0, 3.0, 4.0]);

        let data2: &[f32] = &[2.0, 3.0, 4.0, 5.0, 6.0, 7.0];
        let blocks = vec![
            VertexDataBlock::new("", 2, data1).unwrap(),
            VertexDataBlock::new("", 3, data2).unwrap(),
        ];
        let result = interleave_vertex_data(&blocks).unwrap();
        assert_eq!(
            result,
            vec![1.0, 2.0, 2.0, 3.0, 4.0, 3.0, 4.0, 5.0, 6.0, 7.0]
        );
    }

    #[test]
    fn interleave_vertex_data_fails() {
        let data1: &[f32] = &[1.0, 2.0, 3.0, 4.0];
        let data2: &[f32] = &[2.0, 3.0, 4.0];
        let blocks = vec![
            VertexDataBlock::new("", 2, data1).unwrap(),
            VertexDataBlock::new("", 3, data2).unwrap(),
        ];
        let result = interleave_vertex_data(&blocks);
        assert!(result.is_err());
    }
}

#[cfg(test)]
mod test_color_from_json {
    use crate::myjsonutils::{alpha_from_json, rgb_from_json, rgba_from_json};

    #[test]
    fn rgb_from_json_works() {
        let json_data = r#"[255, 0, 0]"#;
        let json_value = json::parse(json_data).unwrap();
        let color = rgb_from_json(&json_value).unwrap();
        assert_eq!(color.x, 1.0);
        assert_eq!(color.y, 0.0);
        assert_eq!(color.z, 0.0);
    }

    #[test]
    fn rgb_from_json_fails() {
        let json_data = r#"[255, 0]"#;
        let json_value = json::parse(json_data).unwrap();
        let color = rgb_from_json(&json_value);
        assert!(color.is_err());
    }

    #[test]
    fn rgba_from_json_works() {
        let json_data = r#"[255, 0, 0, 255]"#;
        let json_value = json::parse(json_data).unwrap();
        let color = rgba_from_json(&json_value).unwrap();
        assert_eq!(color.x, 1.0);
        assert_eq!(color.y, 0.0);
        assert_eq!(color.z, 0.0);
        assert_eq!(color.w, 1.0);
    }

    #[test]
    fn rgba_from_json_fails() {
        let json_data = r#"[255, 0]"#;
        let json_value = json::parse(json_data).unwrap();
        let color = rgba_from_json(&json_value);
        assert!(color.is_err());
    }

    #[test]
    fn alpha_from_json_works() {
        let json_data = r#"255"#;
        let json_value = json::parse(json_data).unwrap();
        let alpha = alpha_from_json(&json_value).unwrap();
        assert_eq!(alpha, 1.0);
    }

    #[test]
    fn alpha_from_json_fails() {
        let json_data = r#"256"#;
        let json_value = json::parse(json_data).unwrap();
        let alpha = alpha_from_json(&json_value);
        assert!(alpha.is_err());
    }
}

#[cfg(test)]
mod test_collide {
    use crate::mycollide::{check_collide_nonpass, check_collide_bound_nonpass};
    use cgmath::Vector2;

    #[test]
    fn check_collide_nonpass_invaild_param() {
        let pos1 = Vector2::new(10.0, 10.0);
        let v = Vector2::new(0.0, 0.0);
        let span1 = 10.0;
        let pos2 = Vector2::new(40.0, 40.0);
        let span2 = 20.0;
        let time_delta = 10.0;

        let result = check_collide_nonpass(&pos1, span1, &v, &pos2, span2, time_delta);
        assert!(result.is_err());

        let v = Vector2::new(1.0, 1.0);
        let time_delta = 0.0;
        let result = check_collide_nonpass(&pos1, span1, &v, &pos2, span2, time_delta);
        assert!(result.is_err());

        let time_delta = 10.0;
        let span1 = 0.0;
        let result = check_collide_nonpass(&pos1, span1, &v, &pos2, span2, time_delta);
        assert!(result.is_err());

        let span1 = 10.0;
        let pos2 = Vector2::new(15.0, 15.0);
        let result = check_collide_nonpass(&pos1, span1, &v, &pos2, span2, time_delta);
        assert!(result.is_err());
    }

    #[test]
    fn check_collide_nonpass_return_none() {
        let pos1 = Vector2::new(10.0, 10.0);
        let v = Vector2::new(-1.0, 1.0);
        let span1 = 10.0;
        let pos2 = Vector2::new(40.0, 60.0);
        let span2 = 20.0;
        let time_delta = 10.0;

        let result = check_collide_nonpass(&pos1, span1, &v, &pos2, span2, time_delta).unwrap();
        assert!(result.is_none());

        let v = Vector2::new(0.0, 1.0);
        let result = check_collide_nonpass(&pos1, span1, &v, &pos2, span2, time_delta).unwrap();
        assert!(result.is_none());

        let pos2 = Vector2::new(15.0, 60.0);
        let v = Vector2::new(1.0, 0.0);
        let result = check_collide_nonpass(&pos1, span1, &v, &pos2, span2, time_delta).unwrap();
        assert!(result.is_none());

        let pos2 = Vector2::new(40.0, 60.0);
        let v = Vector2::new(1.0, 2.0);
        let time_delta = 5.0;
        let result = check_collide_nonpass(&pos1, span1, &v, &pos2, span2, time_delta).unwrap();
        assert!(result.is_none());

        let v = Vector2::new(1.0, 2.0);
        let time_delta = 5.0;
        let result = check_collide_nonpass(&pos1, span1, &v, &pos2, span2, time_delta).unwrap();
        assert!(result.is_none());

        let v = Vector2::new(6.0, 2.0);
        let time_delta = 12.0;
        let result = check_collide_nonpass(&pos1, span1, &v, &pos2, span2, time_delta).unwrap();
        assert!(result.is_none());

        let pos1 = Vector2::new(80.0, 70.0);
        let v = Vector2::new(-2.0, -7.0);
        let pos2 = Vector2::new(30.0, 30.0);
        let time_delta = 12.0;
        let result = check_collide_nonpass(&pos1, span1, &v, &pos2, span2, time_delta).unwrap();
        assert!(result.is_none());

        let pos1 = Vector2::new(50.0, 80.0);
        let v = Vector2::new(0.0, -2.0);
        let time_delta = 5.0;
        let result = check_collide_nonpass(&pos1, span1, &v, &pos2, span2, time_delta).unwrap();
        assert!(result.is_none());

        let pos1 = Vector2::new(70.0, 130.0);
        let v = Vector2::new(0.0, -1.0);
        let pos2 = Vector2::new(100.0, 100.0);
        let time_delta = 10.0;
        let result = check_collide_nonpass(&pos1, span1, &v, &pos2, span2, time_delta).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn check_collide_nonpass_return_some() {
        let pos1 = Vector2::new(100.0, 100.0);
        let span1 = 10.0;
        let v = Vector2::new(-6.0, 5.0);
        let pos2 = Vector2::new(60.0, 140.0);
        let span2 = 20.0;
        let time_delta = 3.0;

        let result = check_collide_nonpass(&pos1, span1, &v, &pos2, span2, time_delta)
            .unwrap()
            .unwrap();
        assert!(result == 2.0);

        let pos2 = Vector2::new(150.0, 140.0);
        let v = Vector2::new(4.0, 3.0);
        let time_delta = 6.0;
        let result = check_collide_nonpass(&pos1, span1, &v, &pos2, span2, time_delta)
            .unwrap()
            .unwrap();
        assert!(result == 5.0);

        let pos2 = Vector2::new(140.0, 60.0);
        let v = Vector2::new(5.0, -5.0);
        let time_delta = 3.0;
        let result = check_collide_nonpass(&pos1, span1, &v, &pos2, span2, time_delta)
            .unwrap()
            .unwrap();
        assert!(result == 2.0);

        let pos2 = Vector2::new(80.0, 140.0);
        let v = Vector2::new(-3.0, 5.0);
        let time_delta = 3.0;
        let result = check_collide_nonpass(&pos1, span1, &v, &pos2, span2, time_delta)
            .unwrap()
            .unwrap();
        assert!(result == 2.0);

        let pos2 = Vector2::new(150.0, 80.0);
        let v = Vector2::new(5.0, -5.0);
        let time_delta = 5.0;
        let result = check_collide_nonpass(&pos1, span1, &v, &pos2, span2, time_delta)
            .unwrap()
            .unwrap();
        assert!(result == 4.0);

        let pos2 = Vector2::new(100.0, 60.0);
        let v = Vector2::new(0.0, -5.0);
        let time_delta = 3.0;
        let result = check_collide_nonpass(&pos1, span1, &v, &pos2, span2, time_delta)
            .unwrap()
            .unwrap();
        assert!(result == 2.0);

        let pos2 = Vector2::new(60.0, 80.0);
        let v = Vector2::new(-5.0, 0.0);
        let time_delta = 4.0;
        let result = check_collide_nonpass(&pos1, span1, &v, &pos2, span2, time_delta)
            .unwrap()
            .unwrap();
        assert!(result == 2.0);
    }

    #[test]
    fn check_collide_bound_nonpass_return_err() {
        let pos = Vector2::new(20.0, 20.0);
        let span = 0.0;
        let v = Vector2::new(1.0, 0.0);
        let bound = Vector2::new(400.0, 400.0);
        let time_delta = 4.0;

        let result = check_collide_bound_nonpass(&pos, span, &v, &bound, time_delta);
        assert!(result.is_err());

        let span = 10.0;
        let v = Vector2::new(0.0, 0.0);
        let result = check_collide_bound_nonpass(&pos, span, &v, &bound, time_delta);
        assert!(result.is_err());

        let v = Vector2::new(1.0, 0.0);
        let bound = Vector2::new(-1.0, 0.0);
        let result = check_collide_bound_nonpass(&pos, span, &v, &bound, time_delta);
        assert!(result.is_err());

        let bound = Vector2::new(400.0, 400.0);
        let time_delta = -1.0;
        let result = check_collide_bound_nonpass(&pos, span, &v, &bound, time_delta);
        assert!(result.is_err());

        let pos = Vector2::new(0.0, 0.0);
        let time_delta = 3.0;
        let result = check_collide_bound_nonpass(&pos, span, &v, &bound, time_delta);
        assert!(result.is_err());

        let pos = Vector2::new(0.0, 400.0);
        let result = check_collide_bound_nonpass(&pos, span, &v, &bound, time_delta);
        assert!(result.is_err());
    }

    #[test]
    fn check_collide_bound_nonpass_return_non() {
        let pos = Vector2::new(380.0, 380.0);
        let span = 10.0;
        let v = Vector2::new(1.0, 1.0);
        let bound = Vector2::new(400.0, 400.0);
        let time_delta = 4.0;

        let result = check_collide_bound_nonpass(&pos, span, &v, &bound, time_delta).unwrap();
        assert!(result.is_none());

        let pos = Vector2::new(30.0, 200.0);
        let v = Vector2::new(-1.0, 0.0);
        let result = check_collide_bound_nonpass(&pos, span, &v, &bound, time_delta).unwrap();
        assert!(result.is_none());

        let pos = Vector2::new(200.0, 20.0);
        let v = Vector2::new(0.0, -1.0);
        let result = check_collide_bound_nonpass(&pos, span, &v, &bound, time_delta).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn check_collide_bound_nonpass_return_some() {
        let pos = Vector2::new(380.0, 380.0);
        let span = 10.0;
        let v = Vector2::new(5.0, 2.0);
        let bound = Vector2::new(400.0, 400.0);
        let time_delta = 4.0;

        let result = check_collide_bound_nonpass(&pos, span, &v, &bound, time_delta).unwrap().unwrap();
        assert!(result == 2.0);

        let pos = Vector2::new(30.0, 200.0);
        let v = Vector2::new(-10.0, 0.0);
        let result = check_collide_bound_nonpass(&pos, span, &v, &bound, time_delta).unwrap().unwrap();
        assert!(result == 2.0);

        let pos = Vector2::new(200.0, 20.0);
        let v = Vector2::new(0.0, -10.0);
        let result = check_collide_bound_nonpass(&pos, span, &v, &bound, time_delta).unwrap().unwrap();
        assert!(result == 1.0);
    }
}
