enable primitive_index;

struct Uniforms {
    color: vec4f,
    matrix: mat4x4f,
}

struct Vertex {
    @location(0) position: vec4f,
};

struct VSOutput {
    @builtin(position) position: vec4f,
}

@group(0) @binding(0) var<uniform> uni: Uniforms;
@group(1) @binding(0) var<storage, read_write> sto: array<vec4f>;

@vertex fn vs(vert: Vertex, @builtin(vertex_index) idx: u32) -> VSOutput {
    var vsOut: VSOutput;
    var mat = uni;

    vsOut.position = uni.matrix * vert.position;

    sto[idx] = vsOut.position;

    return vsOut;
}

@fragment fn fs(@builtin(primitive_index) tri_idx: u32) -> @location(0) vec4f {
    let colors = array<vec3f, 6>(
        vec3f(1.0, 0.0, 0.0),
        vec3f(0.0, 1.0, 0.0),
        vec3f(1.0, 0.0, 1.0),
        vec3f(1.0, 1.0, 0.0),
        vec3f(1.0, 0.0, 1.0),
        vec3f(0.0, 1.0, 1.0),
    );

    let color = colors[tri_idx % 6];
    return vec4f(color, 1);
}
