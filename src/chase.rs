use crate::canvas::Canvas;
use crate::color::Rgb;
use crate::maze::{Direction, Maze, Tile};
use crate::sprites::{
    draw_ghost, draw_kc_munchkin, draw_pellet, draw_super_pellet, GhostConfig, GhostState,
};
use rand::RngExt;
use std::f32::consts::PI;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Phase {
    /// KC approaches the super pellet, ghost is wandering through maze
    ApproachingPellet,
    /// Pellet just eaten, power surge shockwave
    PelletChomp { timer: f32 },
    /// KC is supercharged and actively chasing the scared ghost
    KcChasing { duration: f32 },
    /// Super pellet effect is wearing off, ghost flashing rapidly
    PowerFading { duration: f32 },
    /// Ghost is chasing KC, KC is fleeing in panic through the maze
    GhostChasing { duration: f32 },
    /// One caught the other! Arcade catch celebration before refreshing maze
    CatchCelebration {
        timer: f32,
        kc_caught_ghost: bool,
        cx: f32,
        cy: f32,
    },
}

#[derive(Debug, Clone)]
pub struct Particle {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub life: f32,
    pub max_life: f32,
    pub color: Rgb,
    pub size: f32,
}

#[derive(Debug, Clone)]
pub struct MazeEntity {
    pub x: f32,
    pub y: f32,
    pub col: i32,
    pub row: i32,
    pub dir: Direction,
    pub speed: f32,
    pub target_col: i32,
    pub target_row: i32,
}

impl MazeEntity {
    pub fn new(col: i32, row: i32, dir: Direction, speed: f32, maze: &Maze) -> Self {
        let (x, y) = maze.tile_to_pixel(col as f32, row as f32);
        MazeEntity {
            x,
            y,
            col,
            row,
            dir,
            speed,
            target_col: col,
            target_row: row,
        }
    }

    /// Move along corridor towards next tile center. Returns true when reached a tile center.
    pub fn advance(&mut self, dt: f32, maze: &Maze) -> bool {
        let (target_x, target_y) =
            maze.tile_to_pixel(self.target_col as f32, self.target_row as f32);

        let move_dist = self.speed * dt;
        let dx = target_x - self.x;
        let dy = target_y - self.y;
        let dist = (dx * dx + dy * dy).sqrt();

        // Check horizontal tunnel wrap
        let canvas_w = maze.cols as f32 * maze.cell_w;
        if self.dir == Direction::Left && self.x < 0.0 {
            self.x += canvas_w;
            self.col = maze.cols as i32 - 1;
            self.target_col = self.col - 1;
            return false;
        } else if self.dir == Direction::Right && self.x > canvas_w {
            self.x -= canvas_w;
            self.col = 0;
            self.target_col = 1;
            return false;
        }

        if dist <= move_dist || dist < 0.1 {
            // Reached target tile center!
            self.x = target_x;
            self.y = target_y;
            self.col = self.target_col;
            self.row = self.target_row;
            true
        } else {
            self.x += (dx / dist) * move_dist;
            self.y += (dy / dist) * move_dist;
            false
        }
    }

    /// Set next step direction from current tile center
    pub fn set_next_direction(&mut self, next_dir: Direction, maze: &Maze) {
        self.dir = next_dir;
        let mut nc = self.col + next_dir.dx();
        let nr = self.row + next_dir.dy();

        // Handle side warp tunnels
        if nc < 0 {
            nc = maze.cols as i32 - 1;
        } else if nc >= maze.cols as i32 {
            nc = 0;
        }

        if maze.is_passable(nc, nr) {
            self.target_col = nc;
            self.target_row = nr;
        } else {
            // Stay in place if impassable
            self.target_col = self.col;
            self.target_row = self.row;
        }
    }
}

pub struct ChaseSimulation {
    pub maze: Maze,

    pub kc: MazeEntity,
    pub kc_mouth: f32,

    pub ghost: MazeEntity,
    pub ghost_config: GhostConfig,

    pub super_pellet_col: i32,
    pub super_pellet_row: i32,
    pub super_pellet_active: bool,

    pub pellets: Vec<(i32, i32, bool)>,
    pub particles: Vec<Particle>,

    pub phase: Phase,
    pub anim_time: f32,
    pub pass_count: usize,

    pub canvas_w: usize,
    pub canvas_h: usize,
}

