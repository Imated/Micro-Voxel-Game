var<private> a: u32;
var<private> b: u32;
var<private> c: u32;
var<private> d: u32;

// https://indico.cern.ch/event/93877/contributions/2118070/attachments/1104200/1575343/acat3_revised_final.pdf
fn rand() -> f32 {
    let e = a - rotate(b, 27);
    a = b ^ rotate(c, 17);
    b = c + d;
    c = d + e;
    d = e + a;
    return f32(d) / 0xFFFFFFFF;
}

fn rand_init(seed: u32) {
    a = 0xF1EA5EED;
    b = seed;
    c = seed;
    d = seed;

    for (var i = 0; i < 4; i++) {
        rand();
    }
}

fn rotate(x: u32, k: u32) -> u32 {
    return ((x << k) | (x >> (32 - k)));
}

fn rand_unit_vector() -> vec3<f32> {
    return normalize(vec3<f32>(rand() * 2 - 1, rand() * 2 - 1, rand() * 2 - 1));
}

fn rand_on_hemisphere(normal: vec3<f32>) -> vec3<f32> {
    let u1 = rand();
    let u2 = rand();

    let r = sqrt(u1);
    let theta = 2.0 * 3.14159265 * u2;

    let x = r * cos(theta);
    let y = r * sin(theta);
    let z = sqrt(1.0 - u1);

    if abs(normal.x) > 0.5 {
        return vec3<f32>(sign(normal.x) * z, x, y);
    } else if abs(normal.y) > 0.5 {
        return vec3<f32>(x, sign(normal.y) * z, y);
    } else {
        return vec3<f32>(x, y, sign(normal.z) * z);
    }
}

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
    bricks: array<array<array<Brick, 8>, 8>, 8>,
}

struct Brick {
    empty: u32,
}

const INFINITY: f32 = 3.402823466e+38;
const WORLD_SIZE: vec3<i32> = vec3<i32>(8, 1, 8);
const CHUNK_SIZE: f32 = 8.0;
const VOXEL_SIZE: f32 = 0.1;
const EPSILON: f32 = 0.000001;

@group(0) @binding(0)
var output: texture_storage_2d<rgba16float, write>;

@group(1) @binding(0)
var<uniform> camera: Camera;

@group(2) @binding(0)
var<storage, read> chunks: array<array<array<Chunk, 8>, 1>, 8>;

fn is_brick_empty(brick_coords: vec3<i32>) -> bool {
    let offsetted_coords = brick_coords + WORLD_SIZE * vec3<i32>(i32(CHUNK_SIZE)) / 2;
    let chunk_coord = offsetted_coords / vec3<i32>(i32(CHUNK_SIZE));
    let local_coord = offsetted_coords % vec3<i32>(i32(CHUNK_SIZE));

    return chunks[chunk_coord.x][chunk_coord.y][chunk_coord.z]
        .bricks[local_coord.x][local_coord.y][local_coord.z]
        .empty != 0;
}

// fn get_voxel(brick_coords: vec3<i32>, voxel_local_coords: vec3<i32>) -> u32 {
//     let offsetted_coords = brick_coords + vec3<i32>(32, 1, 32) * vec3<i32>(i32(CHUNK_SIZE)) / 2;
//     let chunk_coord = offsetted_coords / vec3<i32>(i32(CHUNK_SIZE));
//     let local_coord = offsetted_coords % vec3<i32>(i32(CHUNK_SIZE));

//     return chunks[chunk_coord.x][chunk_coord.y][chunk_coord.z]
//         .bricks[local_coord.x][local_coord.y][local_coord.z]
//         .voxels[voxel_local_coords.x][voxel_local_coords.y][voxel_local_coords.z];
// }

fn is_brick_in_bounds(coords: vec3<i32>) -> bool {
    let world_half = WORLD_SIZE * vec3<i32>(i32(CHUNK_SIZE)) / 2;
    return all(coords < world_half) && all(coords >= -world_half);
}

@compute
@workgroup_size(16, 16)
fn main(@builtin(global_invocation_id)id: vec3<u32>) {
    let size = textureDimensions(output);

    if id.x >= size.x || id.y >= size.y {
        return;
    }

    let rng_state = id.x + id.y * 1099087573;
    rand_init(rng_state);

    //textureStore(output, vec2<i32>(id.xy), vec4<f32>(rand()));
    //return;

    let uv = vec2<f32>(id.xy) / vec2<f32>(size);
    let uv_centered = uv * 2.0 - 1.0;

    let aspect = f32(size.x) / f32(size.y);

    const fov = 67.0;
    const focal_length = 1 / tan(radians(fov) / 2);
    let camera_pos: vec3<f32> = camera.position.xyz;
    let direction = vec3<f32>(uv_centered.x * aspect, -uv_centered.y, -focal_length);

    var ray = Ray(camera_pos, normalize(mat3x3<f32>(camera.rotation[0].xyz, camera.rotation[1].xyz, camera.rotation[2].xyz) * direction));
    var current_ray = ray;
    var in_light = vec4<f32>(0.0);
    var color = vec4<f32>(1.0);
    for (var i = 0; i < 4; i++) {
        let info = trace_ray(current_ray);

        if info.t == INFINITY {
            in_light += sky(current_ray) * color;
            break;
        }

        color *= 0.5 * vec4<f32>(info.hit_normal + 1, 0);
        current_ray.direction = rand_on_hemisphere(info.hit_normal);
        current_ray.origin = info.hit_point + info.hit_normal * 1e-3;
    }
    //let color = ray_color(trace_ray(ray), ray);
    textureStore(output, vec2<i32>(id.xy), in_light);
}

