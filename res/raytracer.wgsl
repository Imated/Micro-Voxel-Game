struct Ray {
    origin: vec3<f32>,
    direction: vec3<f32>,
};

struct HitInfo {
    hit_point: vec3<f32>,
    hit_normal: vec3<f32>,
    t: f32,
    front_face: bool,
}

struct Camera {
    position: vec4<f32>,
    rotation: mat4x4<f32>,
}

struct Chunk {
    empty: u32,
}

const INFINITY: f32 = 3.402823466e+38;
const CHUNK_SIZE: f32 = 32.0;
const VOXEL_SIZE: f32 = 0.1;

@group(0) @binding(0)
var output : texture_storage_2d<rgba16float, write>;

@group(1) @binding(0)
var<uniform> camera : Camera;

@group(2) @binding(0)
var<storage, read> chunks : array<array<array<Chunk, 32>, 1>, 32>;

@compute
@workgroup_size(16, 16)
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
    let camera_pos: vec3<f32> = camera.position.xyz;
    let direction = vec3<f32>(uv_centered.x * aspect, -uv_centered.y, -focal_length);

    var ray = Ray(camera_pos, mat3x3<f32>(camera.rotation[0].xyz, camera.rotation[1].xyz, camera.rotation[2].xyz) * direction);
    let color = ray_color(ray);
    textureStore(output, vec2<i32>(id.xy), color);
}

fn ray_color(ray: Ray) -> vec4<f32> {
    //let hit_info = ray_sphere_intersect(vec3(0, 0, -1), 0.5, ray);
//    let hit_info = ray_aabb(vec3(-0.1, -0.1, -0.1), vec3(0.1, 0.1, 0.1), ray);
//    let hit_info2 = ray_aabb(vec3(0.1, -0.1, -0.1), vec3(0.3, 0.1, 0.1), ray);
//    let voxels = array<HitInfo, 2>(hit_info, hit_info2);
    var closest_hit: HitInfo = HitInfo(vec3<f32>(), vec3<f32>(), INFINITY, false);

    for (var x = 0u; x < 32; x++) {
        for (var y = 0u; y < 1; y++) {
            for (var z = 0u; z < 32; z++) {
                let chunk = chunks[x][y][z];
                let empty = chunk.empty != 0;
                if (empty) {
                    continue;
                }
                let info = ray_aabb(
                    vec3<f32>(-CHUNK_SIZE * VOXEL_SIZE / 2),
                    vec3<f32>(CHUNK_SIZE * VOXEL_SIZE / 2),
                    Ray(ray.origin - vec3<f32>(f32(x), f32(y), f32(z)) * CHUNK_SIZE * VOXEL_SIZE, ray.direction)
                 );
                if (info.t < closest_hit.t) {
                    closest_hit = info;
                }
            }
        }
    }

    if (closest_hit.t != INFINITY) {
        return 0.5 * vec4(closest_hit.hit_normal + 1, 0);
    }

    let a = 0.5 * (normalize(ray.direction).y + 1.0);
    return mix(vec4<f32>(1), vec4<f32>(0.5, 0.7, 1.0, 1.0), a);
}

fn ray_sphere_intersect(center: vec3<f32>, radius: f32, ray: Ray) -> HitInfo {
    let oc = center - ray.origin;
    let a = length(ray.direction) * length(ray.direction);
    let h = dot(ray.direction, oc);
    let c = length(oc) * length(oc) - radius * radius;
    let discriminant = h * h - a * c;
    var t = INFINITY;
    if (discriminant >= 0) {
        t = (h - sqrt(discriminant)) / a;
    }

    // hit info stuff
    let hit_point = ray.origin + ray.direction * t;
    let outward_normal = (hit_point - center) / radius;
    let front_face = dot(ray.direction, outward_normal) < 0;
    var normal = outward_normal;
    if (!front_face) {
        normal = -outward_normal;
    }

    return HitInfo(hit_point, normal, t, front_face);
}

fn ray_aabb(box_min: vec3<f32>, box_max: vec3<f32>, ray: Ray) -> HitInfo {
    let t_min = (box_min - ray.origin) / ray.direction;
    let t_max = (box_max - ray.origin) / ray.direction;
    let t1 = min(t_min, t_max);
    let t2 = max(t_min, t_max);
    let t_near = max(max(t1.x, t1.y), t1.z);
    let t_far = min(min(t2.x, t2.y), t2.z);
    var t = INFINITY;
    if (t_near <= t_far && t_far > 0.0) {
        if (t_near > 0.0) {
            t = t_near;
        } else {
            t = t_far;
        }
    }

    // hit info stuff
    let hit_point = ray.origin + ray.direction * t;
    let center = (box_min + box_max) * 0.5;
    let half_size = (box_max - box_min) * 0.5;
    let local = (hit_point - center) / half_size;
    let abs_local = abs(local);
    var outward_normal = vec3<f32>(sign(local.x), 0.0, 0.0);
    if (abs_local.y > abs_local.x && abs_local.y > abs_local.z) {
        outward_normal = vec3<f32>(0.0, sign(local.y), 0.0);
    } else if (abs_local.z > abs_local.x && abs_local.z > abs_local.y) {
        outward_normal = vec3<f32>(0.0, 0.0, sign(local.z));
    }
    let front_face = dot(ray.direction, outward_normal) < 0;
    var normal = outward_normal;
    if (!front_face) {
        normal = -outward_normal;
    }

    return HitInfo(hit_point, normal, t, front_face);
}