impl ChaseSimulation {
    pub fn new(canvas_w: usize, canvas_h: usize) -> Self {
        let maze = Maze::generate(canvas_w, canvas_h);
        let ghost_config = GhostConfig::red_bit();

        let mut sim = ChaseSimulation {
            maze,
            kc: MazeEntity::new(1, 1, Direction::Right, 45.0, &Maze::generate(10, 10)),
            kc_mouth: 0.5,
            ghost: MazeEntity::new(1, 1, Direction::Left, 40.0, &Maze::generate(10, 10)),
            ghost_config,
            super_pellet_col: 1,
            super_pellet_row: 1,
            super_pellet_active: true,
            pellets: Vec::new(),
            particles: Vec::new(),
            phase: Phase::ApproachingPellet,
            anim_time: 0.0,
            pass_count: 0,
            canvas_w,
            canvas_h,
        };

        sim.reset_maze_pass();
        sim
    }

    /// Resize simulation arena when terminal size changes
    pub fn resize(&mut self, canvas_w: usize, canvas_h: usize) {
        self.canvas_w = canvas_w;
        self.canvas_h = canvas_h;
        self.reset_maze_pass();
    }

    /// Generate a brand new random maze layout with fresh walls, colors, pellet paths, and sizes!
    pub fn reset_maze_pass(&mut self) {
        self.pass_count += 1;
        let mut rng = rand::rng();

        // 1. Generate new random arcade maze
        self.maze = Maze::generate(self.canvas_w, self.canvas_h);

        // 2. Randomize ghost character
        self.ghost_config = match rng.random_range(0..4) {
            0 => GhostConfig::red_bit(),
            1 => GhostConfig::purple_byte(),
            2 => GhostConfig::green_middie(),
            _ => GhostConfig::orange_muncher(),
        };

        // 3. Find all available path tiles
        let mut path_tiles = Vec::new();
        for r in 1..self.maze.rows - 1 {
            for c in 1..self.maze.cols - 1 {
                if self.maze.tiles[r * self.maze.cols + c] == Tile::Path {
                    path_tiles.push((c as i32, r as i32));
                }
            }
        }

        if path_tiles.is_empty() {
            path_tiles.push((1, 1));
        }

        // 4. Place Super Pellet in one of the path tiles (prefer outer corners/quadrants)
        let super_idx = rng.random_range(0..path_tiles.len());
        let (sp_c, sp_r) = path_tiles[super_idx];
        self.super_pellet_col = sp_c;
        self.super_pellet_row = sp_r;
        self.super_pellet_active = true;

        // 5. Place KC and Ghost on path tiles at good distance from each other
        let kc_idx = rng.random_range(0..path_tiles.len());
        let (kc_c, kc_r) = path_tiles[kc_idx];

        // Find ghost tile with maximum distance from KC
        let mut best_ghost_tile = path_tiles[0];
        let mut max_dist = -1.0;
        for &(gc, gr) in &path_tiles {
            let dc = gc - kc_c;
            let dr = gr - kc_r;
            let dist = (dc * dc + dr * dr) as f32;
            if dist > max_dist {
                max_dist = dist;
                best_ghost_tile = (gc, gr);
            }
        }

        // Base speed scales naturally with corridor size
        let base_speed = (self.maze.cell_w.min(self.maze.cell_h) * 2.8).clamp(30.0, 70.0);

        self.kc = MazeEntity::new(kc_c, kc_r, Direction::Right, base_speed, &self.maze);
        self.ghost = MazeEntity::new(
            best_ghost_tile.0,
            best_ghost_tile.1,
            Direction::Left,
            base_speed * 0.85,
            &self.maze,
        );

        // 6. Populate maze corridors with normal pellets
        self.pellets.clear();
        for &(pc, pr) in &path_tiles {
            if !(pc == sp_c && pr == sp_r) && !(pc == kc_c && pr == kc_r) {
                // Sparsely place pellets along paths
                if rng.random_bool(0.75) {
                    self.pellets.push((pc, pr, true));
                }
            }
        }

        self.phase = Phase::ApproachingPellet;
    }

    pub fn character_radius(&self) -> f32 {
        (self.maze.cell_w.min(self.maze.cell_h) * 0.44).clamp(3.5, 12.0)
    }

