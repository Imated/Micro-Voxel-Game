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
    loop {
        let v = vec3<f32>(rand() * 2 - 1, rand() * 2 - 1, rand() * 2 - 1);
        let len_sq = dot(v, v);
        if len_sq > EPSILON && len_sq <= 1.0 {
            return v / sqrt(len_sq);
        }
    }
    return vec3<f32>(1);
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
    inv_dir: vec3<f32>,
    step: vec3<i32>,
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
    brick_indices: array<u32, 512>, // CHUNK_SIZE^3
    brick_mask: array<u32, 16>, // CHUNK_SIZE^3 / 32
}

struct Brick {
    //voxels: array<u32, 512>, allocate in separate buffer latore
    occupancy_mask: array<u32, 16>, // BRICK_SIZE^3 / 32
}

const INFINITY: f32 = 3.402823466e+38;
const WORLD_SIZE: vec3<i32> = vec3<i32>(8, 1, 8);
const CHUNK_SIZE: i32 = 8;
const BRICK_SIZE: i32 = 8;
const EPSILON: f32 = 0.000001;
const PI: f32 = 3.14159265;

const SUN_DIR: vec3<f32> = vec3<f32>(0.3, 1.0, 0.5);
const SUN_ANGULAR_RADIUS: f32 = 0.0093;

@group(0) @binding(0)
var<storage, read> chunks: array<Chunk, 64>; // 8 * 1 * 8

@group(0) @binding(1)
var<storage, read> brick_pool: array<Brick>;

@group(1) @binding(0)
var<uniform> camera: Camera;

@group(2) @binding(0)
var output: texture_storage_2d<rgba16float, write>;

fn flatten(coord: vec3<u32>, size: vec3<u32>) -> u32 {
    return coord.z * size.y * size.x + coord.y * size.x + coord.x;
}

fn create_ray(origin: vec3<f32>, direction: vec3<f32>) -> Ray {
    let step = vec3<i32>(sign(direction));
    let inv_dir = select(vec3<f32>(INFINITY), 1 / direction, step != vec3<i32>(0));
    return Ray(origin, direction, inv_dir, step);
}

fn D(alpha: f32, N: vec3<f32>, H: vec3<f32>) -> f32 {
    let numerator = pow(alpha, 2.0);
    let N_dot_H = max(dot(N, H), 0.0);
    let denominator = max(PI * pow(pow(N_dot_H, 2.0) * (numerator - 1.0) + 1.0, 2.0), EPSILON);
    return numerator / denominator;
}

fn G1(alpha: f32, N: vec3<f32>, X: vec3<f32>) -> f32 {
    let numerator = max(dot(N, X), 0.0);

    let k = alpha / 2.0;
    let denominator = max(numerator * (1.0 - k) + k, EPSILON);
    return numerator / denominator;
}

fn G(alpha: f32, N: vec3<f32>, V: vec3<f32>, L: vec3<f32>) -> f32 {
    return G1(alpha, N, V) * G1(alpha, N, L);
}

fn F(F0: vec3<f32>, V: vec3<f32>, H: vec3<f32>) -> vec3<f32> {
    return F0 + (vec3<f32>(1.0) - F0) * pow(1.0 - max(dot(V, H), 0.0), 5.0);
}

fn shadow_color(p: vec3<f32>, n: vec3<f32>) -> f32 {
    let origin = p + n * 0.01;
    var lit = 0.0;
    let d = normalize(SUN_DIR);
    let hit = trace_ray(create_ray(origin, d));
    return select(0.0, 1.0, hit.t == INFINITY);
}

fn voxel_color(info: HitInfo) -> vec4<f32> {
    let albedo = vec3<f32>(0.12, 0.32, 0.06);
    let roughness = 0.85;
    let F0 = vec3<f32>(0.04);

    let light_color = vec3<f32>(1.0, 0.95, 0.85) * 3.5;
    let ambient_color = vec3<f32>(0.5, 0.7, 1.0) * 0.25;

    let N = normalize(info.hit_normal);
    let V = normalize(camera.position.xyz - info.hit_point);
    let L = normalize(SUN_DIR);
    let H = normalize(V + L);

    let alpha = roughness * roughness;

    let Ks = F(F0, V, H);
    let Kd = vec3<f32>(1.0) - Ks;
    let lambert = albedo / PI;

    let cook_torrance_numerator = D(alpha, N, H) * G(alpha, N, V, L) * Ks;
    let cook_torrance_denominator = max(4.0 * max(dot(V, N), 0.0) * max(dot(L, N), 0.0), EPSILON);
    let cook_torrance = cook_torrance_numerator / cook_torrance_denominator;

    // shadows
    let shadow_color = shadow_color(info.hit_point, N);

    let BRDF = Kd * lambert + cook_torrance;
    let outgoing_light = BRDF * light_color * max(dot(L, N) * shadow_color, 0.0);
    let ambient = albedo * ambient_color;

    return vec4<f32>(outgoing_light + ambient, 1.0);
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

    var ray = create_ray(camera_pos, normalize(mat3x3<f32>(camera.rotation[0].xyz, camera.rotation[1].xyz, camera.rotation[2].xyz) * direction));
    var info = trace_ray(ray);
    var color = vec4<f32>(1.0);
    color = select(sky(ray), voxel_color(info), info.t != INFINITY);
    textureStore(output, vec2<i32>(id.xy), color);
}

struct Dda {
    cell: vec3<i32>,
    t_max: vec3<f32>,
    t: f32,
    normal: vec3<f32>,
}

