// 頂点シェーダ

// カメラ
struct CameraUniform {
    position: vec2<f32>, 
    size: vec2<f32>, 
    rotation: vec2<f32>, 
    reserved: vec2<f32>, 
}
@group(0) @binding(0)
var<uniform> camera: CameraUniform;

struct VertexInput {
    @location(0) position: vec4<f32>, 
    @location(1) tex_coords: vec2<f32>, 
}

struct InstanceInput {
    @location(5) position: vec2<f32>, 
    @location(6) size: vec2<f32>, 
    @location(7) rotation: vec2<f32>, 
    @location(8) color: vec4<f32>, 
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>, 
    @location(0) color: vec4<f32>, 
}

@vertex
fn vs_main(
    model: VertexInput, 
    instance: InstanceInput, 
) -> VertexOutput {
    var out: VertexOutput;
    out.color = instance.color;
    var pos_temp: vec4<f32>;
    pos_temp = vec4<f32>(
        instance.size.x * 0.5 * model.position.x,  
        instance.size.y * 0.5 * model.position.y, 
        0., 
        1., 
    );
    pos_temp = vec4<f32>(
        pos_temp.x * instance.rotation.x - pos_temp.y * instance.rotation.y, 
        pos_temp.x * instance.rotation.y + pos_temp.y * instance.rotation.x,  
        0., 
        1., 
    );
    pos_temp += vec4<f32>(
        instance.position.x, 
        instance.position.y, 
        0., 
        0., 
    );
    pos_temp -= vec4<f32>(camera.position.x, camera.position.y, 0., 0.);
    pos_temp = vec4<f32>(
        pos_temp.x * camera.rotation.x - pos_temp.y * camera.rotation.y, 
        pos_temp.x * camera.rotation.y + pos_temp.y * camera.rotation.x, 
        0., 
        1., 
    );
    pos_temp *= vec4<f32>(camera.size.x, camera.size.y, 0., 1.);
    out.clip_position = pos_temp;
    return out;
}

// フラグメントシェーダ

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}