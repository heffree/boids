use macroquad::prelude::*;

const BOID_HEIGHT: f32 = 13.;
const BOID_BASE: f32 = 8.;
const BOID_COUNT: u32 = 700;
const MAX_SPEED: f32 = 6.;

const SEPARATION_FACTOR: f32 = 20.;
const SEPARATION_DISTANCE_THRESHOLD: f32 = 10.;

const COHESION_FACTOR: f32 = 600.;
const COHESION_DISTANCE_THRESHOLD: f32 = 150.;

const ALIGNMENT_FACTOR: f32 = 75.;
const ALIGNMENT_DISTANCE_THRESHOLD: f32 = 100.;

#[derive(Clone, Debug)]
struct Boid {
    pos: Vec2,
    rot: f32,
    vel: Vec2,
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
    //window::set_fullscreen(true);

    // Wait until the window is fullscreen.
    //while macroquad::window::screen_height() < 700. {
    //    clear_background(BLACK);
    //    draw_text("Waiting for fullscreen...", 20.0, 20.0, 30.0, WHITE);
    //    next_frame().await;
    //}

    let width = screen_width();
    let height = screen_height();

    let row_size = 20.;
    let col_size = 10.;
    let grid_cols = (width / col_size) as u32;
    let grid_rows = (height / row_size) as u32;
    let positions: Vec<Vec2> = (0..grid_cols)
        .flat_map(|i| (0..grid_rows).map(move |j| vec2(i as f32 * row_size, j as f32 * col_size)))
        .collect();

    //println!("{:?}", positions);

    let mut boids: Vec<Boid> = (0..BOID_COUNT)
        .map(|index| Boid {
            pos: positions[index as usize],
            rot: 0.,
            vel: vec2(
                rand::gen_range(-MAX_SPEED, MAX_SPEED),
                rand::gen_range(-MAX_SPEED, MAX_SPEED),
            ),
        })
        .collect();

    loop {
        move_boids(&mut boids);

        for (i, boid) in boids.iter().enumerate() {
            //if i == 0 {
            //    println!("{:?} {:?}", boid, calc_color(&boid));
            //}
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
            let color = calc_color(&boid); // if i == 0 { RED } else { calc_color(&boid) };
            draw_triangle(v1, v2, v3, color);
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

        if boid.vel.length() > MAX_SPEED {
            boid.vel = boid.vel.normalize() * MAX_SPEED
        }
        boid.pos += boid.vel;

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
    let mut super_count = 0;

    for i in 0..num_boids {
        super_count += 1;
        let mut center = vec2(0., 0.);
        let mut count = 0;
        for j in 0..num_boids {
            if i != j {
                let diff = toroidal_diff(boids[i].pos, boids[j].pos);
                if diff.length() < COHESION_DISTANCE_THRESHOLD {
                    center += boids[j].pos;
                    count += 1;
                }
            }
        }
        if count > 0 {
            let perceived_center = center / count as f32;
            adjustments[i] += perceived_center - boids[i].pos;
            //if i == 0 {
            //    //println!("cohesion adjustment {:?}", adjustments[i]);
            //}
        }
    }
    // Update each boid's velocity with its computed cohesion force.
    for (boid, adjustment) in boids.iter_mut().zip(adjustments.iter()) {
        boid.vel += *adjustment / COHESION_FACTOR;
    }
    //println!("count {:?}", super_count);
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
            if i != j {
                let diff = toroidal_diff(boids[i].pos, boids[j].pos);
                if diff.length() < ALIGNMENT_DISTANCE_THRESHOLD {
                    avg_velocity += boids[j].vel;
                    count += 1;
                }
            }
        }
        if count > 0 {
            let perceived_velocity = avg_velocity / count as f32 - boids[i].vel / count as f32;
            adjustments[i] += perceived_velocity;

            //if i == 0 {
            //    println!("alignment adjustment {:?}", adjustments[i]);
            //}
        }
    }
    for (boid, adjustment) in boids.iter_mut().zip(adjustments.iter()) {
        boid.vel += *adjustment / ALIGNMENT_FACTOR;
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
    let screen_vec = vec2(screen_width(), screen_height());

    for i in 0..num_boids {
        for j in 0..num_boids {
            if i != j {
                let real_diff = boids[j].pos - boids[i].pos;
                let wrapped_diff = boids[j].pos - boids[i].pos - screen_vec;
                let diff = if real_diff.length() < wrapped_diff.length() {
                    real_diff
                } else {
                    wrapped_diff
                };
                if diff.length() < SEPARATION_DISTANCE_THRESHOLD {
                    adjustments[i] -= diff;
                }
            }
        }
        //if i == 0 {
        //    println!("separation adjustment {:?}", adjustments[i]);
        //}
    }

    // Update each boid's velocity with its computed separation force.
    for (boid, adjustment) in boids.iter_mut().zip(adjustments.iter()) {
        boid.vel += *adjustment / SEPARATION_FACTOR;
    }
}

//fn calc_color(boid: &Boid) -> Color {
//    Color::new(
//        boid.vel.x + MAX_SPEED,
//        boid.vel.y + MAX_SPEED,
//        boid.vel.x + boid.vel.y + MAX_SPEED,
//        1.0,
//    )
//}
fn calc_color(boid: &Boid) -> Color {
    // https://gist.github.com/popcorn245/30afa0f98eea1c2fd34d
    // https://babelcolor.com/index_htm_files/A%20review%20of%20RGB%20color%20spaces.pdf
    let big_y = 1.0;
    let x = (boid.vel.normalize().x + 1.) / 2.;
    let y = (boid.vel.normalize().y + 1.) / 2.;
    let big_x = x * (big_y / y);
    let big_z = (1. - x - y) * (big_y / y);
    //let red_value = 2.3707 * big_x - 0.9001 * big_y - 0.4706 * big_z;
    //let green_value = -0.5139 * big_x + 1.4253 * big_y + 0.0806 * big_z;
    //let blue_value = 0.0053 * big_x - 2.807 * big_y + 1.0094 * big_z;
    let red_value = (1.3707 * big_x - 0.9001 * big_y - 0.4706 * big_z).clamp(0.2, 0.6);
    let green_value = (-0.5139 * big_x + 1.4253 * big_y + 0.0806 * big_z).clamp(0.2, 0.6);
    let blue_value = (1.0053 * big_x - 1.807 * big_y + 2.0094 * big_z).clamp(0.6, 1.0);
    Color::new(red_value, green_value, blue_value, 1.0)
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