fn dda_begin(ray: Ray, t: f32, cell_size: f32, lo: vec3<i32>, hi: vec3<i32>, normal: vec3<f32>) -> Dda {
    let grid_pos = (ray.origin + ray.direction * t) / cell_size;
    let cell = clamp(vec3<i32>(floor(grid_pos)), lo, hi - vec3<i32>(1));
    let cb = select(vec3<f32>(cell), vec3<f32>(cell) + 1, ray.direction > vec3<f32>(0));
    let t_max = select(vec3<f32>(INFINITY), t + (cb - grid_pos) * ray.inv_dir * cell_size, ray.step != vec3<i32>(0));
    return Dda(cell, t_max, t, normal);
}

fn dda_step(dda: Dda, ray: Ray, t_delta: vec3<f32>) -> Dda {
    let crossed = dda.t_max <= min(dda.t_max.yzx, dda.t_max.zxy);
    return Dda(
        dda.cell + vec3<i32>(crossed) * ray.step,
        select(dda.t_max, dda.t_max + t_delta, crossed),
        min(dda.t_max.x, min(dda.t_max.y, dda.t_max.z)),
        -vec3<f32>(crossed) * vec3<f32>(ray.step),
    );
}

fn ray_aabb(box_min: vec3<f32>, box_max: vec3<f32>, ray: Ray) -> HitInfo {
    let safe_dir = select(vec3<f32>(EPSILON), ray.direction, abs(ray.direction) > vec3<f32>(EPSILON));
    let inv_dir = 1.0 / safe_dir;
    let t_min = (box_min - ray.origin) * inv_dir;
    let t_max = (box_max - ray.origin) * inv_dir;
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

fn chunk_is_empty(chunk_index: u32) -> bool {
    var words = 0u;
    for (var i = 0u; i < 16u; i++) {
        words |= chunks[chunk_index].brick_mask[i];
    }
    return words == 0;
}

fn trace_ray(ray: Ray) -> HitInfo {
    let miss = HitInfo(vec3<f32>(0.0), vec3<f32>(0.0), INFINITY, false);
    let world_half = vec3<f32>(WORLD_SIZE * CHUNK_SIZE) * f32(BRICK_SIZE / 2);
    let world_hit = ray_aabb(-world_half, world_half, ray);
    if world_hit.t == INFINITY {
        return miss;
    }

    // either the first point where the ray hits if we are outside of the world or the chunk we are in if we are inside
    let inside_world = all(ray.origin >= -world_half) && all(ray.origin <= world_half);
    let t = select(world_hit.t + 0.0001, 0.0, inside_world);

    let chunk_t_delta = abs(ray.inv_dir) * f32(CHUNK_SIZE * BRICK_SIZE);
    let brick_t_delta = abs(ray.inv_dir) * f32(BRICK_SIZE);
    let voxel_t_delta = abs(ray.inv_dir);

    let grid_ray = Ray(ray.origin + world_half, ray.direction, ray.inv_dir, ray.step);
    var chunk = dda_begin(grid_ray, t, f32(CHUNK_SIZE * BRICK_SIZE), vec3<i32>(0), WORLD_SIZE, world_hit.hit_normal);

    loop {
        let chunk_index = flatten(vec3<u32>(chunk.cell), vec3<u32>(WORLD_SIZE));
        if !chunk_is_empty(chunk_index) {
            let brick_lo = chunk.cell * CHUNK_SIZE;
            let brick_hi = brick_lo + vec3<i32>(CHUNK_SIZE);
            var brick = dda_begin(grid_ray, chunk.t, f32(BRICK_SIZE), brick_lo, brick_hi, chunk.normal);
            loop {
                let brick_slot = flatten(vec3<u32>(brick.cell & vec3<i32>(CHUNK_SIZE - 1)), vec3<u32>(u32(CHUNK_SIZE)));
                if ((chunks[chunk_index].brick_mask[brick_slot >> 5] >> (brick_slot & 31)) & 1) != 0 {
                    let brick_index = chunks[chunk_index].brick_indices[brick_slot];
                    let voxel_lo = brick.cell * BRICK_SIZE;
                    let voxel_hi = voxel_lo + vec3<i32>(BRICK_SIZE);
                    var voxel = dda_begin(grid_ray, brick.t, 1.0, voxel_lo, voxel_hi, brick.normal);
                    loop {
                        let bit = flatten(vec3<u32>(voxel.cell & vec3<i32>(BRICK_SIZE - 1)), vec3<u32>(u32(BRICK_SIZE)));
                        if ((brick_pool[brick_index].occupancy_mask[bit >> 5] >> (bit & 31)) & 1) != 0 {
                            return HitInfo(ray.origin + ray.direction * voxel.t, voxel.normal, voxel.t, true);
                        }

                        voxel = dda_step(voxel, grid_ray, voxel_t_delta);
                        if any(voxel.cell < voxel_lo) || any(voxel.cell >= voxel_hi) {
                            break;
                        }
                    }
                }

                brick = dda_step(brick, grid_ray, brick_t_delta);
                if any(brick.cell < brick_lo) || any(brick.cell >= brick_hi) {
                    break;
                }
            }
        }

        chunk = dda_step(chunk, grid_ray, chunk_t_delta);
        if any(chunk.cell < vec3<i32>(0)) || any(chunk.cell >= WORLD_SIZE) {
            break;
        }
    }

    return miss;
}

fn sky(ray: Ray) -> vec4<f32> {
    let t = 0.5 * (normalize(ray.direction).y + 1.0);
    return mix(vec4<f32>(1), vec4<f32>(0.5, 0.7, 1.0, 1.0), t);
}
