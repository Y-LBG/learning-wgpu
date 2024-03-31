// Vertex shader
struct VertIn {
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
};

struct VertOutFragIn {
    @builtin(position) position: vec4<f32>, // Reminder: The meaning/content differs when used as an "output in the vertex shader" vs. "input in the fragment shader"
    @location(0) color: vec3<f32>,
};

@vertex
fn vs_main(
    model: VertIn,
) -> VertOutFragIn {
    var out: VertOutFragIn;
    out.color = model.color;
    out.position = vec4<f32>(model.position, 1.0);
    return out;
}

// Fragment shader
@fragment
fn fs_main(in: VertOutFragIn) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color, 1.0);
}