    /// Advance physics, AI, and animations by dt (seconds)
    pub fn update(&mut self, dt: f32) {
        self.anim_time += dt;

        // Animate mouth chomping
        let chomp_speed = match self.phase {
            Phase::ApproachingPellet => 11.0,
            Phase::KcChasing { .. } => 20.0,
            Phase::PowerFading { .. } => 15.0,
            Phase::GhostChasing { .. } => 25.0, // Chomping frantically in panic!
            _ => 8.0,
        };
        self.kc_mouth = ((self.anim_time * chomp_speed).sin() + 1.0) * 0.5;

        // Update active particles
        for p in self.particles.iter_mut() {
            p.x += p.vx * dt;
            p.y += p.vy * dt;
            p.life -= dt;
        }
        self.particles.retain(|p| p.life > 0.0);

        // Chomp normal pellets along KC's path
        let mut eaten_sparkles = Vec::new();
        for (pc, pr, active) in self.pellets.iter_mut() {
            if *active && *pc == self.kc.col && *pr == self.kc.row {
                *active = false;
                let (px, py) = self.maze.tile_to_pixel(*pc as f32, *pr as f32);
                eaten_sparkles.push((px, py));
            }
        }
        for (px, py) in eaten_sparkles {
            self.spawn_sparkles(px, py, Rgb::new(255, 230, 80), 4);
        }

        let base_speed = (self.maze.cell_w.min(self.maze.cell_h) * 2.8).clamp(30.0, 70.0);
        let mut next_phase = self.phase;

        match self.phase {
            Phase::ApproachingPellet => {
                self.kc.speed = base_speed;
                self.ghost.speed = base_speed * 0.75;

                // Move KC
                if self.kc.advance(dt, &self.maze) {
                    // At tile center: BFS pathfind toward Super Pellet
                    if let Some(next_dir) = self.maze.bfs_next_direction(
                        self.kc.col,
                        self.kc.row,
                        self.super_pellet_col,
                        self.super_pellet_row,
                    ) {
                        self.kc.set_next_direction(next_dir, &self.maze);
                    } else {
                        // Reached Super Pellet tile!
                        self.super_pellet_active = false;
                        let (spx, spy) = self
                            .maze
                            .tile_to_pixel(self.super_pellet_col as f32, self.super_pellet_row as f32);
                        self.spawn_shockwave(spx, spy);
                        next_phase = Phase::PelletChomp { timer: 0.6 };
                    }
                }

                // Move Ghost (wanders casually through maze corridors)
                if self.ghost.advance(dt, &self.maze) {
                    let next_dir = self.ghost_wander_direction();
                    self.ghost.set_next_direction(next_dir, &self.maze);
                }
            }

            Phase::PelletChomp { timer } => {
                let new_timer = timer - dt;
                if new_timer <= 0.0 {
                    // KC starts chasing ghost!
                    next_phase = Phase::KcChasing { duration: 9.0 };
                } else {
                    next_phase = Phase::PelletChomp { timer: new_timer };
                }
            }

            Phase::KcChasing { duration } => {
                let new_duration = duration - dt;
                self.kc.speed = base_speed * 1.25; // Supercharged speed!
                self.ghost.speed = base_speed * 0.95;

                // KC pursues Ghost using BFS shortest path
                if self.kc.advance(dt, &self.maze) {
                    if let Some(next_dir) = self.maze.bfs_next_direction(
                        self.kc.col,
                        self.kc.row,
                        self.ghost.col,
                        self.ghost.row,
                    ) {
                        self.kc.set_next_direction(next_dir, &self.maze);
                    }
                }

                // Ghost flees away from KC at intersections
                if self.ghost.advance(dt, &self.maze) {
                    let flee_dir = self.maze.flee_direction(
                        self.ghost.col,
                        self.ghost.row,
                        self.kc.col,
                        self.kc.row,
                        self.ghost.dir,
                    );
                    self.ghost.set_next_direction(flee_dir, &self.maze);
                }

                // Check if KC catches the Ghost!
                let dx = self.kc.x - self.ghost.x;
                let dy = self.kc.y - self.ghost.y;
                let dist = (dx * dx + dy * dy).sqrt();
                let catch_threshold = self.character_radius() * 1.6;

                if dist < catch_threshold {
                    // KC CAUGHT THE GHOST!
                    self.spawn_shockwave(self.ghost.x, self.ghost.y);
                    self.spawn_sparkles(self.ghost.x, self.ghost.y, Rgb::new(100, 200, 255), 18);
                    next_phase = Phase::CatchCelebration {
                        timer: 1.3,
                        kc_caught_ghost: true,
                        cx: self.ghost.x,
                        cy: self.ghost.y,
                    };
                } else if new_duration <= 0.0 {
                    // Power wearing off
                    next_phase = Phase::PowerFading { duration: 3.5 };
                } else {
                    next_phase = Phase::KcChasing {
                        duration: new_duration,
                    };
                }
            }

            Phase::PowerFading { duration } => {
                let new_duration = duration - dt;
                self.kc.speed = base_speed * 0.95;
                self.ghost.speed = base_speed * 0.95;

                if self.kc.advance(dt, &self.maze) {
                    if let Some(next_dir) = self.maze.bfs_next_direction(
                        self.kc.col,
                        self.kc.row,
                        self.ghost.col,
                        self.ghost.row,
                    ) {
                        self.kc.set_next_direction(next_dir, &self.maze);
                    }
                }

                if self.ghost.advance(dt, &self.maze) {
                    let flee_dir = self.maze.flee_direction(
                        self.ghost.col,
                        self.ghost.row,
                        self.kc.col,
                        self.kc.row,
                        self.ghost.dir,
                    );
                    self.ghost.set_next_direction(flee_dir, &self.maze);
                }

                if new_duration <= 0.0 {
                    // Effect completely faded! Ghost turns around and hunts KC!
                    self.spawn_shockwave(self.ghost.x, self.ghost.y);
                    next_phase = Phase::GhostChasing { duration: 12.0 };
                } else {
                    next_phase = Phase::PowerFading {
                        duration: new_duration,
                    };
                }
            }

            Phase::GhostChasing { duration } => {
                let new_duration = duration - dt;
                self.ghost.speed = base_speed * 1.18; // Ghost is fast and aggressive!
                self.kc.speed = base_speed * 1.05;

                // Ghost hunts KC using BFS shortest path!
                if self.ghost.advance(dt, &self.maze) {
                    if let Some(next_dir) = self.maze.bfs_next_direction(
                        self.ghost.col,
                        self.ghost.row,
                        self.kc.col,
                        self.kc.row,
                    ) {
                        self.ghost.set_next_direction(next_dir, &self.maze);
                    }
                }

                // KC flees in panic away from Ghost!
                if self.kc.advance(dt, &self.maze) {
                    let flee_dir = self.maze.flee_direction(
                        self.kc.col,
                        self.kc.row,
                        self.ghost.col,
                        self.ghost.row,
                        self.kc.dir,
                    );
                    self.kc.set_next_direction(flee_dir, &self.maze);
                }

                // Check if Ghost catches KC!
                let dx = self.ghost.x - self.kc.x;
                let dy = self.ghost.y - self.kc.y;
                let dist = (dx * dx + dy * dy).sqrt();
                let catch_threshold = self.character_radius() * 1.6;

                if dist < catch_threshold {
                    // GHOST CAUGHT KC!
                    self.spawn_shockwave(self.kc.x, self.kc.y);
                    self.spawn_sparkles(self.kc.x, self.kc.y, Rgb::new(255, 60, 60), 20);
                    next_phase = Phase::CatchCelebration {
                        timer: 1.3,
                        kc_caught_ghost: false,
                        cx: self.kc.x,
                        cy: self.kc.y,
                    };
                } else if new_duration <= 0.0 {
                    // If chase goes on long without a catch, transition to new maze anyway
                    next_phase = Phase::CatchCelebration {
                        timer: 0.5,
                        kc_caught_ghost: false,
                        cx: self.kc.x,
                        cy: self.kc.y,
                    };
                } else {
                    next_phase = Phase::GhostChasing {
                        duration: new_duration,
                    };
                }
            }

            Phase::CatchCelebration {
                timer,
                kc_caught_ghost: _,
                cx: _,
                cy: _,
            } => {
                let new_timer = timer - dt;
                if new_timer <= 0.0 {
                    // REFRESH TO A BRAND NEW RANDOM MAZE PATTERN!
                    // This changes all walls, corners, colors, and layout to prevent screen burn!
                    self.reset_maze_pass();
                    return;
                } else {
                    next_phase = Phase::CatchCelebration {
                        timer: new_timer,
                        kc_caught_ghost: true,
                        cx: self.kc.x,
                        cy: self.kc.y,
                    };
                }
            }
        }

        self.phase = next_phase;
    }

