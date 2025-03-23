use std::process;

use macroquad::{prelude::*, window};

const BOID_HEIGHT: f32 = 13.;
const BOID_BASE: f32 = 8.;
const BOID_COUNT: u32 = 15000;
const MAX_SPEED: f32 = 0.5;

const SEPARATION_FACTOR: f32 = 300.;
const SEPARATION_DISTANCE_THRESHOLD: f32 = 12.;

const CLOSE_SEPARATION_FACTOR: f32 = 1.;
const CLOSE_SEPARATION_DISTANCE_THRESHOLD: f32 = 2.;

const COHESION_FACTOR: f32 = 400.;
const COHESION_DISTANCE_THRESHOLD: f32 = 20.;
// const SWIRL_FACTOR: f32 = 0.0;

const ALIGNMENT_FACTOR: f32 = 40.; // lower this gently when you get back, This is really good below 1 as a natural drive
const ALIGNMENT_DISTANCE_THRESHOLD: f32 = 14.;

const MAXIMUM_DISTANCE: f32 = 20.; // UPDATE TO MATCH MAX OF RULE_FACTORS! (this is so bad...)

const DRIVE_FACTOR: f32 = 0.04;

const CELL_SIZE: f32 = 30.;

const DEBUG_ENABLED: bool = false;

// Hard coded because Lazy static kills performance
// Move simulation to a struct and make these properties for dynamism
const SCREEN_HEIGHT: f32 = 1080.;
const SCREEN_WIDTH: f32 = 1920.;

const HALF_SCREEN_HEIGHT: f32 = 1080. / 2.0;
const HALF_SCREEN_WIDTH: f32 = 1920. / 2.0;

#[derive(Clone, Debug)]
pub struct Boid {
    pos: Vec2,
    rot: f32,
    vel: Vec2,
}

/// This runs better than non-branching check?
/// Maybe I'm doing it wrong?
///
/// # Non-Branching
/// ```
/// fn toroidal_diff(a: Vec2, b: Vec2) -> Vec2 {
///    let w = SCREEN_WIDTH;
///    let h = SCREEN_HEIGHT;
///    let half_w = w * 0.5;
///    let half_h = h * 0.5;
///    let dx = (a.x - b.x + half_w).rem_euclid(w) - half_w;
///    let dy = (a.y - b.y + half_h).rem_euclid(h) - half_h;
///    vec2(dx, dy)
/// }
/// ```
#[inline(always)]
fn toroidal_diff(a: Vec2, b: Vec2) -> Vec2 {
    let w = SCREEN_WIDTH;
    let h = SCREEN_HEIGHT;
    let mut dx = a.x - b.x;
    let mut dy = a.y - b.y;
    if dx > HALF_SCREEN_WIDTH {
        dx -= w;
    } else if dx < -HALF_SCREEN_WIDTH {
        dx += w;
    }
    if dy > HALF_SCREEN_HEIGHT {
        dy -= h;
    } else if dy < -HALF_SCREEN_HEIGHT {
        dy += h;
    }
    vec2(dx, dy)
}

/// Initialize SpatialGrid with `new`.
/// Clear the internal map with `clear_grid` at the beginning of every loop.
/// Use `register_pos` on each boid to fill the grid after clearing.
/// Use `get_neighbors` in rules to get the indexes of a boid's neighbors.
pub struct SpatialGrid {
    cell_size: f32,
    grid_rows: i32,
    grid_cols: i32,
    /// First Vec is row/cols
    /// Second Vec is list at that row/col index
    /// usize is index so we can ignore ourselves when grabbing neighbors
    /// First Vec2 is current position, Second Vec2 is current velocity
    cells: Vec<Vec<(usize, Vec2, Vec2)>>,
}

impl SpatialGrid {
    pub fn new() -> Self {
        let grid_cols = (SCREEN_WIDTH / CELL_SIZE).ceil() as i32;
        let grid_rows = (SCREEN_HEIGHT / CELL_SIZE).ceil() as i32;
        let total_cells = (grid_rows * grid_cols) as usize;
        Self {
            cell_size: CELL_SIZE,
            grid_rows,
            grid_cols,
            cells: vec![Vec::new(); total_cells],
        }
    }

