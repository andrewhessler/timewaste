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
    //
    // Notes above are out of date, but I want to keep it for historical purposes
    #[rustfmt::skip]
    let vertex_data = vec![
        0., 0., 0., // 0
        100., 0., 0., // 1
        0., 100., 0., // 2
        0., 0., 100., // 3
        100., 100., 0., // 4
        100., 0., 100., // 5
        0., 100., 100., // 6
        100., 100., 100. // 7
    ];

    #[rustfmt::skip]
    let index_data = vec![
        // face front
        0, 1, 4,
        0, 4, 2,

        // face left
        0, 6, 3,
        0, 2, 6,

        // face right
        1, 5, 7,
        1, 7, 4,

        // // face back
        3, 7, 5,
        3, 6, 7,

        // // face top
        // 3, 4, 7,
        // 3, 7, 6,
        //
        // // face bottom
        // 0, 3, 5,
        // 0, 5, 2,
    ];

    let num_vertices = index_data.len() as u32;

    (vertex_data, index_data, num_vertices)
}

pub fn create_practice_vertices() -> (Vec<f32>, Vec<u32>, u32) {
    #[rustfmt::skip]
    let vertex_data = vec![
        0., 0., 0., // 0
        100., 0., 0., 0., 100., 0.,  // 1, 2
        100., 100., 0. // 3
    ];

    #[rustfmt::skip]
    let index_data = vec![
        // face front
        0, 1, 3,
        0, 3, 2,
    ];

    let num_vertices = index_data.len() as u32;

    (vertex_data, index_data, num_vertices)
}
