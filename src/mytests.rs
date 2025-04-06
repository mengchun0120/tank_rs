#[cfg(test)]
mod test_vertex_data_block {
    use crate::myopengl::{VertexDataBlock, interleave_vertex_data};

    #[test]
    fn new_vertex_data_block_fails() {
        let data: &[f32] = &[];
        let block = VertexDataBlock::new(0, data);
        assert!(block.is_err());

        let block = VertexDataBlock::new(2, data);
        assert!(block.is_err());

        let data: &[f32] = &[1.0, 2.0, 3.0];
        let block = VertexDataBlock::new(2, data);
        assert!(block.is_err());
    }

    #[test]
    fn vertex_data_block_works() {
        let data: &[f32] = &[1.0, 2.0, 3.0, 4.0];
        let block = VertexDataBlock::new(2, data).unwrap();
        assert_eq!(block.vertex_size(), 2);
        assert_eq!(block.data(), vec![1.0, 2.0, 3.0, 4.0]);
        assert_eq!(block.num_of_vertices(), 2);
    }

    #[test]
    fn vertex_data_block_get_slice_works() {
        let data: &[f32] = &[1.0, 2.0, 3.0, 4.0];
        let block = VertexDataBlock::new(2, data).unwrap();
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
        let block = VertexDataBlock::new(2, data).unwrap();
        let slice = block.get_slice(3);
        assert!(slice.is_err());
    }

    #[test]
    fn interleave_vertex_data_works() {
        let data1: &[f32] = &[1.0, 2.0, 3.0, 4.0];
        let blocks = vec![VertexDataBlock::new(2, data1).unwrap()];
        let result = interleave_vertex_data(&blocks).unwrap();
        assert_eq!(result, vec![1.0, 2.0, 3.0, 4.0]);

        let data2: &[f32] = &[2.0, 3.0, 4.0, 5.0, 6.0, 7.0];
        let blocks = vec![
            VertexDataBlock::new(2, data1).unwrap(),
            VertexDataBlock::new(3, data2).unwrap(),
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
            VertexDataBlock::new(2, data1).unwrap(),
            VertexDataBlock::new(3, data2).unwrap(),
        ];
        let result = interleave_vertex_data(&blocks);
        assert!(result.is_err());
    }
}