    /// Clear each cell's vector at the start of each frame.
    pub fn clear_grid(&mut self) {
        for cell in &mut self.cells {
            cell.clear();
        }
    }

    /// Register the position of a boid by computing its cell index.
    pub fn register_pos(&mut self, index: usize, boid: &Boid) {
        let cell = self.get_cell(boid.pos);
        // Wrap the cell coordinates toroidally.
        let wrapped_cell = ivec2(
            cell.x.rem_euclid(self.grid_cols),
            cell.y.rem_euclid(self.grid_rows),
        );
        let flat_index = (wrapped_cell.x + wrapped_cell.y * self.grid_cols) as usize;
        self.cells[flat_index].push((index, boid.pos, boid.vel));
    }

    /// Returns neighbors' toroidal position diff and neighbors' velocity in a tuple
    pub fn get_neighbors(
        &self,
        index: usize,
        current_pos: Vec2,
        view_distance: f32,
    ) -> Vec<(Vec2, Vec2)> {
        let current_cell = self.get_cell(current_pos);
        let radius = (view_distance / self.cell_size).ceil() as i32; // radius in cells
        let mut neighbors = Vec::new();

        for dx in -radius..=radius {
            for dy in -radius..=radius {
                let cell = current_cell + ivec2(dx, dy);
                let wrapped_cell = ivec2(
                    cell.x.rem_euclid(self.grid_cols),
                    cell.y.rem_euclid(self.grid_rows),
                );
                let flat_index = (wrapped_cell.x + wrapped_cell.y * self.grid_cols) as usize;
                for (neighbor_index, pos, vel) in self.cells[flat_index].iter() {
                    if *neighbor_index != index {
                        let diff = toroidal_diff(*pos, current_pos); // was toroidal diff
                                                                     // Rules will filter out neighbors beyond view_distance:
                                                                     // TODO: Test if filtering out by view_distance here helps performance
                        if diff.length() < view_distance {
                            neighbors.push((diff, *vel));
                        }
                    }
                }
            }
        }
        neighbors
    }

    /// Computes the grid cell from a position.
    fn get_cell(&self, pos: Vec2) -> IVec2 {
        (pos / self.cell_size).floor().as_ivec2()
    }
}

#[macroquad::main("Boids")]
async fn main() {
    let pid = process::id();
    println!("My PID is: {}", pid);
    window::set_fullscreen(true);

    // Wait until the window is fullscreen-ish.
    while screen_height() < 700. {
        clear_background(BLACK);
        draw_text("Waiting for fullscreen...", 20.0, 20.0, 30.0, WHITE);
        next_frame().await;
    }

    let width = screen_width();
    let height = screen_height();

    // Calculate grid dimensions for an approximately square grid.
    let grid_cols = (BOID_COUNT as f32).sqrt().ceil() as u32;
    let grid_rows = grid_cols; // Using a square grid

    // Calculate the spacing between boids.
    let col_size = width / grid_cols as f32;
    let row_size = height / grid_rows as f32;

    // Create positions for each grid cell.
    let positions: Vec<Vec2> = (0..grid_rows)
        .flat_map(|j| {
            (0..grid_cols).map(move |i| {
                // Optionally, add col_size/2 and row_size/2 to center the boid in its cell.
                vec2(
                    i as f32 * col_size + col_size / 2.0,
                    j as f32 * row_size + row_size / 2.0,
                )
            })
        })
        .take(BOID_COUNT as usize) // Only take as many positions as needed.
        .collect();

    let mut boids: Vec<Boid> = (0..BOID_COUNT)
        .map(|index| Boid {
            pos: positions[index as usize],
            rot: 0.,
            vel: vec2(
                rand::gen_range(-MAX_SPEED, MAX_SPEED) / 2.,
                rand::gen_range(-MAX_SPEED, MAX_SPEED) / 2.,
            ),
        })
        .collect();

    let mut spatial_grid = SpatialGrid::new();

    loop {
        spatial_grid.clear_grid();
        for (index, boid) in boids.iter().enumerate() {
            spatial_grid.register_pos(index, boid);
        }

        move_boids(&mut boids, &spatial_grid);

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
            let color = if i == 0 && DEBUG_ENABLED {
                RED
            } else {
                calc_color(&boid)
            };
            draw_triangle(v1, v2, v3, color);
        }

        next_frame().await;
    }
}

