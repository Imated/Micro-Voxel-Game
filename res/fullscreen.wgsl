struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(1) uv: vec2<f32>,
};

@vertex
fn vs_main(
    @builtin(vertex_index) in_vertex_index: u32,
) -> VertexOutput {
    var out: VertexOutput;
    var uv: vec2<f32> = vec2<f32>(f32((in_vertex_index << 1) & 2), f32(in_vertex_index & 2));
    out.clip_position = vec4<f32>(uv * vec2<f32>(2, -2) + vec2<f32>(-1, 1), 0, 1);
    out.uv = uv;
    return out;
}

struct Ray {
    origin: vec3<f32>,
    direction: vec3<f32>,
};

@fragment
fn fs_main(
    in: VertexOutput
) -> @location(0) vec4<f32> {
    const screen_width: f32 = 800;
    const screen_height: f32 = 600;
    const aspect = screen_width / screen_height;
    const focal_len = 1;
    const camera_pos: vec3<f32> = vec3<f32>(0);
    let uv = in.uv;
    let centered = uv * 2.0 - 1.0;
    let screen_point = vec3<f32>(centered.x * aspect, -centered.y, -focal_len);

    var ray = Ray(camera_pos, screen_point - camera_pos);

    let t = ray_sphere_intersect(vec3(0, 0, -1), 0.5, ray);
    if (t > 0.0) {
        let N = normalize(ray.origin + ray.direction * t - vec3(0, 0, -1));
        return 0.5 * vec4(N + 1, 0);
    }

    let a = 0.5 * (normalize(ray.direction).y + 1.0);
    return mix(vec4<f32>(1), vec4<f32>(0.5, 0.7, 1.0, 1.0), a);
}

fn ray_sphere_intersect(center: vec3<f32>, radius: f32, ray: Ray) -> f32 {
    let oc = center - ray.origin;
    let a = length(ray.direction) * length(ray.direction);
    let h = dot(ray.direction, oc);
    let c = length(oc) * length(oc) - radius * radius;
    let discriminant = h * h - a * c;
    if (discriminant < 0) {
        return -1;
    } else {
        return (h - sqrt(discriminant)) / a;
    }
}
