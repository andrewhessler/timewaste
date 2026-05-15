pub fn create_cube_vertices() -> (Vec<f32>, Vec<u32>, u32) {
    // // [x, y, z]
    // // x -> <-
    // // y b f
    // // z ^ v
    // //   + -
    // // [], [], []
    // // face front
    // [0, 0, 0], [1, 0, 0], [1, 0, 1], // bottom right
    // [0, 0, 0], [1, 0, 1], [0, 0, 1], // top left
    //
    // // face left
    // [0, 0, 0], [0, 1, 1], [0, 1, 0], // bottom left
    // [0, 0, 0], [0, 0, 1], [0, 1, 1], // top right
    //
    // // face right
    // [1, 0, 0], [1, 1, 0], [1, 1, 1], // bottom right
    // [1, 0, 0], [1, 1, 1], [1, 0, 1], // top left
    //
    // // face back
    // [0, 1, 0], [1, 1, 1], [1, 1, 0], // bottom left
    // [0, 1, 0], [0, 1, 1], [1, 1, 1], // top right
    //
    // // face top (top down)
    // [0, 0, 1], [1, 0, 1], [1, 1, 1], // bottom right
    // [0, 0, 1], [1, 1, 1], [0, 1, 1], // top left
    //
    // // face bottom (bottom up)
    // [0, 0, 0], [0, 0, 1], [1, 1, 0], // bottom left
    // [0, 0, 0], [1, 1, 0], [0, 1, 0], // top right
    #[rustfmt::skip]
    let vertex_data = vec![
        0., 0., 0., // 0
        1., 0., 0., 0., 1., 0., 0., 0., 1., // 1, 2, 3
        1., 0., 1., 1., 1., 0., 0., 1., 1., // 4, 5, 6
        1., 1., 1. // 7
    ];

    #[rustfmt::skip]
    let index_data = vec![
        // face front
        0, 1, 4,
        0, 4, 3,

        // face left
        0, 6, 2,
        0, 3, 6,

        // face right
        1, 5, 7,
        1, 7, 4,
        
        // face back
        2, 7, 5,
        2, 6, 7,

        // face top
        3, 4, 7,
        3, 7, 6,

        // face bottom
        0, 3, 5,
        0, 5, 2,
    ];

    let num_vertices = index_data.len() as u32;

    (vertex_data, index_data, num_vertices)
}
