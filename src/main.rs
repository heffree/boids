use macroquad::{prelude::*, window};

const BOID_HEIGHT: f32 = 13.;
const BOID_BASE: f32 = 8.;
const BOID_COUNT: u32 = 1000;
const MAX_SPEED: f32 = 2.;

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
            pos: Vec2::new(screen_width() / 2., screen_height() / 2.),
            rot: 0.,
            vel: Vec2::new(
                rand::gen_range(-MAX_SPEED, MAX_SPEED),
                rand::gen_range(-MAX_SPEED, MAX_SPEED),
            ),
            color: WHITE,
        })
        .collect();

    let mut start_time = get_time();
    loop {
        let time_now = get_time();

        if time_now - start_time > 2. {
            for boid in boids.iter_mut() {
                boid.vel = Vec2::new(
                    rand::gen_range(-MAX_SPEED, MAX_SPEED),
                    rand::gen_range(-MAX_SPEED, MAX_SPEED),
                );
                boid.color = calc_color(&boid.vel);
            }
            start_time = time_now;
        }

        for boid in boids.iter_mut() {
            boid.pos += boid.vel;
            boid.rot = boid.vel.x.atan2(-boid.vel.y);
        }

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

fn calc_color(v: &Vec2) -> Color {
    Color::new(v.x + MAX_SPEED, v.y + MAX_SPEED, v.x + v.y + MAX_SPEED, 1.0)
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
