use std::f32::consts::PI;

#[derive(Default)]
pub struct CircleVerticesInput {
    pub radius: Option<f32>,
    pub subdivisions: Option<u32>,
    pub inner_radius: Option<f32>,
    pub start_angle: Option<f32>,
    pub end_angle: Option<f32>,
}

pub fn create_circle_vertices(
    CircleVerticesInput {
        radius,
        subdivisions,
        inner_radius,
        start_angle,
        end_angle,
    }: CircleVerticesInput,
) -> (Vec<f32>, u32) {
    let radius = radius.unwrap_or(0.3);
    let subdivisions = subdivisions.unwrap_or(72);
    let inner_radius = inner_radius.unwrap_or(0.28);
    let start_angle = start_angle.unwrap_or(0.0);
    let end_angle = end_angle.unwrap_or(PI * 2.0);

    // 2 triangles per subdivison, 3 vertices per triangle, 2 values (xy)
    let num_vertices = subdivisions * 3 * 2;
    let mut vertex_data: Vec<f32> = Vec::with_capacity((num_vertices * 2) as usize);

    for i in 0..subdivisions {
        let angle1 =
            start_angle + (i as f32 + 0.0) * (end_angle - start_angle) / subdivisions as f32;
        let angle2 =
            start_angle + (i as f32 + 1.0) * (end_angle - start_angle) / subdivisions as f32;

        let c1 = angle1.cos();
        let s1 = angle1.sin();
        let c2 = angle2.cos();
        let s2 = angle2.sin();

        // triangle 1
        vertex_data.push(c1 * radius); // 0 then 12
        vertex_data.push(s1 * radius);
        vertex_data.push(c2 * radius);
        vertex_data.push(s2 * radius);
        vertex_data.push(c1 * inner_radius);
        vertex_data.push(s1 * inner_radius);

        // triangle 2
        vertex_data.push(c1 * inner_radius);
        vertex_data.push(s1 * inner_radius);
        vertex_data.push(c2 * radius);
        vertex_data.push(s2 * radius);
        vertex_data.push(c2 * inner_radius);
        vertex_data.push(s2 * inner_radius);
    }

    (vertex_data, num_vertices)
}

pub fn create_f_vertices() -> (Vec<f32>, Vec<u32>, u32) {
    let vertex_data = vec![
        // left column
        0., 0., 30., 0., 0., 150., 30., 150., // top rung
        30., 0., 100., 0., 30., 30., 100., 30., // middle rung
        30., 60., 70., 60., 30., 90., 70., 90.,
    ];

    let index_data = vec![
        0, 1, 2, 2, 1, 3, // left rung
        4, 5, 6, 6, 5, 7, // top rung
        8, 9, 10, 10, 9, 11, // middle rung
    ];

    let num_vertices = index_data.len() as u32;

    (vertex_data, index_data, num_vertices)
}
