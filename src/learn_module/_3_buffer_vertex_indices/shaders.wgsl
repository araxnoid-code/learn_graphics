struct MyVertex {
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>}

@vertex
fn main_vertex(
    vertex_in: MyVertex
) -> VertexOutput {
    var out: VertexOutput;
    out.position = vec4(vertex_in.position, 1.);
    out.color = vec4(vertex_in.color, 1.);

    return out;
}

@fragment
fn main_fragement(
    in: VertexOutput
) -> @location(0) vec4<f32> {
    return in.color;
}
