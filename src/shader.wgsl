struct Uniforms {
    color: vec4f,
    resolution: vec2f,
    translation: vec2f,
}

struct Vertex {
    @location(0) position: vec2f,
};

struct VSOutput {
    @builtin(position) position: vec4f,
}

@group(0) @binding(0) var<uniform> uni: Uniforms;

@vertex fn vs(vert: Vertex) -> VSOutput {
    var vsOut: VSOutput;

    let position = vert.position + uni.translation;

    // converting pixel space to clip space
    let zeroToOne = position / uni.resolution;

    let zeroToTwo = zeroToOne * 2.0;

    let flippedClipSpace = zeroToTwo - 1.0;

    let clipSpace = flippedClipSpace * vec2f(1, -1);

    vsOut.position = vec4f(clipSpace, 0.0, 1.0);
    return vsOut;
}

@fragment fn fs() -> @location(0) vec4f {
    return uni.color;
}