    /// Render the full screensaver frame onto canvas
    pub fn draw(&self, canvas: &mut Canvas) {
        canvas.clear();

        // 1. Draw procedural neon arcade maze walls
        self.maze.draw(canvas, self.anim_time);

        // 2. Draw normal pellets along corridors
        for &(pc, pr, active) in &self.pellets {
            if active {
                let (px, py) = self.maze.tile_to_pixel(pc as f32, pr as f32);
                draw_pellet(canvas, px, py, self.anim_time);
            }
        }

        // 3. Draw Super Pellet if active
        if self.super_pellet_active {
            let (spx, spy) = self
                .maze
                .tile_to_pixel(self.super_pellet_col as f32, self.super_pellet_row as f32);
            let pellet_rad = (self.character_radius() * 0.7).max(2.2);
            draw_super_pellet(canvas, spx, spy, pellet_rad, self.anim_time);
        }

        // 4. Draw active particles
        for p in &self.particles {
            let alpha = (p.life / p.max_life).clamp(0.0, 1.0);
            let col = p.color.lerp(Rgb::BLACK, 1.0 - alpha);
            canvas.fill_circle_smooth(p.x, p.y, p.size, col);
        }

        let radius = self.character_radius();

        // 5. Draw KC Munchkin
        let is_supercharged =
            matches!(self.phase, Phase::KcChasing { .. } | Phase::PelletChomp { .. });
        let is_terrified = matches!(self.phase, Phase::GhostChasing { .. });

        // If KC was caught during celebration, don't draw or fade KC
        let draw_kc = match self.phase {
            Phase::CatchCelebration {
                kc_caught_ghost,
                timer,
                ..
            } => kc_caught_ghost || (timer > 0.6),
            _ => true,
        };

        if draw_kc {
            draw_kc_munchkin(
                canvas,
                self.kc.x,
                self.kc.y,
                radius,
                self.kc.dir.to_angle(),
                self.kc_mouth,
                is_supercharged,
                is_terrified,
                self.anim_time,
            );
        }

        // 6. Draw Ghost
        let ghost_state = match self.phase {
            Phase::ApproachingPellet => GhostState::Patrolling,
            Phase::PelletChomp { .. } | Phase::KcChasing { .. } => GhostState::Vulnerable,
            Phase::PowerFading { duration } => GhostState::Fading {
                time_remaining: duration,
            },
            Phase::GhostChasing { .. } => GhostState::Hunting,
            Phase::CatchCelebration {
                kc_caught_ghost, ..
            } => {
                if kc_caught_ghost {
                    GhostState::Vulnerable
                } else {
                    GhostState::Hunting
                }
            }
        };

        let draw_ghost_entity = match self.phase {
            Phase::CatchCelebration {
                kc_caught_ghost,
                timer,
                ..
            } => !kc_caught_ghost || (timer > 0.6),
            _ => true,
        };

        if draw_ghost_entity {
            draw_ghost(
                canvas,
                self.ghost.x,
                self.ghost.y,
                radius,
                self.ghost.dir.to_angle(),
                ghost_state,
                &self.ghost_config,
                self.anim_time,
            );
        }
    }

