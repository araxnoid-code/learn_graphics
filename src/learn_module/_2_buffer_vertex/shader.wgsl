struct VertexInput{
    @location(0) postion: vec3<f32>,
    @location(1) color: vec3<f32>,
}

struct VertexOutput{
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec3<f32>
}


@vertex
fn vertex_main(
    input_vertex: VertexInput
) -> VertexOutput{
    var output: VertexOutput;
    output.position = vec4(input_vertex.postion, 1.);
    output.color = input_vertex.color;

    return output;
}

@fragment
fn fragment_main(
    in: VertexOutput
) -> @location(0) vec4<f32>{
    return vec4(in.color, 1.0);
}
