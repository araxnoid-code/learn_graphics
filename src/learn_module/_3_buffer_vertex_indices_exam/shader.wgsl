struct My_Vertex {
    @location(0) pos: vec3<f32>,
    @location(1) color: vec3<f32>,
}

struct Pos {
    @builtin(position) pos: vec4<f32>,
    @location(0) color: vec4<f32>}

@vertex
fn main_vertex(my_vertex: My_Vertex) -> Pos {
    var pos: Pos;
    pos.pos = vec4(my_vertex.pos, 1.);
    pos.color = vec4(my_vertex.color, 1.);

    return pos;
}

@fragment
fn main_fragment(in: Pos) -> @location(0) vec4<f32> {
    return in.color;
}
