use crate::canvas::Canvas;
use crate::color::Rgb;
use crate::maze::{Direction, Maze, Tile};
use crate::sprites::{
    draw_ghost, draw_ghost_death, draw_kc_cheering, draw_kc_munchkin, draw_pellet,
    draw_super_pellet, GhostConfig, GhostState,
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
    /// KC caught the ghost! Ghost plays its death animation first
    GhostDeath {
        timer: f32,
        max_timer: f32,
        ghost_x: f32,
        ghost_y: f32,
    },
    /// Immediately after ghost death: KC faces the camera with a cute toothy smile,
    /// quickly grows arms and legs at 45 degree angles, and pumps them to cheer!
    KcCheer {
        timer: f32,
        cheer_time: f32,
    },
    /// Ghost caught KC: defeat effect before maze refresh
    GhostCatchKc {
        timer: f32,
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
    pub warping: bool,
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
            warping: false,
        }
    }

    /// Move along corridor towards next tile center. Returns true when reached a tile center.
    pub fn advance(&mut self, dt: f32, maze: &Maze) -> bool {
        let canvas_w = maze.cols as f32 * maze.cell_w;
        let warp_r = maze.warp_row.unwrap_or(maze.rows / 2) as i32;
        let move_dist = self.speed * dt;

        // If currently emerging from warp tunnel
        if self.warping {
            if self.dir == Direction::Left {
                // Moving left from outside the right edge towards tile cols - 1
                let (target_x, target_y) = maze.tile_to_pixel((maze.cols - 1) as f32, warp_r as f32);
                self.x -= move_dist;
                self.y = target_y;
                if self.x <= target_x {
                    self.x = target_x;
                    self.col = maze.cols as i32 - 1;
                    self.row = warp_r;
                    self.target_col = self.col;
                    self.target_row = self.row;
                    self.warping = false;
                    return true;
                }
                return false;
            } else if self.dir == Direction::Right {
                // Moving right from outside the left edge towards tile 0
                let (target_x, target_y) = maze.tile_to_pixel(0.0, warp_r as f32);
                self.x += move_dist;
                self.y = target_y;
                if self.x >= target_x {
                    self.x = target_x;
                    self.col = 0;
                    self.row = warp_r;
                    self.target_col = 0;
                    self.target_row = warp_r;
                    self.warping = false;
                    return true;
                }
                return false;
            }
        }

        // Check entering left warp tunnel: moving left from col 0
        if self.dir == Direction::Left && self.col == 0 && self.row == warp_r && self.target_col == -1 {
            self.x -= move_dist;
            // When fully exited off left screen
            if self.x <= -maze.cell_w * 0.5 {
                // Teleport to right side!
                self.x = canvas_w + maze.cell_w * 0.5;
                self.col = maze.cols as i32 - 1;
                self.row = warp_r;
                self.target_col = maze.cols as i32 - 1;
                self.target_row = warp_r;
                self.warping = true;
            }
            return false;
        }

        // Check entering right warp tunnel: moving right from col cols - 1
        if self.dir == Direction::Right
            && self.col == maze.cols as i32 - 1
            && self.row == warp_r
            && self.target_col == maze.cols as i32
        {
            self.x += move_dist;
            // When fully exited off right screen
            if self.x >= canvas_w + maze.cell_w * 0.5 {
                // Teleport to left side!
                self.x = -maze.cell_w * 0.5;
                self.col = 0;
                self.row = warp_r;
                self.target_col = 0;
                self.target_row = warp_r;
                self.warping = true;
            }
            return false;
        }

        // Standard movement to adjacent tile
        let (target_x, target_y) = maze.tile_to_pixel(self.target_col as f32, self.target_row as f32);
        let dx = target_x - self.x;
        let dy = target_y - self.y;
        let dist = (dx * dx + dy * dy).sqrt();

        if dist <= move_dist || dist < 0.1 {
            // Arrived at target tile center!
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
        let warp_r = maze.warp_row.unwrap_or(maze.rows / 2) as i32;
        self.dir = next_dir;

        // Check warp tunnel navigation
        if self.row == warp_r {
            if self.col == 0 && next_dir == Direction::Left {
                self.target_col = -1;
                self.target_row = warp_r;
                return;
            }
            if self.col == maze.cols as i32 - 1 && next_dir == Direction::Right {
                self.target_col = maze.cols as i32;
                self.target_row = warp_r;
                return;
            }
        }

        let nc = self.col + next_dir.dx();
        let nr = self.row + next_dir.dy();

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SuperPellet {
    pub col: i32,
    pub row: i32,
    pub active: bool,
}

pub struct ChaseSimulation {
    pub maze: Maze,

    pub kc: MazeEntity,
    pub kc_mouth: f32,

    pub ghost: MazeEntity,
    pub ghost_config: GhostConfig,

    pub super_pellets: Vec<SuperPellet>,

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
            super_pellets: Vec::new(),
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

        // 3. Find all available path tiles that are strictly reachable from center
        let mid_c = self.maze.cols as i32 / 2;
        let mid_r = self.maze.rows as i32 / 2;

        let mut path_tiles = Vec::new();
        for r in 1..self.maze.rows - 1 {
            for c in 1..self.maze.cols - 1 {
                if self.maze.tiles[r * self.maze.cols + c] == Tile::Path {
                    let tc = c as i32;
                    let tr = r as i32;
                    if (tc == mid_c && tr == mid_r)
                        || self.maze.bfs_next_direction(tc, tr, mid_c, mid_r).is_some()
                    {
                        path_tiles.push((tc, tr));
                    }
                }
            }
        }

        if path_tiles.len() < 4 {
            path_tiles.push((mid_c, mid_r));
            path_tiles.push((mid_c + 1, mid_r));
            path_tiles.push((mid_c - 1, mid_r));
            path_tiles.push((mid_c, mid_r + 1));
        }

        // 4. Place 4 Super Pellets, one in each corner quadrant (Top-Left, Top-Right, Bottom-Left, Bottom-Right)
        let corner_targets: [(i32, i32, fn(i32, i32, i32, i32) -> bool); 4] = [
            (1, 1, |c, r, mc, mr| c <= mc && r <= mr),
            (self.maze.cols as i32 - 2, 1, |c, r, mc, mr| c >= mc && r <= mr),
            (1, self.maze.rows as i32 - 2, |c, r, mc, mr| c <= mc && r >= mr),
            (self.maze.cols as i32 - 2, self.maze.rows as i32 - 2, |c, r, mc, mr| c >= mc && r >= mr),
        ];

        let mut chosen_corners = Vec::new();
        for &(tc, tr, in_quadrant) in &corner_targets {
            let mut best = None;
            let mut min_dist = i32::MAX;
            for &(pc, pr) in &path_tiles {
                if chosen_corners.contains(&(pc, pr)) {
                    continue;
                }
                if in_quadrant(pc, pr, mid_c, mid_r) {
                    let dist = (pc - tc).pow(2) + (pr - tr).pow(2);
                    if dist < min_dist {
                        min_dist = dist;
                        best = Some((pc, pr));
                    }
                }
            }

            // Fallback if no unchosen tile in quadrant
            if best.is_none() {
                for &(pc, pr) in &path_tiles {
                    if chosen_corners.contains(&(pc, pr)) {
                        continue;
                    }
                    let dist = (pc - tc).pow(2) + (pr - tr).pow(2);
                    if dist < min_dist {
                        min_dist = dist;
                        best = Some((pc, pr));
                    }
                }
            }

            if let Some(tile) = best {
                chosen_corners.push(tile);
            }
        }

        self.super_pellets = chosen_corners
            .into_iter()
            .map(|(c, r)| SuperPellet {
                col: c,
                row: r,
                active: true,
            })
            .collect();

        // 5. Place KC on a path tile away from the Super Pellets
        let kc_candidates: Vec<(i32, i32)> = path_tiles
            .iter()
            .copied()
            .filter(|&(c, r)| !self.super_pellets.iter().any(|sp| sp.col == c && sp.row == r))
            .collect();

        let (kc_c, kc_r) = if !kc_candidates.is_empty() {
            kc_candidates[rng.random_range(0..kc_candidates.len())]
        } else {
            (mid_c, mid_r)
        };

        // Find ghost tile with maximum distance from KC that is GUARANTEED to have a path to KC!
        let mut best_ghost_tile = path_tiles[0];
        let mut max_dist = -1.0;
        for &(gc, gr) in &path_tiles {
            if gc == kc_c && gr == kc_r {
                continue;
            }
            if self.maze.bfs_next_direction(gc, gr, kc_c, kc_r).is_some() {
                let dc = gc - kc_c;
                let dr = gr - kc_r;
                let dist = (dc * dc + dr * dr) as f32;
                if dist > max_dist {
                    max_dist = dist;
                    best_ghost_tile = (gc, gr);
                }
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
            if !self.super_pellets.iter().any(|sp| sp.col == pc && sp.row == pr) && !(pc == kc_c && pr == kc_r) {
                // Sparsely place pellets along paths
                if rng.random_bool(0.75) {
                    self.pellets.push((pc, pr, true));
                }
            }
        }

        self.phase = Phase::ApproachingPellet;
    }

    /// Find the nearest active super pellet from (from_c, from_r)
    pub fn nearest_active_super_pellet(&self, from_c: i32, from_r: i32) -> Option<(usize, i32, i32)> {
        let mut best = None;
        let mut min_dist = i32::MAX;

        for (idx, sp) in self.super_pellets.iter().enumerate() {
            if !sp.active {
                continue;
            }
            if sp.col == from_c && sp.row == from_r {
                return Some((idx, sp.col, sp.row));
            }
            let dist = (sp.col - from_c).abs() + (sp.row - from_r).abs();
            if dist < min_dist {
                min_dist = dist;
                best = Some((idx, sp.col, sp.row));
            }
        }

        best
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
                    // Check if KC reached any active Super Pellet
                    if let Some(idx) = self.super_pellets.iter().position(|sp| sp.active && sp.col == self.kc.col && sp.row == self.kc.row) {
                        self.super_pellets[idx].active = false;
                        let (spx, spy) = self
                            .maze
                            .tile_to_pixel(self.kc.col as f32, self.kc.row as f32);
                        self.spawn_shockwave(spx, spy);
                        self.spawn_sparkles(spx, spy, Rgb::new(255, 255, 100), 16);
                        next_phase = Phase::PelletChomp { timer: 0.6 };
                    } else if let Some((_, sp_c, sp_r)) = self.nearest_active_super_pellet(self.kc.col, self.kc.row) {
                        if let Some(next_dir) = self.maze.bfs_next_direction(
                            self.kc.col,
                            self.kc.row,
                            sp_c,
                            sp_r,
                        ) {
                            self.kc.set_next_direction(next_dir, &self.maze);
                        }
                    } else {
                        // All super pellets eaten! KC starts chasing
                        next_phase = Phase::KcChasing { duration: 9.0 };
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
                let mut new_duration = duration - dt;
                self.kc.speed = base_speed * 1.25; // Supercharged speed!
                self.ghost.speed = base_speed * 0.95;

                // KC pursues Ghost using BFS shortest path
                if self.kc.advance(dt, &self.maze) {
                    // Check if KC runs over another active corner Super Pellet!
                    if let Some(idx) = self.super_pellets.iter().position(|sp| sp.active && sp.col == self.kc.col && sp.row == self.kc.row) {
                        self.super_pellets[idx].active = false;
                        let (spx, spy) = self
                            .maze
                            .tile_to_pixel(self.kc.col as f32, self.kc.row as f32);
                        self.spawn_shockwave(spx, spy);
                        self.spawn_sparkles(spx, spy, Rgb::new(255, 255, 100), 16);
                        new_duration = 9.0; // Refresh energizer duration!
                    }

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
                let dist = entity_distance(&self.kc, &self.ghost, &self.maze);
                let catch_threshold = self.character_radius() * 1.6;

                if dist < catch_threshold {
                    // KC CAUGHT THE GHOST!
                    self.spawn_shockwave(self.ghost.x, self.ghost.y);
                    self.spawn_sparkles(self.ghost.x, self.ghost.y, Rgb::new(100, 200, 255), 18);
                    next_phase = Phase::GhostDeath {
                        timer: 0.8,
                        max_timer: 0.8,
                        ghost_x: self.ghost.x,
                        ghost_y: self.ghost.y,
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
                    // Check if KC grabs a remaining corner Super Pellet while power is fading!
                    if let Some(idx) = self.super_pellets.iter().position(|sp| sp.active && sp.col == self.kc.col && sp.row == self.kc.row) {
                        self.super_pellets[idx].active = false;
                        let (spx, spy) = self
                            .maze
                            .tile_to_pixel(self.kc.col as f32, self.kc.row as f32);
                        self.spawn_shockwave(spx, spy);
                        self.spawn_sparkles(spx, spy, Rgb::new(255, 255, 100), 16);
                        next_phase = Phase::PelletChomp { timer: 0.5 };
                    } else if let Some(next_dir) = self.maze.bfs_next_direction(
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

                if matches!(next_phase, Phase::PelletChomp { .. }) {
                    // KC ate a super pellet, stay in PelletChomp!
                } else if new_duration <= 0.0 {
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
                    // Check if KC reaches one of the remaining corner Super Pellets while fleeing!
                    if let Some(idx) = self.super_pellets.iter().position(|sp| sp.active && sp.col == self.kc.col && sp.row == self.kc.row) {
                        self.super_pellets[idx].active = false;
                        let (spx, spy) = self
                            .maze
                            .tile_to_pixel(self.kc.col as f32, self.kc.row as f32);
                        self.spawn_shockwave(spx, spy);
                        self.spawn_sparkles(spx, spy, Rgb::new(255, 255, 100), 16);
                        next_phase = Phase::PelletChomp { timer: 0.5 };
                    } else {
                        let flee_dir = self.maze.flee_direction(
                            self.kc.col,
                            self.kc.row,
                            self.ghost.col,
                            self.ghost.row,
                            self.kc.dir,
                        );
                        self.kc.set_next_direction(flee_dir, &self.maze);
                    }
                }

                // Check if Ghost catches KC!
                let dist = entity_distance(&self.ghost, &self.kc, &self.maze);
                let catch_threshold = self.character_radius() * 1.6;

                if matches!(next_phase, Phase::PelletChomp { .. }) {
                    // KC ate an energizer super pellet and turned the tables on the ghost!
                } else if dist < catch_threshold {
                    // GHOST CAUGHT KC!
                    self.spawn_shockwave(self.kc.x, self.kc.y);
                    self.spawn_sparkles(self.kc.x, self.kc.y, Rgb::new(255, 60, 60), 20);
                    next_phase = Phase::GhostCatchKc { timer: 1.2 };
                } else if new_duration <= 0.0 {
                    // If chase goes on long without a catch, transition to new maze anyway
                    next_phase = Phase::GhostCatchKc { timer: 0.5 };
                } else {
                    next_phase = Phase::GhostChasing {
                        duration: new_duration,
                    };
                }
            }

            Phase::GhostDeath {
                timer,
                max_timer,
                ghost_x,
                ghost_y,
            } => {
                let new_timer = timer - dt;
                if new_timer <= 0.0 {
                    // Ghost death animation finished!
                    // Now KC faces the camera, smiles with teeth, quickly grows arms & legs, and pumps to cheer!
                    self.spawn_shockwave(self.kc.x, self.kc.y);
                    self.spawn_sparkles(self.kc.x, self.kc.y, Rgb::new(255, 230, 40), 16);
                    next_phase = Phase::KcCheer {
                        timer: 2.2,
                        cheer_time: 0.0,
                    };
                } else {
                    next_phase = Phase::GhostDeath {
                        timer: new_timer,
                        max_timer,
                        ghost_x,
                        ghost_y,
                    };
                }
            }

            Phase::KcCheer { timer, cheer_time } => {
                let new_timer = timer - dt;
                let new_cheer_time = cheer_time + dt;
                if new_timer <= 0.0 {
                    // Cheer finished! Refresh maze to a brand-new layout to prevent burn-in
                    self.reset_maze_pass();
                    return;
                } else {
                    next_phase = Phase::KcCheer {
                        timer: new_timer,
                        cheer_time: new_cheer_time,
                    };
                }
            }

            Phase::GhostCatchKc { timer } => {
                let new_timer = timer - dt;
                if new_timer <= 0.0 {
                    self.reset_maze_pass();
                    return;
                } else {
                    next_phase = Phase::GhostCatchKc { timer: new_timer };
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

        // 3. Draw Super Pellets if active
        let pellet_rad = (self.character_radius() * 0.7).max(2.2);
        for sp in &self.super_pellets {
            if sp.active {
                let (spx, spy) = self
                    .maze
                    .tile_to_pixel(sp.col as f32, sp.row as f32);
                draw_super_pellet(canvas, spx, spy, pellet_rad, self.anim_time);
            }
        }

        // 4. Draw active particles
        for p in &self.particles {
            let alpha = (p.life / p.max_life).clamp(0.0, 1.0);
            let col = p.color.lerp(Rgb::BLACK, 1.0 - alpha);
            canvas.fill_circle_smooth(p.x, p.y, p.size, col);
        }

        let radius = self.character_radius();

        // 5. Draw KC Munchkin
        match self.phase {
            Phase::KcCheer { cheer_time, .. } => {
                // KC faces the camera, with a cute teeth-showing smile,
                // quickly grows arms and legs at 45 degree angles, and pumps them to cheer!
                let growth = (cheer_time / 0.28).min(1.0);
                draw_kc_cheering(
                    canvas,
                    self.kc.x,
                    self.kc.y,
                    radius,
                    growth,
                    cheer_time,
                    self.anim_time,
                );
            }
            Phase::GhostCatchKc { timer } => {
                if timer > 0.4 {
                    draw_kc_munchkin(
                        canvas,
                        self.kc.x,
                        self.kc.y,
                        radius,
                        self.kc.dir.to_angle(),
                        self.kc_mouth,
                        false,
                        true,
                        self.anim_time,
                    );
                }
            }
            _ => {
                let is_supercharged =
                    matches!(self.phase, Phase::KcChasing { .. } | Phase::PelletChomp { .. });
                let is_terrified = matches!(self.phase, Phase::GhostChasing { .. });
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
        }

        // 6. Draw Ghost
        match self.phase {
            Phase::GhostDeath {
                timer,
                max_timer,
                ghost_x,
                ghost_y,
            } => {
                let progress = (1.0 - (timer / max_timer)).clamp(0.0, 1.0);
                draw_ghost_death(
                    canvas,
                    ghost_x,
                    ghost_y,
                    radius,
                    progress,
                    &self.ghost_config,
                    self.anim_time,
                );
            }
            Phase::KcCheer { .. } => {
                // Ghost has dissolved after its death animation; KC is celebrating!
            }
            _ => {
                let ghost_state = match self.phase {
                    Phase::ApproachingPellet => GhostState::Patrolling,
                    Phase::PelletChomp { .. } | Phase::KcChasing { .. } => GhostState::Vulnerable,
                    Phase::PowerFading { duration } => GhostState::Fading {
                        time_remaining: duration,
                    },
                    Phase::GhostChasing { .. } | Phase::GhostCatchKc { .. } => GhostState::Hunting,
                    _ => GhostState::Patrolling,
                };
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

fn entity_distance(e1: &MazeEntity, e2: &MazeEntity, maze: &Maze) -> f32 {
    let canvas_w = maze.cols as f32 * maze.cell_w;
    let warp_r = maze.warp_row.unwrap_or(maze.rows / 2) as i32;

    let mut dx = (e1.x - e2.x).abs();
    if e1.row == warp_r && e2.row == warp_r {
        if dx > canvas_w * 0.5 {
            dx = canvas_w - dx;
        }
    }
    let dy = e1.y - e2.y;
    (dx * dx + dy * dy).sqrt()
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

    #[test]
    fn test_warp_tunnel_left_to_right() {
        let maze = Maze::generate(100, 50);
        let warp_r = maze.warp_row.unwrap() as i32;

        let mut entity = MazeEntity::new(0, warp_r, Direction::Left, 100.0, &maze);
        entity.set_next_direction(Direction::Left, &maze);

        assert_eq!(entity.target_col, -1);

        // Advance through warp tunnel
        let mut warped = false;
        for _ in 0..50 {
            if entity.advance(0.02, &maze) {
                warped = true;
                break;
            }
        }

        // Entity should have successfully emerged at cols - 1 on the right side!
        assert!(warped);
        assert_eq!(entity.col, maze.cols as i32 - 1);
        assert_eq!(entity.row, warp_r);
        assert_eq!(entity.dir, Direction::Left);
    }

    #[test]
    fn test_warp_tunnel_right_to_left() {
        let maze = Maze::generate(100, 50);
        let warp_r = maze.warp_row.unwrap() as i32;
        let right_col = maze.cols as i32 - 1;

        let mut entity = MazeEntity::new(right_col, warp_r, Direction::Right, 100.0, &maze);
        entity.set_next_direction(Direction::Right, &maze);

        assert_eq!(entity.target_col, maze.cols as i32);

        // Advance through warp tunnel
        let mut warped = false;
        for _ in 0..50 {
            if entity.advance(0.02, &maze) {
                warped = true;
                break;
            }
        }

        // Entity should have successfully emerged at col 0 on the left side!
        assert!(warped);
        assert_eq!(entity.col, 0);
        assert_eq!(entity.row, warp_r);
        assert_eq!(entity.dir, Direction::Right);
    }

    #[test]
    fn test_entities_never_cross_walls() {
        let mut sim = ChaseSimulation::new(120, 60);

        for _ in 0..1000 {
            sim.update(0.03);

            // Check KC center tile
            let kc_col = ((sim.kc.x - sim.maze.offset_x) / sim.maze.cell_w).floor() as i32;
            let kc_row = ((sim.kc.y - sim.maze.offset_y) / sim.maze.cell_h).floor() as i32;

            if !sim.kc.warping && kc_col >= 0 && kc_col < sim.maze.cols as i32 && kc_row >= 0 && kc_row < sim.maze.rows as i32 {
                assert_eq!(
                    sim.maze.tiles[(kc_row as usize) * sim.maze.cols + (kc_col as usize)],
                    Tile::Path,
                    "KC clipped into wall at tile ({}, {})!",
                    kc_col,
                    kc_row
                );
            }

            // Check Ghost center tile
            let ghost_col = ((sim.ghost.x - sim.maze.offset_x) / sim.maze.cell_w).floor() as i32;
            let ghost_row = ((sim.ghost.y - sim.maze.offset_y) / sim.maze.cell_h).floor() as i32;

            if !sim.ghost.warping && ghost_col >= 0 && ghost_col < sim.maze.cols as i32 && ghost_row >= 0 && ghost_row < sim.maze.rows as i32 {
                assert_eq!(
                    sim.maze.tiles[(ghost_row as usize) * sim.maze.cols + (ghost_col as usize)],
                    Tile::Path,
                    "Ghost clipped into wall at tile ({}, {})!",
                    ghost_col,
                    ghost_row
                );
            }
        }
    }

    #[test]
    fn test_kc_catch_sequence_ghost_death_then_cheer() {
        let mut sim = ChaseSimulation::new(100, 50);

        // Put simulation into KcChasing mode
        sim.phase = Phase::KcChasing { duration: 10.0 };

        // Position KC right next to Ghost so KC catches Ghost
        sim.kc.col = sim.ghost.col;
        sim.kc.row = sim.ghost.row;
        sim.kc.x = sim.ghost.x;
        sim.kc.y = sim.ghost.y;

        // Step one frame: catch should trigger GhostDeath!
        sim.update(0.02);
        assert!(
            matches!(sim.phase, Phase::GhostDeath { .. }),
            "Expected GhostDeath phase after catch, got {:?}",
            sim.phase
        );

        // Advance through the ghost death animation (~0.8s)
        for _ in 0..45 {
            sim.update(0.02);
        }

        // Now it must be in KcCheer phase!
        assert!(
            matches!(sim.phase, Phase::KcCheer { .. }),
            "Expected KcCheer phase after GhostDeath, got {:?}",
            sim.phase
        );

        // Advance through cheer phase (~2.2s)
        let initial_pass = sim.pass_count;
        for _ in 0..120 {
            sim.update(0.02);
        }

        // After cheer, maze should have refreshed to a brand new pass
        assert_eq!(
            sim.pass_count,
            initial_pass + 1,
            "Expected maze pass count to increment after cheer"
        );
    }

    #[test]
    fn test_50_mazes_always_fully_connected() {
        for _ in 0..50 {
            let sim = ChaseSimulation::new(120, 60);

            // Verify all 4 corner Super Pellets are reachable from KC
            assert_eq!(sim.super_pellets.len(), 4);
            for sp in &sim.super_pellets {
                let path_to_sp = sim.maze.bfs_next_direction(
                    sim.kc.col,
                    sim.kc.row,
                    sp.col,
                    sp.row,
                );
                // Either already at tile or reachable via BFS
                assert!(
                    (sim.kc.col == sp.col && sim.kc.row == sp.row) || path_to_sp.is_some(),
                    "KC at ({}, {}) cannot reach Super Pellet at ({}, {})!",
                    sim.kc.col,
                    sim.kc.row,
                    sp.col,
                    sp.row
                );
            }

            // Verify KC can reach Ghost
            let path_to_ghost = sim.maze.bfs_next_direction(
                sim.kc.col,
                sim.kc.row,
                sim.ghost.col,
                sim.ghost.row,
            );
            assert!(
                (sim.kc.col == sim.ghost.col && sim.kc.row == sim.ghost.row) || path_to_ghost.is_some(),
                "KC at ({}, {}) cannot reach Ghost at ({}, {})!",
                sim.kc.col,
                sim.kc.row,
                sim.ghost.col,
                sim.ghost.row
            );
        }
    }

    #[test]
    fn test_four_corner_super_pellets_and_re_energize() {
        let mut sim = ChaseSimulation::new(120, 60);

        // Verify exactly 4 super pellets
        assert_eq!(sim.super_pellets.len(), 4);
        assert!(sim.super_pellets.iter().all(|sp| sp.active));

        let mid_c = sim.maze.cols as i32 / 2;
        let mid_r = sim.maze.rows as i32 / 2;

        // Check each quadrant has one pellet
        let top_left = sim.super_pellets.iter().any(|sp| sp.col <= mid_c && sp.row <= mid_r);
        let top_right = sim.super_pellets.iter().any(|sp| sp.col >= mid_c && sp.row <= mid_r);
        let bot_left = sim.super_pellets.iter().any(|sp| sp.col <= mid_c && sp.row >= mid_r);
        let bot_right = sim.super_pellets.iter().any(|sp| sp.col >= mid_c && sp.row >= mid_r);
        assert!(top_left && top_right && bot_left && bot_right, "All four quadrants must have a corner pellet");

        // Test fleeing in GhostChasing and touching a corner super pellet turns the tables!
        sim.phase = Phase::GhostChasing { duration: 10.0 };
        let pellet_tile = (sim.super_pellets[0].col, sim.super_pellets[0].row);
        sim.kc.col = pellet_tile.0;
        sim.kc.row = pellet_tile.1;
        sim.kc.target_col = pellet_tile.0;
        sim.kc.target_row = pellet_tile.1;
        let (px, py) = sim.maze.tile_to_pixel(pellet_tile.0 as f32, pellet_tile.1 as f32);
        sim.kc.x = px;
        sim.kc.y = py;

        // Position Ghost far away so ghost catch doesn't trigger
        sim.ghost.col = sim.maze.cols as i32 - 2;
        sim.ghost.row = sim.maze.rows as i32 - 2;
        let (gx, gy) = sim.maze.tile_to_pixel(sim.ghost.col as f32, sim.ghost.row as f32);
        sim.ghost.x = gx;
        sim.ghost.y = gy;

        // Update one step
        sim.update(0.01);

        // Pellet 0 should now be inactive
        assert!(!sim.super_pellets[0].active);
        // Phase should have switched to PelletChomp
        assert!(matches!(sim.phase, Phase::PelletChomp { .. }));
    }
}
