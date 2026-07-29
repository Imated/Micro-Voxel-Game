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
const CHUNK_SIZE: f32 = 8.0;
const VOXEL_SIZE: f32 = 0.1;

@group(0) @binding(0)
var output: texture_storage_2d<rgba16float, write>;

@group(1) @binding(0)
var<uniform> camera: Camera;

@group(2) @binding(0)
var<storage, read> chunks: array<array<array<Chunk, 32>, 1>, 32>;

fn get_global_brick(coords: vec3<i32>) -> Brick {
    let offsetted_coords = coords + vec3<i32>(32, 1, 32) * vec3<i32>(i32(CHUNK_SIZE)) / 2;
    let chunk_coord = offsetted_coords / vec3<i32>(i32(CHUNK_SIZE));
    let local_coord = offsetted_coords % vec3<i32>(i32(CHUNK_SIZE));
    let chunk = chunks[chunk_coord.x][chunk_coord.y][chunk_coord.z];

    return chunk.bricks[local_coord.x][local_coord.y][local_coord.z];
}

fn is_brick_in_bounds(coords: vec3<i32>) -> bool {
    let world_half = vec3<i32>(32, 1, 32) * vec3<i32>(i32(CHUNK_SIZE)) / 2;
    return all(coords <= world_half) && all(coords >= -world_half);
}

@compute
@workgroup_size(16, 16)
fn main(@builtin(global_invocation_id)id: vec3<u32>) {
    let size = textureDimensions(output);

    if id.x >= size.x || id.y >= size.y {
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
    let world_half = vec3<f32>(32, 1, 32) * vec3<f32>(CHUNK_SIZE) * VOXEL_SIZE / 2;
    let world_hit = ray_aabb(-world_half, world_half, ray);
    if world_hit.t == INFINITY {
        return sky(ray);
    }

    // either the first point where the ray hits if we are outside of the world or the brick we are in if we are inside
    let inside_world = all(ray.origin >= -world_half) && all(ray.origin <= world_half);
    let t = select(world_hit.t + 0.0001, 0.0, inside_world);
    let start_pos = ray.origin + ray.direction * t;

    let grid_pos = start_pos / VOXEL_SIZE;
    var current_brick = vec3<i32>(floor(grid_pos));

    let step = vec3<i32>(sign(ray.direction));
    let inv_dir = select(vec3<f32>(INFINITY), 1 / ray.direction, step != vec3<i32>(0));
    let t_delta = abs(inv_dir);
    let cb = select(vec3<f32>(current_brick), vec3<f32>(current_brick) + 1, ray.direction > vec3<f32>(0));
    var t_max = (cb - grid_pos) * inv_dir;

    var normal = world_hit.hit_normal;

    loop {
        // ray stepped out of loaded chunks, no point to march ray anymore
        if (!is_brick_in_bounds(current_brick)) {
            break;
        }

        let brick = get_global_brick(current_brick);
        if brick.empty == 0 {
            return 0.5 * vec4(normal + 1, 0);
        }

        if (t_max.x < t_max.y && t_max.x < t_max.z) {
            current_brick.x += step.x;
            t_max.x += t_delta.x;

            normal = vec3<f32>(0);
            normal.x = -f32(step.x);
        } else if (t_max.y < t_max.z) {
            current_brick.y += step.y;
            t_max.y += t_delta.y;

            normal = vec3<f32>(0);
            normal.y = -f32(step.y);
        } else {
            current_brick.z += step.z;
            t_max.z += t_delta.z;

            normal = vec3<f32>(0);
            normal.z = -f32(step.z);
        }
    }

    return sky(ray);
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
