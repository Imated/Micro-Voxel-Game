struct Ray {
    origin: vec3<f32>,
    direction: vec3<f32>,
};


@group(0) @binding(0)
var output : texture_storage_2d<rgba16float, write>;

@compute
@workgroup_size(8, 8)
fn main(@builtin(global_invocation_id) id : vec3<u32>) {
    let size = textureDimensions(output);

    if (id.x >= size.x || id.y >= size.y) {
        return;
    }

    let uv = vec2<f32>(id.xy) / vec2<f32>(size);
    let uv_centered = uv * 2.0 - 1.0;

    let aspect = f32(size.x) / f32(size.y);

    const fov = 67.0;
    const focal_length = 1 / tan(radians(fov) / 2);
    let camera_pos: vec3<f32> = vec3(0);
    let direction = vec3<f32>(uv_centered.x * aspect, uv_centered.y, -focal_length);

    var ray = Ray(camera_pos, direction);
    let color = ray_color(ray);
    textureStore(output, vec2<i32>(id.xy), color);
}

fn ray_color(ray: Ray) -> vec4<f32> {
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