fn move_boids(boids: &mut Vec<Boid>, grid: &SpatialGrid) {
    for (index, mut boid) in boids.iter_mut().enumerate() {
        let neighbors = grid.get_neighbors(index, boid.pos, MAXIMUM_DISTANCE);
        apply_rules(&mut boid, &neighbors);
    }
    for boid in boids.iter_mut() {
        let target_rotation = boid.vel.x.atan2(-boid.vel.y);
        boid.rot = target_rotation;

        boid.vel += boid.vel.normalize() * DRIVE_FACTOR;

        if boid.vel.length() > MAX_SPEED {
            boid.vel = boid.vel.normalize() * MAX_SPEED
        }
        boid.pos += boid.vel;

        boid.pos = wrap_around(&boid.pos);
    }
}
fn apply_rules(current_boid: &mut Boid, neighbors: &[(Vec2, Vec2)]) {
    // Sums for the three rules
    let mut center = vec2(0., 0.);
    let mut center_count = 0;

    let mut vel_sum = vec2(0., 0.);
    let mut vel_count = 0;

    let mut separation = vec2(0., 0.);
    let mut close_separation = vec2(0., 0.);

    let mut net_force = vec2(0., 0.);

    for (diff, vel) in neighbors.iter() {
        let dist = diff.length();

        // Cohesion
        if dist < COHESION_DISTANCE_THRESHOLD {
            center += *diff + current_boid.pos;
            center_count += 1;
        }

        // Alignment
        if dist < ALIGNMENT_DISTANCE_THRESHOLD {
            vel_sum += *vel;
            vel_count += 1;
        }

        // Separation
        if dist < SEPARATION_DISTANCE_THRESHOLD {
            separation -= *diff;
        }
        if dist < CLOSE_SEPARATION_DISTANCE_THRESHOLD {
            close_separation -= *diff;
        }
    }

    // Cohesion
    // Aligns the boids velocity with the boids around it.
    //
    // 1. Find the average velocity.
    // 2. Determine velocity of others.
    // 3. Align our boid a percentage of the way.
    if center_count > 0 {
        let perceived_center = center / (center_count as f32) - current_boid.pos;
        // let perpendicular = if perceived_center.length() != 0.0 {
        //     vec2(-perceived_center.y, -perceived_center.x).normalize()
        // } else {
        //     vec2(0.0, 0.0)
        // };
        // net_force += (perceived_center + perpendicular * SWIRL_FACTOR) / COHESION_FACTOR;
        net_force += perceived_center / COHESION_FACTOR;
    }

    // Alignment
    // Keep our boid going in the directions of its friends.
    if vel_count > 0 {
        let perceived_velocity = vel_sum / (vel_count as f32);
        net_force += (perceived_velocity - current_boid.vel) / ALIGNMENT_FACTOR;
    }

    // Separation
    // Keep our boid away from other boids.
    //
    // 1. Create adjustments vec2 for each boid.
    // 2. Get the diff in distance for boids many to many.
    // 3. If diff is less than constant, adjust boid directly away?
    net_force += separation / SEPARATION_FACTOR;
    net_force += close_separation / CLOSE_SEPARATION_FACTOR;

    current_boid.vel += net_force;
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
    if vr.x > SCREEN_WIDTH {
        vr.x = 0.;
    }
    if vr.x < 0. {
        vr.x = SCREEN_WIDTH;
    }
    if vr.y > SCREEN_HEIGHT {
        vr.y = 0.;
    }
    if vr.y < 0. {
        vr.y = SCREEN_HEIGHT;
    }
    vr
}
