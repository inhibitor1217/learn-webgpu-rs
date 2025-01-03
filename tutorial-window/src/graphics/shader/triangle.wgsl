struct VertexOutput {
  @builtin(position) clip_position: vec4<f32>,
}

@vertex
fn vertex_main(
  @builtin(vertex_index) in_vertex_index: u32,
) -> VertexOutput {
  var output: VertexOutput;
  let x = f32(1 - i32(in_vertex_index)) * 0.5;
  let y = f32(i32(in_vertex_index & 1u) * 2 - 1) * 0.5;
  output.clip_position = vec4<f32>(x, y, 0.0, 1.0);
  return output;
}

@fragment
fn fragment_main(in: VertexOutput) -> @location(0) vec4<f32> {
  return vec4<f32>(1.0, 0.0, 0.0, 1.0);
}
