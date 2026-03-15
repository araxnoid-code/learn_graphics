struct VertexOutput {
    @builtin(position) position: vec4<f32>}

@vertex
fn main_vertex(
    @builtin(vertex_index) vertex_idx: u32
) -> VertexOutput {
    var out: VertexOutput;
    let x = f32(1 - i32(vertex_idx)) * 0.5;
    let y = (-1 + f32(vertex_idx & 1u) * 2) * 0.5;
    out.position = vec4(x, y, 1, 1);

    return out;
}

@fragment
fn main_fragement(
    in: VertexOutput
) -> @location(0) vec4<f32> {
    return vec4(0.5, 0.5, 0, 1.);
}