fn trace_ray(ray: Ray) -> HitInfo {
    let world_half = vec3<f32>(32, 1, 32) * vec3<f32>(CHUNK_SIZE) * VOXEL_SIZE / 2;
    let world_hit = ray_aabb(-world_half, world_half, ray);
    if world_hit.t == INFINITY {
        return HitInfo(vec3<f32>(0.0), vec3<f32>(0.0), INFINITY, false);
    }

    // either the first point where the ray hits if we are outside of the world or the brick we are in if we are inside
    let inside_world = all(ray.origin >= -world_half) && all(ray.origin <= world_half);
    let t = select(world_hit.t + 0.0001, 0.0, inside_world);
    let start_pos = ray.origin + ray.direction * t;

    let grid_pos = start_pos / VOXEL_SIZE;
    var current_brick = vec3<i32>(floor(grid_pos));

    let step = vec3<i32>(sign(ray.direction));
    let inv_dir = select(vec3<f32>(INFINITY), 1 / ray.direction, step != vec3<i32>(0));
    let t_delta = abs(inv_dir) * VOXEL_SIZE;
    let cb = select(vec3<f32>(current_brick), vec3<f32>(current_brick) + 1, ray.direction > vec3<f32>(0));
    var t_max = t + (cb - grid_pos) * inv_dir * VOXEL_SIZE;

    var normal = world_hit.hit_normal;
    var t_current = t;

    loop {
        // ray stepped out of loaded chunks, no point to march ray anymore
        if !is_brick_in_bounds(current_brick) {
            break;
        }

        if !is_brick_empty(current_brick) {
            return HitInfo(ray.origin + ray.direction * t_current, normal, t_current, true);
        }

        if t_max.x < t_max.y && t_max.x < t_max.z {
            t_current = t_max.x;

            current_brick.x += step.x;
            t_max.x += t_delta.x;

            normal = vec3<f32>(0);
            normal.x = -f32(step.x);
        } else if t_max.y < t_max.z {
            t_current = t_max.y;

            current_brick.y += step.y;
            t_max.y += t_delta.y;

            normal = vec3<f32>(0);
            normal.y = -f32(step.y);
        } else {
            t_current = t_max.z;

            current_brick.z += step.z;
            t_max.z += t_delta.z;

            normal = vec3<f32>(0);
            normal.z = -f32(step.z);
        }
    }

    return HitInfo(vec3<f32>(0.0), vec3<f32>(0.0), INFINITY, false);
}

fn sky(ray: Ray) -> vec4<f32> {
    let t = 0.5 * (normalize(ray.direction).y + 1.0);
    return mix(vec4<f32>(1), vec4<f32>(0.5, 0.7, 1.0, 1.0), t);
}

fn ray_aabb(box_min: vec3<f32>, box_max: vec3<f32>, ray: Ray) -> HitInfo {
    let t_min = (box_min - ray.origin) / ray.direction;
    let t_max = (box_max - ray.origin) / ray.direction;
    let t1 = min(t_min, t_max);
    let t2 = max(t_min, t_max);
    let t_near = max(max(t1.x, t1.y), t1.z);
    let t_far = min(min(t2.x, t2.y), t2.z);
    var t = INFINITY;
    if t_near <= t_far && t_far > 0.0 {
        if t_near > 0.0 {
            t = t_near;
        } else {
            t = t_far;
        }
    }

    // hit info stuff
    if t == INFINITY {
        return HitInfo(vec3<f32>(0.0), vec3<f32>(0.0), t, false);
    }

    let hit_point = ray.origin + ray.direction * t;
    let center = (box_min + box_max) * 0.5;
    let half_size = (box_max - box_min) * 0.5;
    let local = (hit_point - center) / half_size;
    let abs_local = abs(local);
    var outward_normal = vec3<f32>(sign(local.x), 0.0, 0.0);
    if abs_local.y > abs_local.x && abs_local.y > abs_local.z {
        outward_normal = vec3<f32>(0.0, sign(local.y), 0.0);
    } else if abs_local.z > abs_local.x && abs_local.z > abs_local.y {
        outward_normal = vec3<f32>(0.0, 0.0, sign(local.z));
    }
    let front_face = dot(ray.direction, outward_normal) < 0;
    var normal = outward_normal;
    if !front_face {
        normal = -outward_normal;
    }

    return HitInfo(hit_point, normal, t, front_face);
}
