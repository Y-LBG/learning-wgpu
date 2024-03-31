struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) vertex_position: vec3<f32>,
    @location(1) color: vec4<f32>,
};

// Vertex shader
@vertex
fn vs_main(
    @builtin(vertex_index) in_vertex_index: u32,
) -> VertexOutput {
    let x = f32(1 - i32(in_vertex_index)) * 0.5;
    let y = f32(i32(in_vertex_index & 1u) * 2 - 1) * 0.5;

    var color = array<vec4f, 3>(
          vec4f(1, 0, 0, 1), // red
          vec4f(0, 1, 0, 1), // green
          vec4f(0, 0, 1, 1), // blue
        );
    
    var out: VertexOutput;
    out.position = vec4<f32>(x, y, 0.0, 1.0);
    out.vertex_position = out.position.xyz;
    out.color = color[in_vertex_index];
    return out;
}

// Fragment shader
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Fixed color for our triangle
    // return vec4<f32>(1.0, 0.5, 0.1, 1.0);

    // @builtin(position) has a different meaning in the fragment shader, than in the vertex shader
    // Here, it's the position of the fragment in the screen (i.e. the pixel position)
    // return vec4<f32>(in.position.xy-200, 0.1, 1.0);

    // Using the actual position of the vertices
    // return vec4<f32>(in.vertex_position, 1.0);

    return in.color;
}