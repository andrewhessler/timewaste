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

    vsOut.position = vert.position - vec4f(0.5, 0.5, 0.0, 0.0);

    sto[idx] = vsOut.position;

    return vsOut;
}

@fragment fn fs() -> @location(0) vec4f {
    return vec4f(1, 0, 0, 1);
}
