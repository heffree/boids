use macroquad::{prelude::*, window};
use rand::rand;

const BOID_HEIGHT: f32 = 13.;
const BOID_BASE: f32 = 8.;
const BOID_COUNT: u32 = 1000;
const MAX_SPEED: f32 = 1.;
const DISTANCE: f32 = 2000.;

#[derive(Clone)]
struct Boid {
    pos: Vec2,
    rot: f32,
    vel: Vec2,
    color: Color,
}

#[macroquad::main("Boids")]
async fn main() {
    window::set_fullscreen(true);
    let mut boids: Vec<Boid> = (0..BOID_COUNT)
        .map(|_| Boid {
            pos: Vec2::new(
                rand::gen_range(-DISTANCE, DISTANCE),
                rand::gen_range(-DISTANCE, DISTANCE),
            ),
            rot: 0.,
            vel: Vec2::new(
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
            let v1 = Vec2::new(
                boid.pos.x + boid.rot.sin() * BOID_HEIGHT / 2.,
                boid.pos.y - boid.rot.cos() * BOID_HEIGHT / 2.,
            );
            let v2 = Vec2::new(
                boid.pos.x - boid.rot.cos() * BOID_BASE / 2. - boid.rot.sin() * BOID_HEIGHT / 2.,
                boid.pos.y - boid.rot.sin() * BOID_BASE / 2. + boid.rot.cos() * BOID_HEIGHT / 2.,
            );
            let v3 = Vec2::new(
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
        boid.pos += boid.vel.clamp(
            Vec2::new(-MAX_SPEED, -MAX_SPEED),
            Vec2::new(-MAX_SPEED, MAX_SPEED),
        );
        boid.pos = wrap_around(&boid.pos);
        boid.rot = boid.vel.x.atan2(-boid.vel.y);
    }
}

/// moves the boid towards the center of all boids
///
/// 1. Find the center of all boids
/// 2. Determine perceived center
/// 3. Get our boid a percentage of the way there
///
fn cohesion_rule(boids: &mut Vec<Boid>) -> () {
    let center_of_mass = boids
        .iter()
        .fold(Vec2::new(0., 0.), |acc, boid| acc + boid.pos);

    for boid in boids.iter_mut() {
        let perceived_center = (center_of_mass - boid.pos) / (BOID_COUNT - 1) as f32;
        boid.vel += (perceived_center - boid.pos) / 100.;
    }
}

/// aligns the boids velocity with the boids around it
///
/// 1. Find the average velocity
/// 2. Determine velocity of others
/// 3. Align our boid a percentage of the way
fn alignment_rule(boids: &mut Vec<Boid>) -> () {
    let average_velocity = boids
        .iter()
        .fold(Vec2::new(0., 0.), |acc, boid| acc + boid.vel);

    for boid in boids.iter_mut() {
        let other_velocity = (average_velocity - boid.vel) / (BOID_COUNT - 1) as f32;
        boid.vel += (other_velocity - boid.vel) / 8.;
    }
}

/// keep our boid away from other boids
///
/// TODO fill this out
fn separation_rule(boids: &mut Vec<Boid>) {
    let count = boids.len();
    // Temporary vector to store each boid's separation adjustment.
    let mut adjustments = vec![Vec2::new(0.0, 0.0); count];

    for i in 0..count {
        for j in 0..count {
            if i != j {
                let diff = boids[j].pos - boids[i].pos;
                if diff.length() < 20.0 {
                    adjustments[i] -= diff;
                }
            }
        }
    }

    // Update each boid's velocity with its computed separation force.
    for (boid, adjustment) in boids.iter_mut().zip(adjustments.iter()) {
        boid.vel += *adjustment * 2.;
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
    let mut vr = Vec2::new(v.x, v.y);
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
