@vertex
fn vertex_main(
    @builtin(vertex_index) vertex_idx: u32
) -> @builtin(position) vec4<f32>{
    let x = f32(1 - i32(vertex_idx)) * 0.5;
    let y = (-1 + f32(vertex_idx & 1u) * 2) * 0.5;
    let clip_pos = vec4(x, y, 1, 1);
    return clip_pos;
}

@fragment
fn fragment_main(
    @builtin(position) in: vec4<f32>
) -> @location(0) vec4<f32> {
    return vec4(0.0, 1.0, 1.0, 1.0);
}