    fn ghost_wander_direction(&self) -> Direction {
        let available = self.maze.available_directions(self.ghost.col, self.ghost.row);
        if available.is_empty() {
            return self.ghost.dir;
        }

        // Avoid turning directly backwards unless in dead end
        let forward_choices: Vec<Direction> = available
            .iter()
            .copied()
            .filter(|&d| d != self.ghost.dir.opposite())
            .collect();

        let mut rng = rand::rng();
        if !forward_choices.is_empty() {
            forward_choices[rng.random_range(0..forward_choices.len())]
        } else {
            available[rng.random_range(0..available.len())]
        }
    }

    fn spawn_shockwave(&mut self, cx: f32, cy: f32) {
        let num_particles = 24;
        let mut rng = rand::rng();
        for i in 0..num_particles {
            let angle = (i as f32 / num_particles as f32) * 2.0 * PI;
            let speed = rng.random_range(25.0..60.0);
            let color = match i % 3 {
                0 => Rgb::new(255, 230, 40),
                1 => Rgb::new(255, 120, 240),
                _ => Rgb::WHITE,
            };
            self.particles.push(Particle {
                x: cx,
                y: cy,
                vx: angle.cos() * speed,
                vy: angle.sin() * speed,
                life: rng.random_range(0.4..0.8),
                max_life: 0.8,
                color,
                size: rng.random_range(1.0..2.0),
            });
        }
    }

    fn spawn_sparkles(&mut self, cx: f32, cy: f32, color: Rgb, count: usize) {
        let mut rng = rand::rng();
        for _ in 0..count {
            let angle = rng.random_range(0.0..2.0 * PI);
            let speed = rng.random_range(12.0..30.0);
            self.particles.push(Particle {
                x: cx,
                y: cy,
                vx: angle.cos() * speed,
                vy: angle.sin() * speed,
                life: rng.random_range(0.25..0.45),
                max_life: 0.45,
                color,
                size: 0.9,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_maze_chase_simulation() {
        let mut sim = ChaseSimulation::new(100, 50);
        assert_eq!(sim.phase, Phase::ApproachingPellet);

        // Advance simulation
        for _ in 0..100 {
            sim.update(0.04);
        }

        let mut canvas = Canvas::new(50, 25);
        sim.draw(&mut canvas);
    }
}
