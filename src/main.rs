use macroquad::{prelude::*, window};

const BOID_HEIGHT: f32 = 13.;
const BOID_BASE: f32 = 8.;
const BOID_COUNT: u32 = 500;
const MAX_SPEED: f32 = 5.;
const DISTANCE: f32 = 500.;

const SEPARATION_THRESHOLD: f32 = 10.;

const COHESION_FACTOR: f32 = 8.;
const COHESION_THRESHOLD: f32 = 25.;

const ALIGNMENT_FACTOR: f32 = 400.;
const ALIGNMENT_THRESHOLD: f32 = 25.;

#[derive(Clone)]
struct Boid {
    pos: Vec2,
    rot: f32,
    vel: Vec2,
    color: Color,
}

fn toroidal_diff(a: Vec2, b: Vec2) -> Vec2 {
    let w = screen_width();
    let h = screen_height();
    let mut dx = a.x - b.x;
    let mut dy = a.y - b.y;
    if dx > w / 2.0 {
        dx -= w;
    } else if dx < -w / 2.0 {
        dx += w;
    }
    if dy > h / 2.0 {
        dy -= h;
    } else if dy < -h / 2.0 {
        dy += h;
    }
    vec2(dx, dy)
}

#[macroquad::main("Boids")]
async fn main() {
    window::set_fullscreen(true);
    let mut boids: Vec<Boid> = (0..BOID_COUNT)
        .map(|_| Boid {
            pos: vec2(
                rand::gen_range(-DISTANCE, DISTANCE * 4.),
                rand::gen_range(-DISTANCE, DISTANCE * 4.),
            ),
            rot: 0.,
            vel: vec2(
                rand::gen_range(-MAX_SPEED, MAX_SPEED),
                rand::gen_range(-MAX_SPEED, MAX_SPEED),
            ),
            color: WHITE,
        })
        .collect();

    loop {
        move_boids(&mut boids);

        for boid in boids.iter() {
            // Stole this from a macroquad example.
            let v1 = vec2(
                boid.pos.x + boid.rot.sin() * BOID_HEIGHT / 2.,
                boid.pos.y - boid.rot.cos() * BOID_HEIGHT / 2.,
            );
            let v2 = vec2(
                boid.pos.x - boid.rot.cos() * BOID_BASE / 2. - boid.rot.sin() * BOID_HEIGHT / 2.,
                boid.pos.y - boid.rot.sin() * BOID_BASE / 2. + boid.rot.cos() * BOID_HEIGHT / 2.,
            );
            let v3 = vec2(
                boid.pos.x + boid.rot.cos() * BOID_BASE / 2. - boid.rot.sin() * BOID_HEIGHT / 2.,
                boid.pos.y + boid.rot.sin() * BOID_BASE / 2. + boid.rot.cos() * BOID_HEIGHT / 2.,
            );
            draw_triangle(v1, v2, v3, boid.color);
        }

        next_frame().await;
    }
}

fn move_boids(boids: &mut Vec<Boid>) {
    cohesion_rule(boids);
    alignment_rule(boids);
    separation_rule(boids);
    for boid in boids.iter_mut() {
        let target_rotation = boid.vel.x.atan2(-boid.vel.y);
        boid.rot = target_rotation;

        boid.pos += boid
            .vel
            .clamp(vec2(-MAX_SPEED, -MAX_SPEED), vec2(MAX_SPEED, MAX_SPEED));

        boid.pos = wrap_around(&boid.pos);
    }
}

/// moves the boid towards the center of all boids
///
/// 1. Find the center of nearby boids
/// 2. Determine perceived center
/// 3. Get our boid a percentage of the way there
///
fn cohesion_rule(boids: &mut Vec<Boid>) {
    let num_boids = boids.len();
    let mut adjustments = vec![vec2(0.0, 0.0); num_boids];

    for i in 0..num_boids {
        let mut center = vec2(0., 0.);
        let mut count = 0;
        for j in 0..num_boids {
            let diff = boids[j].pos - boids[i].pos;
            if i != j && diff.length() < COHESION_THRESHOLD {
                center += boids[j].pos;
                count += 1;
            }
        }
        if count > 0 {
            let perceived_center = center / count as f32;
            adjustments[i] += (perceived_center - boids[i].pos) / COHESION_FACTOR;
        }
    }
    // Update each boid's velocity with its computed cohesion force.
    for (boid, adjustment) in boids.iter_mut().zip(adjustments.iter()) {
        boid.vel += *adjustment;
    }
}

/// aligns the boids velocity with the boids around it
///
/// 1. Find the average velocity
/// 2. Determine velocity of others
/// 3. Align our boid a percentage of the way
fn alignment_rule(boids: &mut Vec<Boid>) {
    let num_boids = boids.len();
    let mut adjustments = vec![vec2(0.0, 0.0); num_boids];

    for i in 0..boids.len() {
        let mut avg_velocity = vec2(0., 0.);
        let mut count = 0;
        for j in 0..boids.len() {
            let diff = boids[j].pos - boids[i].pos;
            if i != j && diff.length() < ALIGNMENT_THRESHOLD {
                avg_velocity += boids[j].vel;
                count += 1;
            }
        }
        if count > 0 {
            let perceived_velocity = avg_velocity / count as f32;
            adjustments[i] += (perceived_velocity - boids[i].vel) / ALIGNMENT_FACTOR;
        }
    }
    for (boid, adjustment) in boids.iter_mut().zip(adjustments.iter()) {
        boid.vel += *adjustment;
    }
}

/// keep our boid away from other boids
///
/// 1. create adjustments vec2 for each boid
/// 2. get the diff in distance for boids many to many
/// 3. if diff is less than constant, adjust boid directly away?
fn separation_rule(boids: &mut Vec<Boid>) {
    let num_boids = boids.len();
    let mut adjustments = vec![vec2(0.0, 0.0); num_boids];

    for i in 0..num_boids {
        for j in 0..num_boids {
            if i != j {
                let diff = boids[j].pos - boids[i].pos;
                if diff.abs().length() < SEPARATION_THRESHOLD {
                    adjustments[i] -= diff;
                }
            }
        }
    }

    // Update each boid's velocity with its computed separation force.
    for (boid, adjustment) in boids.iter_mut().zip(adjustments.iter()) {
        boid.vel += *adjustment;
    }
}

fn calc_color(boid: &Boid) -> Color {
    Color::new(
        boid.vel.x + MAX_SPEED,
        boid.vel.y + MAX_SPEED,
        boid.vel.x + boid.vel.y + MAX_SPEED,
        1.0,
    )
}

fn wrap_around(v: &Vec2) -> Vec2 {
    let mut vr = vec2(v.x, v.y);
    if vr.x > screen_width() {
        vr.x = 0.;
    }
    if vr.x < 0. {
        vr.x = screen_width()
    }
    if vr.y > screen_height() {
        vr.y = 0.;
    }
    if vr.y < 0. {
        vr.y = screen_height()
    }
    vr
}
