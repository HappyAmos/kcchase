use crate::canvas::Canvas;
use crate::color::Rgb;
use crate::maze::{Direction, Maze, Tile};
use crate::sprites::{
    draw_ghost, draw_kc_cheering, draw_kc_munchkin, draw_pellet,
    draw_super_pellet, draw_text_3x5, GhostConfig, GhostState,
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
    #[allow(dead_code)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum StartSequence {
    #[default]
    Game,
    Intermission,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SequenceMode {
    #[default]
    Game,
    Intermission,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntermissionPhase {
    /// KC is supercharged, chasing 3 vulnerable ghosts across the screen (Left to Right)
    KcChasingGhosts,
    /// Ghosts are hunting in full colors, chasing terrified KC across the screen (Right to Left)
    GhostsChasingKc,
}

fn pick_different_y(canvas_h: usize, prev_y: f32, radius: f32) -> f32 {
    let min_y = radius + 3.0;
    let max_y = (canvas_h as f32 - radius - 3.0).max(min_y + 2.0);
    let span = max_y - min_y;
    if span <= 2.0 {
        return (min_y + max_y) * 0.5;
    }

    let mut rng = rand::rng();
    for _ in 0..12 {
        let candidate = min_y + rng.random_range(0.0..=1.0) * span;
        if (candidate - prev_y).abs() >= span * 0.22 {
            return candidate;
        }
    }
    // Deterministic fallback if random picks were too close
    if prev_y < min_y + span * 0.5 {
        min_y + span * 0.75
    } else {
        min_y + span * 0.25
    }
}

#[derive(Debug, Clone)]
pub struct IntermissionState {
    pub phase: IntermissionPhase,
    pub cycle: u32,
    pub target_cycles: u32,
    pub current_y: f32,
    pub progress_x: f32,
    pub speed: f32,
    pub finished: bool,
}

impl IntermissionState {
    pub fn new(canvas_w: usize, canvas_h: usize, target_cycles: u32) -> Self {
        let radius = (canvas_h as f32 * 0.08).clamp(6.0, 16.0);
        let speed = (canvas_w as f32 * 0.40).clamp(45.0, 95.0);
        let current_y = pick_different_y(canvas_h, -100.0, radius);
        let progress_x = -radius - 2.0;

        IntermissionState {
            phase: IntermissionPhase::KcChasingGhosts,
            cycle: 0,
            target_cycles: target_cycles.max(6),
            current_y,
            progress_x,
            speed,
            finished: false,
        }
    }

    pub fn reset(&mut self, canvas_w: usize, canvas_h: usize) {
        let radius = (canvas_h as f32 * 0.08).clamp(6.0, 16.0);
        self.phase = IntermissionPhase::KcChasingGhosts;
        self.cycle = 0;
        self.speed = (canvas_w as f32 * 0.40).clamp(45.0, 95.0);
        self.current_y = pick_different_y(canvas_h, self.current_y, radius);
        self.progress_x = -radius - 2.0;
        self.finished = false;
    }

    pub fn update(&mut self, dt: f32, canvas_w: usize, canvas_h: usize, character_radius: f32) {
        if self.finished {
            return;
        }

        let radius = (character_radius * 1.6).clamp(5.5, 18.0);
        let d = radius * 2.8;
        let train_len = 3.6 * d;

        match self.phase {
            IntermissionPhase::KcChasingGhosts => {
                // Moving Left to Right across screen
                self.progress_x += self.speed * dt;

                // KC is the rear entity at (progress_x - train_len)
                let kc_x = self.progress_x - train_len;
                if kc_x > canvas_w as f32 + radius + 4.0 {
                    // Everyone has exited the right side!
                    // Return from the SAME (right) side, running away from ghosts!
                    self.phase = IntermissionPhase::GhostsChasingKc;
                    self.current_y = pick_different_y(canvas_h, self.current_y, radius);
                    self.progress_x = canvas_w as f32 + radius + 2.0;
                }
            }
            IntermissionPhase::GhostsChasingKc => {
                // Moving Right to Left across screen
                self.progress_x -= self.speed * dt;

                // Rear-most ghost is Ghost 2 at (progress_x + train_len)
                let last_ghost_x = self.progress_x + train_len;
                if last_ghost_x < -radius - 4.0 {
                    // Everyone has exited the left side!
                    self.cycle += 1;
                    if self.cycle >= self.target_cycles {
                        self.finished = true;
                    } else {
                        // Return from the SAME (left) side, chasing ghosts!
                        self.phase = IntermissionPhase::KcChasingGhosts;
                        self.current_y = pick_different_y(canvas_h, self.current_y, radius);
                        self.progress_x = -radius - 2.0;
                    }
                }
            }
        }
    }

    pub fn draw(&self, canvas: &mut Canvas, anim_time: f32, character_radius: f32) {
        let radius = (character_radius * 1.6).clamp(5.5, 18.0);
        let d = radius * 2.8;

        let ghost_configs = [
            GhostConfig::red_bit(),
            GhostConfig::green_middie(),
            GhostConfig::yellow_byte(),
        ];

        match self.phase {
            IntermissionPhase::KcChasingGhosts => {
                // Moving Left to Right:
                // Lead is Ghost 2 (Yellow), followed by Ghost 1 (Green), Ghost 0 (Red), then KC
                let g2_x = self.progress_x;
                let g1_x = self.progress_x - d;
                let g0_x = self.progress_x - 2.0 * d;
                let kc_x = self.progress_x - 3.6 * d;

                // Ghosts in vulnerable scared blue state running to the right
                draw_ghost(canvas, g0_x, self.current_y, radius, 0.0, GhostState::Vulnerable, &ghost_configs[0], anim_time);
                draw_ghost(canvas, g1_x, self.current_y, radius, 0.0, GhostState::Vulnerable, &ghost_configs[1], anim_time);
                draw_ghost(canvas, g2_x, self.current_y, radius, 0.0, GhostState::Vulnerable, &ghost_configs[2], anim_time);

                // KC in supercharged mode with magenta aura chasing to the right
                let mouth_open = ((anim_time * 18.0).sin() + 1.0) * 0.5;
                draw_kc_munchkin(canvas, kc_x, self.current_y, radius, 0.0, mouth_open, true, false, anim_time);
            }
            IntermissionPhase::GhostsChasingKc => {
                // Moving Right to Left:
                // Lead is KC running to the left, followed by Ghost 0 (Red), Ghost 1 (Green), Ghost 2 (Yellow)
                let kc_x = self.progress_x;
                let g0_x = self.progress_x + 1.6 * d;
                let g1_x = self.progress_x + 2.6 * d;
                let g2_x = self.progress_x + 3.6 * d;

                // KC running terrified to the left
                let mouth_open = ((anim_time * 22.0).sin() + 1.0) * 0.5;
                draw_kc_munchkin(canvas, kc_x, self.current_y, radius, PI, mouth_open, false, true, anim_time);

                // Ghosts in hunting mode with distinct colors chasing to the left
                draw_ghost(canvas, g0_x, self.current_y, radius, PI, GhostState::Hunting, &ghost_configs[0], anim_time);
                draw_ghost(canvas, g1_x, self.current_y, radius, PI, GhostState::Hunting, &ghost_configs[1], anim_time);
                draw_ghost(canvas, g2_x, self.current_y, radius, PI, GhostState::Hunting, &ghost_configs[2], anim_time);
            }
        }
    }

    pub fn resize(&mut self, canvas_w: usize, canvas_h: usize) {
        self.speed = (canvas_w as f32 * 0.40).clamp(45.0, 95.0);
        let max_y = (canvas_h as f32 - 10.0).max(10.0);
        self.current_y = self.current_y.clamp(10.0, max_y);
    }
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

#[derive(Debug, Clone)]
pub struct SuperPellet {
    pub col: i32,
    pub row: i32,
    pub active: bool,
    pub entity: MazeEntity,
}

impl PartialEq for SuperPellet {
    fn eq(&self, other: &Self) -> bool {
        self.col == other.col && self.row == other.row && self.active == other.active
    }
}

#[derive(Debug, Clone)]
pub struct NormalPellet {
    pub col: i32,
    pub row: i32,
    pub active: bool,
    pub entity: MazeEntity,
}

impl PartialEq for NormalPellet {
    fn eq(&self, other: &Self) -> bool {
        self.col == other.col && self.row == other.row && self.active == other.active
    }
}

/// Advance a moving pill along corridors and pathfind at intersections or dead ends
fn update_pill_movement(
    entity: &mut MazeEntity,
    col: &mut i32,
    row: &mut i32,
    speed: f32,
    dt: f32,
    maze: &Maze,
) {
    entity.speed = speed;
    if entity.advance(dt, maze) {
        *col = entity.col;
        *row = entity.row;

        // Pathfind when reaching a tile center:
        // Query available directions from current tile, strictly avoiding the central ghost cage
        let available = maze.available_directions(entity.col, entity.row);
        let valid_dirs: Vec<Direction> = available
            .into_iter()
            .filter(|&d| {
                let nc = entity.col + d.dx();
                let nr = entity.row + d.dy();
                !maze.cage.is_cage_tile(nc, nr)
            })
            .collect();

        if valid_dirs.is_empty() {
            entity.set_next_direction(entity.dir, maze);
            return;
        }

        let forward_valid = valid_dirs.contains(&entity.dir);
        let forward_choices: Vec<Direction> = valid_dirs
            .iter()
            .copied()
            .filter(|&d| d != entity.dir.opposite())
            .collect();

        let mut rng = rand::rng();
        if !forward_valid {
            // Encountered a wall or dead end! Randomly choose from available paths to navigate
            let choice = if !forward_choices.is_empty() {
                forward_choices[rng.random_range(0..forward_choices.len())]
            } else {
                valid_dirs[rng.random_range(0..valid_dirs.len())]
            };
            entity.set_next_direction(choice, maze);
        } else if forward_choices.len() > 1 && rng.random_bool(0.35) {
            // At an open junction / intersection, occasionally choose a random path
            let choice = forward_choices[rng.random_range(0..forward_choices.len())];
            entity.set_next_direction(choice, maze);
        } else {
            // Continue along corridor in current direction
            entity.set_next_direction(entity.dir, maze);
        }
    }
}

/// Check collision between KC and a moving pill (either tile match or sub-pixel contact)
fn entity_collides_with_pill(kc: &MazeEntity, pill_entity: &MazeEntity, chomp_radius: f32) -> bool {
    if kc.col == pill_entity.col && kc.row == pill_entity.row {
        return true;
    }
    let dx = kc.x - pill_entity.x;
    let dy = kc.y - pill_entity.y;
    (dx * dx + dy * dy) < chomp_radius * chomp_radius
}

/// Determine target tile for KC pursuing a vulnerable ghost.
/// If KC has reached the ghost's tile, pursue towards the ghost's next destination so KC never freezes.
fn get_ghost_target_tile(kc_col: i32, kc_row: i32, vg: &GhostInstance, maze: &Maze) -> (i32, i32) {
    if kc_col == vg.entity.col && kc_row == vg.entity.row {
        if vg.entity.target_col != vg.entity.col || vg.entity.target_row != vg.entity.row {
            (vg.entity.target_col, vg.entity.target_row)
        } else {
            let nc = vg.entity.col + vg.entity.dir.dx();
            let nr = vg.entity.row + vg.entity.dir.dy();
            if maze.is_passable(nc, nr) {
                (nc, nr)
            } else {
                (vg.entity.col, vg.entity.row)
            }
        }
    } else {
        (vg.entity.col, vg.entity.row)
    }
}

#[derive(Debug, Clone)]
pub struct GhostInstance {
    pub entity: MazeEntity,
    pub config: GhostConfig,
    pub state: GhostState,
}

#[derive(Debug, Clone)]
pub struct ScorePopup {
    pub x: f32,
    pub y: f32,
    pub text: String,
    pub color: Rgb,
    pub life: f32,
}

pub struct ChaseSimulation {
    pub maze: Maze,

    pub kc: MazeEntity,
    pub kc_mouth: f32,

    pub ghosts: Vec<GhostInstance>,

    pub super_pellets: Vec<SuperPellet>,
    pub pellets: Vec<NormalPellet>,
    pub particles: Vec<Particle>,
    pub popups: Vec<ScorePopup>,

    pub score: u32,
    pub high_score: u32,
    pub ghosts_eaten_this_power: u32,

    pub phase: Phase,
    pub anim_time: f32,
    pub pass_count: usize,

    pub sequence_mode: SequenceMode,
    pub intermission: IntermissionState,
    pub levels_in_game_sequence: u32,

    pub canvas_w: usize,
    pub canvas_h: usize,
}

impl ChaseSimulation {
    #[allow(dead_code)]
    pub fn new(canvas_w: usize, canvas_h: usize) -> Self {
        Self::new_with_sequence(canvas_w, canvas_h, StartSequence::Game)
    }

    pub fn new_with_sequence(canvas_w: usize, canvas_h: usize, start_sequence: StartSequence) -> Self {
        let maze = Maze::generate(canvas_w, canvas_h);
        let intermission = IntermissionState::new(canvas_w, canvas_h, 6);
        let sequence_mode = match start_sequence {
            StartSequence::Game => SequenceMode::Game,
            StartSequence::Intermission => SequenceMode::Intermission,
        };

        let mut sim = ChaseSimulation {
            maze,
            kc: MazeEntity::new(1, 1, Direction::Right, 45.0, &Maze::generate(10, 10)),
            kc_mouth: 0.5,
            ghosts: Vec::new(),
            super_pellets: Vec::new(),
            pellets: Vec::new(),
            particles: Vec::new(),
            popups: Vec::new(),
            score: 0,
            high_score: 0,
            ghosts_eaten_this_power: 0,
            phase: Phase::ApproachingPellet,
            anim_time: 0.0,
            pass_count: 0,
            sequence_mode,
            intermission,
            levels_in_game_sequence: 0,
            canvas_w,
            canvas_h,
        };

        if sequence_mode == SequenceMode::Game {
            sim.reset_maze_pass();
        } else {
            sim.intermission.reset(canvas_w, canvas_h);
        }
        sim
    }

    /// Resize simulation arena when terminal size changes
    pub fn resize(&mut self, canvas_w: usize, canvas_h: usize) {
        self.canvas_w = canvas_w;
        self.canvas_h = canvas_h;
        self.intermission.resize(canvas_w, canvas_h);
        if self.sequence_mode == SequenceMode::Game {
            self.reset_maze_pass();
        }
    }

    /// Generate a brand new random maze layout with fresh walls, colors, pellet paths, and sizes!
    pub fn reset_maze_pass(&mut self) {
        self.pass_count += 1;
        let mut rng = rand::rng();

        // 1. Generate new random arcade maze
        self.maze = Maze::generate(self.canvas_w, self.canvas_h);

        // 2. Find all available path tiles that are strictly reachable from center
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

        // 4. Place exactly 1 Power Pill and 2 Regular Pills in each of the 4 corner quadrants (12 total pills)
        let corner_targets: [(i32, i32, fn(i32, i32, i32, i32) -> bool); 4] = [
            (1, 1, |c, r, mc, mr| c < mc && r < mr),
            (self.maze.cols as i32 - 2, 1, |c, r, mc, mr| c > mc && r < mr),
            (1, self.maze.rows as i32 - 2, |c, r, mc, mr| c < mc && r > mr),
            (self.maze.cols as i32 - 2, self.maze.rows as i32 - 2, |c, r, mc, mr| c > mc && r > mr),
        ];

        let mut chosen_super = Vec::new();
        let mut chosen_normal = Vec::new();

        for &(tc, tr, in_quadrant) in &corner_targets {
            // Find 1 power pill in this corner
            let mut best_power = None;
            let mut min_dist = i32::MAX;
            for &(pc, pr) in &path_tiles {
                if chosen_super.contains(&(pc, pr)) || chosen_normal.contains(&(pc, pr)) || self.maze.cage.is_cage_tile(pc, pr) {
                    continue;
                }
                if in_quadrant(pc, pr, mid_c, mid_r) {
                    let dist = (pc - tc).pow(2) + (pr - tr).pow(2);
                    if dist < min_dist {
                        min_dist = dist;
                        best_power = Some((pc, pr));
                    }
                }
            }

            // Fallback if needed
            if best_power.is_none() {
                for &(pc, pr) in &path_tiles {
                    if chosen_super.contains(&(pc, pr)) || chosen_normal.contains(&(pc, pr)) || self.maze.cage.is_cage_tile(pc, pr) {
                        continue;
                    }
                    let dist = (pc - tc).pow(2) + (pr - tr).pow(2);
                    if dist < min_dist {
                        min_dist = dist;
                        best_power = Some((pc, pr));
                    }
                }
            }

            if let Some(tile) = best_power {
                chosen_super.push(tile);
            }

            // Find exactly 2 regular pills in this same corner quadrant
            let mut quad_candidates: Vec<(i32, i32)> = path_tiles
                .iter()
                .copied()
                .filter(|&(pc, pr)| {
                    in_quadrant(pc, pr, mid_c, mid_r)
                        && !chosen_super.contains(&(pc, pr))
                        && !chosen_normal.contains(&(pc, pr))
                        && !self.maze.cage.is_cage_tile(pc, pr)
                })
                .collect();

            // Shuffle quadrant candidates for variety
            for i in (1..quad_candidates.len()).rev() {
                let j = rng.random_range(0..=i);
                quad_candidates.swap(i, j);
            }

            for &(pc, pr) in quad_candidates.iter().take(2) {
                chosen_normal.push((pc, pr));
            }
        }

        // Base speed scales naturally with corridor size
        let base_speed = (self.maze.cell_w.min(self.maze.cell_h) * 2.8).clamp(30.0, 70.0);

        self.super_pellets = chosen_super
            .iter()
            .map(|&(c, r)| {
                let initial_dir = Direction::ALL[rng.random_range(0..4)];
                SuperPellet {
                    col: c,
                    row: r,
                    active: true,
                    entity: MazeEntity::new(c, r, initial_dir, base_speed * 0.22, &self.maze),
                }
            })
            .collect();

        self.pellets = chosen_normal
            .iter()
            .map(|&(c, r)| {
                let initial_dir = Direction::ALL[rng.random_range(0..4)];
                NormalPellet {
                    col: c,
                    row: r,
                    active: true,
                    entity: MazeEntity::new(c, r, initial_dir, base_speed * 0.22, &self.maze),
                }
            })
            .collect();

        // 5. Place KC on a path tile away from the central cage and all pills
        let cage_c = self.maze.cage.center_c as i32;
        let cage_r = self.maze.cage.center_r as i32;
        let kc_candidates: Vec<(i32, i32)> = path_tiles
            .iter()
            .copied()
            .filter(|&(c, r)| {
                !self.maze.cage.is_cage_tile(c, r)
                    && !chosen_super.contains(&(c, r))
                    && !chosen_normal.contains(&(c, r))
                    && (c < cage_c - 2 || c > cage_c + 2 || r < cage_r - 2 || r > cage_r + 2)
            })
            .collect();

        let (kc_c, kc_r) = if !kc_candidates.is_empty() {
            kc_candidates[rng.random_range(0..kc_candidates.len())]
        } else {
            (1, 1)
        };

        // 6. Ghosts start INSIDE the central ghost cage, facing the open cage opening!
        let ghost_start_c = self.maze.cage.center_c as i32;
        let ghost_start_r = self.maze.cage.center_r as i32;

        let configs = if self.maze.cols < 9 || self.maze.rows < 7 {
            vec![GhostConfig::red_bit(), GhostConfig::green_middie()]
        } else {
            vec![
                GhostConfig::red_bit(),
                GhostConfig::green_middie(),
                GhostConfig::yellow_byte(),
            ]
        };

        self.ghosts = configs
            .into_iter()
            .enumerate()
            .map(|(idx, config)| {
                let state = if idx == 0 {
                    GhostState::Patrolling
                } else {
                    GhostState::InCage {
                        timer: idx as f32 * 1.5,
                    }
                };
                GhostInstance {
                    entity: MazeEntity::new(
                        ghost_start_c,
                        ghost_start_r,
                        self.maze.cage.door_dir,
                        base_speed,
                        &self.maze,
                    ),
                    config,
                    state,
                }
            })
            .collect();

        self.kc = MazeEntity::new(kc_c, kc_r, Direction::Right, base_speed, &self.maze);
        self.particles.clear();
        self.popups.clear();
        self.phase = Phase::ApproachingPellet;
    }

    /// Calculate dynamic pill speed based on how many pills remain in the maze.
    /// Pills start slow, gradually increase in speed as KC eats them, move pretty fast
    /// down to the last two pills, and the final pill moves about the same speed as the ghost.
    pub fn current_pill_speed(&self, base_speed: f32) -> f32 {
        let active_super = self.super_pellets.iter().filter(|sp| sp.active).count();
        let active_normal = self.pellets.iter().filter(|p| p.active).count();
        let remaining = active_super + active_normal;
        let total_pills = self.super_pellets.len() + self.pellets.len();

        let ghost_speed = base_speed;

        match remaining {
            0 => 0.0,
            1 => ghost_speed * 0.98, // Final pill: about the same speed as the ghost
            2 => ghost_speed * 0.85, // Last two pills: moving pretty fast (not faster than ghost/KC)
            _ => {
                // When >= 3 pills remain: very gradually increases from slow (0.22) up to 0.85
                let max_ref = (total_pills.saturating_sub(2)).max(1) as f32;
                let eaten_ratio = ((total_pills - remaining) as f32 / max_ref).clamp(0.0, 1.0);
                ghost_speed * (0.22 + eaten_ratio * (0.85 - 0.22))
            }
        }
    }

    /// Find the nearest active pill (normal or super) from (from_c, from_r)
    pub fn nearest_active_pill(&self, from_c: i32, from_r: i32) -> Option<(i32, i32, bool)> {
        let mut best = None;
        let mut min_dist = i32::MAX;

        for sp in &self.super_pellets {
            if !sp.active {
                continue;
            }
            let dist = (sp.col - from_c).abs() + (sp.row - from_r).abs();
            if dist < min_dist {
                min_dist = dist;
                best = Some((sp.col, sp.row, true));
            }
        }

        for p in &self.pellets {
            if !p.active {
                continue;
            }
            let dist = (p.col - from_c).abs() + (p.row - from_r).abs();
            if dist < min_dist {
                min_dist = dist;
                best = Some((p.col, p.row, false));
            }
        }

        best
    }

    /// Find the nearest active super pellet from (from_c, from_r)
    #[allow(dead_code)]
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

        match self.sequence_mode {
            SequenceMode::Intermission => {
                self.intermission.update(dt, self.canvas_w, self.canvas_h, self.character_radius());
                if self.intermission.finished {
                    self.sequence_mode = SequenceMode::Game;
                    self.levels_in_game_sequence = 0;
                    self.reset_maze_pass();
                }
                return;
            }
            SequenceMode::Game => {}
        }

        // If KC died or is cheering victory, entities freeze while animation plays out,
        // then the level resets cleanly.
        if let Phase::GhostCatchKc { timer } = self.phase {
            self.kc_mouth = ((self.anim_time * 8.0).sin() + 1.0) * 0.5;
            for p in self.particles.iter_mut() {
                p.x += p.vx * dt;
                p.y += p.vy * dt;
                p.life -= dt;
            }
            self.particles.retain(|p| p.life > 0.0);
            for pop in self.popups.iter_mut() {
                pop.y -= 14.0 * dt;
                pop.life -= dt;
            }
            self.popups.retain(|pop| pop.life > 0.0);

            let new_timer = timer - dt;
            if new_timer <= 0.0 {
                if self.score > self.high_score {
                    self.high_score = self.score;
                }
                self.score = 0;
                self.ghosts_eaten_this_power = 0;
                self.levels_in_game_sequence += 1;

                if self.levels_in_game_sequence >= 2 {
                    // After at least 2 levels and KC is not winning: switch to intermission sequence!
                    self.sequence_mode = SequenceMode::Intermission;
                    self.intermission.reset(self.canvas_w, self.canvas_h);
                } else {
                    self.reset_maze_pass();
                }
            } else {
                self.phase = Phase::GhostCatchKc { timer: new_timer };
            }
            return;
        }

        if let Phase::KcCheer { timer, cheer_time } = self.phase {
            self.kc_mouth = ((self.anim_time * 8.0).sin() + 1.0) * 0.5;
            for p in self.particles.iter_mut() {
                p.x += p.vx * dt;
                p.y += p.vy * dt;
                p.life -= dt;
            }
            self.particles.retain(|p| p.life > 0.0);
            for pop in self.popups.iter_mut() {
                pop.y -= 14.0 * dt;
                pop.life -= dt;
            }
            self.popups.retain(|pop| pop.life > 0.0);

            let new_timer = timer - dt;
            let new_cheer_time = cheer_time + dt;
            if new_timer <= 0.0 {
                if self.score > self.high_score {
                    self.high_score = self.score;
                }
                self.ghosts_eaten_this_power = 0;
                self.levels_in_game_sequence += 1;

                // KC won! Since KC is winning, continue advancing to the next level
                self.reset_maze_pass();
            } else {
                self.phase = Phase::KcCheer {
                    timer: new_timer,
                    cheer_time: new_cheer_time,
                };
            }
            return;
        }

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

        // Update active score popups (float upward and fade)
        for pop in self.popups.iter_mut() {
            pop.y -= 14.0 * dt;
            pop.life -= dt;
        }
        self.popups.retain(|pop| pop.life > 0.0);

        let base_speed = (self.maze.cell_w.min(self.maze.cell_h) * 2.8).clamp(30.0, 70.0);
        // The speed of the ghost should never change
        let ghost_speed = base_speed;

        // Update central ghost cage rotating door (rotates clockwise once every 1.5 seconds)
        let mut blocked = vec![
            (self.kc.col, self.kc.row),
            (self.kc.target_col, self.kc.target_row),
        ];
        for g in &self.ghosts {
            blocked.push((g.entity.col, g.entity.row));
            blocked.push((g.entity.target_col, g.entity.target_row));
        }
        self.maze.update_cage_door(dt, &blocked);

        // Dynamically compute pill speed based on remaining pill count
        let pill_speed = self.current_pill_speed(base_speed);

        // Move all active super pills (pathfinding around the maze)
        for sp in self.super_pellets.iter_mut() {
            if sp.active {
                update_pill_movement(&mut sp.entity, &mut sp.col, &mut sp.row, pill_speed, dt, &self.maze);
            }
        }

        // Move all active normal pills (pathfinding around the maze)
        for p in self.pellets.iter_mut() {
            if p.active {
                update_pill_movement(&mut p.entity, &mut p.col, &mut p.row, pill_speed, dt, &self.maze);
            }
        }

        let chomp_radius = self.character_radius() * 1.1;

        // Chomp normal pellets along KC's path / collisions (1 point each)
        let mut eaten_sparkles = Vec::new();
        for p in self.pellets.iter_mut() {
            if p.active && entity_collides_with_pill(&self.kc, &p.entity, chomp_radius) {
                p.active = false;
                self.score += 1;
                eaten_sparkles.push((p.entity.x, p.entity.y));
            }
        }
        for (px, py) in eaten_sparkles {
            self.spawn_sparkles(px, py, Rgb::new(255, 230, 80), 4);
        }

        // Check if KC reaches any active Super Pellet (3 points each)
        let mut eaten_super_pos = Vec::new();
        for sp in self.super_pellets.iter_mut() {
            if sp.active && entity_collides_with_pill(&self.kc, &sp.entity, chomp_radius) {
                sp.active = false;
                eaten_super_pos.push((sp.entity.x, sp.entity.y));
            }
        }

        let mut next_phase = self.phase;

        if !eaten_super_pos.is_empty() {
            for (sx, sy) in eaten_super_pos {
                self.score += 3;
                self.ghosts_eaten_this_power = 0; // Reset ghost multiplier!
                self.spawn_shockwave(sx, sy);
                self.spawn_sparkles(sx, sy, Rgb::new(255, 255, 100), 16);
                self.popups.push(ScorePopup {
                    x: sx,
                    y: sy,
                    text: "+3".to_string(),
                    color: Rgb::new(255, 255, 100),
                    life: 0.8,
                });
            }

            // Turn non-eyes/non-cage ghosts vulnerable
            for g in self.ghosts.iter_mut() {
                if !matches!(g.state, GhostState::Eyes | GhostState::InCage { .. }) {
                    g.state = GhostState::Vulnerable;
                }
            }
            next_phase = Phase::PelletChomp { timer: 0.4 };
        }

        // If all pills (both normal and super) have been eaten, celebrate victory!
        let remaining_pills = self.super_pellets.iter().filter(|sp| sp.active).count()
            + self.pellets.iter().filter(|p| p.active).count();
        if remaining_pills == 0 {
            self.phase = Phase::KcCheer {
                timer: 2.2,
                cheer_time: 0.0,
            };
            return;
        }

        // Check collision between KC and each ghost
        let catch_threshold = (self.character_radius() * 2.2).max(self.maze.cell_w.min(self.maze.cell_h) * 0.95);
        let is_enhanced = matches!(
            next_phase,
            Phase::KcChasing { .. } | Phase::PowerFading { .. } | Phase::PelletChomp { .. }
        );

        let mut eaten_ghost_events = Vec::new();
        let mut ghost_caught_kc = false;

        for ghost in self.ghosts.iter_mut() {
            match ghost.state {
                GhostState::Eyes | GhostState::InCage { .. } => {}
                GhostState::Vulnerable | GhostState::Fading { .. } => {
                    let dist = entity_distance(&self.kc, &ghost.entity, &self.maze);
                    let same_tile = self.kc.col == ghost.entity.col && self.kc.row == ghost.entity.row;
                    if dist < catch_threshold || same_tile {
                        ghost.state = GhostState::Eyes;
                        eaten_ghost_events.push((ghost.entity.x, ghost.entity.y));
                    }
                }
                GhostState::Patrolling | GhostState::Hunting => {
                    let dist = entity_distance(&self.kc, &ghost.entity, &self.maze);
                    let same_tile = self.kc.col == ghost.entity.col && self.kc.row == ghost.entity.row;
                    if dist < catch_threshold || same_tile {
                        ghost_caught_kc = true;
                        break;
                    }
                }
            }
        }

        for (gx, gy) in eaten_ghost_events {
            self.ghosts_eaten_this_power += 1;
            let points = match self.ghosts_eaten_this_power {
                1 => 5,
                2 => 10,
                _ => 20,
            };
            self.score += points;
            self.spawn_shockwave(gx, gy);
            self.spawn_sparkles(gx, gy, Rgb::new(100, 200, 255), 18);
            self.popups.push(ScorePopup {
                x: gx,
                y: gy,
                text: format!("+{}", points),
                color: Rgb::new(100, 240, 255),
                life: 1.0,
            });
        }

        if ghost_caught_kc {
            self.spawn_shockwave(self.kc.x, self.kc.y);
            self.spawn_sparkles(self.kc.x, self.kc.y, Rgb::new(255, 60, 60), 20);
            self.phase = Phase::GhostCatchKc { timer: 0.8 };
            return;
        }

        // Move and update ghosts
        let cage_c = self.maze.cage.center_c as i32;
        let cage_r = self.maze.cage.center_r as i32;

        for (idx, ghost) in self.ghosts.iter_mut().enumerate() {
            match ghost.state {
                GhostState::InCage { ref mut timer } => {
                    *timer -= dt;
                    if *timer <= 0.0 {
                        ghost.state = if is_enhanced {
                            GhostState::Vulnerable
                        } else {
                            GhostState::Patrolling
                        };
                        ghost.entity.dir = self.maze.cage.door_dir;
                        ghost.entity.target_col = ghost.entity.col;
                        ghost.entity.target_row = ghost.entity.row;
                    }
                }
                GhostState::Eyes => {
                    // Floating eyes return swiftly to base, cannot be intercepted, stopped, or eaten
                    ghost.entity.speed = ghost_speed * 1.75;
                    if ghost.entity.advance(dt, &self.maze) {
                        if ghost.entity.col == cage_c && ghost.entity.row == cage_r {
                            ghost.state = GhostState::InCage { timer: 2.0 };
                            let (px, py) = self.maze.tile_to_pixel(cage_c as f32, cage_r as f32);
                            ghost.entity.x = px;
                            ghost.entity.y = py;
                        } else if let Some(next_dir) = self.maze.bfs_next_direction(
                            ghost.entity.col,
                            ghost.entity.row,
                            cage_c,
                            cage_r,
                        ) {
                            ghost.entity.set_next_direction(next_dir, &self.maze);
                        } else {
                            let wander = ghost_wander_direction(&ghost.entity, &self.maze);
                            ghost.entity.set_next_direction(wander, &self.maze);
                        }
                    }
                }
                GhostState::Vulnerable | GhostState::Fading { .. } => {
                    ghost.entity.speed = ghost_speed;
                    if ghost.entity.advance(dt, &self.maze) {
                        let flee_dir = self.maze.flee_direction(
                            ghost.entity.col,
                            ghost.entity.row,
                            self.kc.col,
                            self.kc.row,
                            ghost.entity.dir,
                        );
                        ghost.entity.set_next_direction(flee_dir, &self.maze);
                    }
                }
                GhostState::Patrolling | GhostState::Hunting => {
                    ghost.entity.speed = ghost_speed;
                    if ghost.entity.advance(dt, &self.maze) {
                        if self.maze.cage.is_inside(ghost.entity.col, ghost.entity.row) {
                            ghost.entity.set_next_direction(self.maze.cage.door_dir, &self.maze);
                        } else if matches!(self.phase, Phase::GhostChasing { .. }) || idx == 0 {
                            if let Some(dir) = self.maze.bfs_next_direction(
                                ghost.entity.col,
                                ghost.entity.row,
                                self.kc.col,
                                self.kc.row,
                            ) {
                                ghost.entity.set_next_direction(dir, &self.maze);
                            } else {
                                let wander = ghost_wander_direction(&ghost.entity, &self.maze);
                                ghost.entity.set_next_direction(wander, &self.maze);
                            }
                        } else {
                            let wander = ghost_wander_direction(&ghost.entity, &self.maze);
                            ghost.entity.set_next_direction(wander, &self.maze);
                        }
                    }
                }
            }
        }

        // Phase progression & KC movement
        match next_phase {
            Phase::ApproachingPellet => {
                self.kc.speed = base_speed;
                if self.kc.advance(dt, &self.maze) {
                    if let Some((target_c, target_r, _)) = self.nearest_active_pill(self.kc.col, self.kc.row) {
                        if let Some(next_dir) = self.maze.bfs_next_direction(
                            self.kc.col,
                            self.kc.row,
                            target_c,
                            target_r,
                        ) {
                            self.kc.set_next_direction(next_dir, &self.maze);
                        }
                    }
                }
            }

            Phase::PelletChomp { timer } => {
                let new_timer = timer - dt;
                if new_timer <= 0.0 {
                    next_phase = Phase::KcChasing { duration: 9.0 };
                } else {
                    next_phase = Phase::PelletChomp { timer: new_timer };
                }
            }

            Phase::KcChasing { duration } => {
                let new_duration = duration - dt;
                // Noticeable speed boost under enhanced pill (25% faster than ghosts)
                self.kc.speed = ghost_speed * 1.25;

                if self.kc.advance(dt, &self.maze) {
                    // KC's first priority is always getting all the pills;
                    // getting extra points from ghosts is a secondary opportunistic goal.
                    let nearest_pill = self.nearest_active_pill(self.kc.col, self.kc.row)
                        .map(|(c, r, _)| (c, r));

                    let nearest_vulnerable = self.ghosts
                        .iter()
                        .filter(|g| matches!(g.state, GhostState::Vulnerable | GhostState::Fading { .. }))
                        .map(|g| (g, (g.entity.col - self.kc.col).abs() + (g.entity.row - self.kc.row).abs()))
                        .min_by_key(|&(_, dist)| dist);

                    let (target, pursued_ghost_dir) = match (nearest_pill, nearest_vulnerable) {
                        // Secondary goal: if a vulnerable ghost is closer than the nearest pill (or nearby <= 3 tiles),
                        // take the opportunistic detour to score extra points!
                        (Some((pc, pr)), Some((vg, g_dist)))
                            if g_dist <= 3 || (g_dist <= 6 && g_dist < (pc - self.kc.col).abs() + (pr - self.kc.row).abs()) =>
                        {
                            (Some(get_ghost_target_tile(self.kc.col, self.kc.row, vg, &self.maze)), Some(vg.entity.dir))
                        }
                        // First priority: always clear the pills!
                        (Some(pill), _) => (Some(pill), None),
                        // Fallback if no pills remain
                        (None, Some((vg, _))) => {
                            (Some(get_ghost_target_tile(self.kc.col, self.kc.row, vg, &self.maze)), Some(vg.entity.dir))
                        }
                        (None, None) => (None, None),
                    };

                    if let Some((tc, tr)) = target {
                        if let Some(next_dir) = self.maze.bfs_next_direction(
                            self.kc.col,
                            self.kc.row,
                            tc,
                            tr,
                        ) {
                            self.kc.set_next_direction(next_dir, &self.maze);
                        } else if let Some(g_dir) = pursued_ghost_dir {
                            // If already on the ghost's tile, charge forward in the ghost's direction without stalling
                            self.kc.set_next_direction(g_dir, &self.maze);
                        }
                    }
                }

                if new_duration <= 0.0 {
                    next_phase = Phase::PowerFading { duration: 3.5 };
                    for g in self.ghosts.iter_mut() {
                        if matches!(g.state, GhostState::Vulnerable) {
                            g.state = GhostState::Fading { time_remaining: 3.5 };
                        }
                    }
                } else {
                    next_phase = Phase::KcChasing { duration: new_duration };
                }
            }

            Phase::PowerFading { duration } => {
                let new_duration = duration - dt;
                // Power waning, still 15% faster than ghosts
                self.kc.speed = ghost_speed * 1.15;

                if self.kc.advance(dt, &self.maze) {
                    let nearest_pill = self.nearest_active_pill(self.kc.col, self.kc.row)
                        .map(|(c, r, _)| (c, r));

                    let nearest_vulnerable = self.ghosts
                        .iter()
                        .filter(|g| matches!(g.state, GhostState::Vulnerable | GhostState::Fading { .. }))
                        .map(|g| (g, (g.entity.col - self.kc.col).abs() + (g.entity.row - self.kc.row).abs()))
                        .min_by_key(|&(_, dist)| dist);

                    let (target, pursued_ghost_dir) = match (nearest_pill, nearest_vulnerable) {
                        // Secondary goal: if a vulnerable ghost is closer than the nearest pill (or nearby <= 3 tiles),
                        // take the opportunistic detour to score extra points!
                        (Some((pc, pr)), Some((vg, g_dist)))
                            if g_dist <= 3 || (g_dist <= 6 && g_dist < (pc - self.kc.col).abs() + (pr - self.kc.row).abs()) =>
                        {
                            (Some(get_ghost_target_tile(self.kc.col, self.kc.row, vg, &self.maze)), Some(vg.entity.dir))
                        }
                        // First priority: always clear the pills!
                        (Some(pill), _) => (Some(pill), None),
                        // Fallback if no pills remain
                        (None, Some((vg, _))) => {
                            (Some(get_ghost_target_tile(self.kc.col, self.kc.row, vg, &self.maze)), Some(vg.entity.dir))
                        }
                        (None, None) => (None, None),
                    };

                    if let Some((tc, tr)) = target {
                        if let Some(next_dir) = self.maze.bfs_next_direction(
                            self.kc.col,
                            self.kc.row,
                            tc,
                            tr,
                        ) {
                            self.kc.set_next_direction(next_dir, &self.maze);
                        } else if let Some(g_dir) = pursued_ghost_dir {
                            self.kc.set_next_direction(g_dir, &self.maze);
                        }
                    }
                }

                if new_duration <= 0.0 {
                    self.ghosts_eaten_this_power = 0;
                    for g in self.ghosts.iter_mut() {
                        if matches!(g.state, GhostState::Fading { .. }) {
                            g.state = GhostState::Hunting;
                        }
                    }
                    next_phase = Phase::GhostChasing { duration: 12.0 };
                } else {
                    next_phase = Phase::PowerFading { duration: new_duration };
                }
            }

            Phase::GhostChasing { duration } => {
                let new_duration = duration - dt;
                self.kc.speed = ghost_speed;

                if self.kc.advance(dt, &self.maze) {
                    // Check if a hunting ghost is close by (< 4 tiles); if so, flee!
                    let mut closest_ghost: Option<&MazeEntity> = None;
                    let mut min_d = i32::MAX;
                    for g in &self.ghosts {
                        if matches!(g.state, GhostState::Hunting | GhostState::Patrolling) {
                            let d = (g.entity.col - self.kc.col).abs() + (g.entity.row - self.kc.row).abs();
                            if d < min_d {
                                min_d = d;
                                closest_ghost = Some(&g.entity);
                            }
                        }
                    }

                    if let Some(g_ent) = closest_ghost {
                        if min_d < 4 {
                            let flee = self.maze.flee_direction(
                                self.kc.col,
                                self.kc.row,
                                g_ent.col,
                                g_ent.row,
                                self.kc.dir,
                            );
                            self.kc.set_next_direction(flee, &self.maze);
                        } else if let Some((target_c, target_r, _)) = self.nearest_active_pill(self.kc.col, self.kc.row) {
                            if let Some(next_dir) = self.maze.bfs_next_direction(
                                self.kc.col,
                                self.kc.row,
                                target_c,
                                target_r,
                            ) {
                                self.kc.set_next_direction(next_dir, &self.maze);
                            }
                        }
                    } else if let Some((target_c, target_r, _)) = self.nearest_active_pill(self.kc.col, self.kc.row) {
                        if let Some(next_dir) = self.maze.bfs_next_direction(
                            self.kc.col,
                            self.kc.row,
                            target_c,
                            target_r,
                        ) {
                            self.kc.set_next_direction(next_dir, &self.maze);
                        }
                    }
                }

                if new_duration <= 0.0 {
                    next_phase = Phase::ApproachingPellet;
                } else {
                    next_phase = Phase::GhostChasing { duration: new_duration };
                }
            }

            Phase::GhostDeath { timer, max_timer, ghost_x, ghost_y } => {
                let new_timer = timer - dt;
                if new_timer <= 0.0 {
                    next_phase = Phase::ApproachingPellet;
                } else {
                    next_phase = Phase::GhostDeath {
                        timer: new_timer,
                        max_timer,
                        ghost_x,
                        ghost_y,
                    };
                }
            }

            Phase::KcCheer { .. } | Phase::GhostCatchKc { .. } => {}
        }

        self.phase = next_phase;
    }

    /// Render the full screensaver frame onto canvas
    pub fn draw(&self, canvas: &mut Canvas) {
        canvas.clear();

        if self.sequence_mode == SequenceMode::Intermission {
            self.intermission.draw(canvas, self.anim_time, self.character_radius());
            return;
        }

        // 1. Draw procedural neon arcade maze walls
        self.maze.draw(canvas, self.anim_time);

        // 2. Draw normal moving pellets along corridors
        for p in &self.pellets {
            if p.active {
                draw_pellet(canvas, p.entity.x, p.entity.y, self.anim_time);
            }
        }

        // 3. Draw moving Super Pellets if active
        let pellet_rad = (self.character_radius() * 0.7).max(2.2);
        for sp in &self.super_pellets {
            if sp.active {
                draw_super_pellet(canvas, sp.entity.x, sp.entity.y, pellet_rad, self.anim_time);
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

        // 6. Draw Ghosts (paused at their current coordinates during victory cheer, but skirts/eyes still animate)
        for g in &self.ghosts {
            draw_ghost(
                canvas,
                g.entity.x,
                g.entity.y,
                radius,
                g.entity.dir.to_angle(),
                g.state,
                &g.config,
                self.anim_time,
            );
        }

        // 7. Draw floating Score Popups
        for pop in &self.popups {
            draw_text_3x5(
                canvas,
                (pop.x - 6.0) as i32,
                (pop.y - 3.0) as i32,
                &pop.text,
                pop.color,
            );
        }

        // 8. Draw Retro Arcade HUD (Score, High Score, Pills Remaining)
        let hud_y = 2;
        let hud_score = format!("SCORE:{:04}", self.score);
        let hud_high = format!("HIGH:{:04}", self.high_score.max(self.score));
        let rem_pills = self.super_pellets.iter().filter(|sp| sp.active).count()
            + self.pellets.iter().filter(|p| p.active).count();
        let hud_pills = format!("PILLS:{:02}", rem_pills);

        let canvas_w = canvas.width as i32;
        let score_x = 4;
        let pills_x = (canvas_w / 2) - ((hud_pills.len() as i32 * 4) / 2);
        let high_x = canvas_w - (hud_high.len() as i32 * 4) - 4;

        draw_text_3x5(canvas, score_x, hud_y, &hud_score, Rgb::new(255, 230, 80));
        draw_text_3x5(canvas, pills_x, hud_y, &hud_pills, Rgb::new(140, 255, 140));
        draw_text_3x5(canvas, high_x, hud_y, &hud_high, Rgb::new(80, 220, 255));
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

fn ghost_wander_direction(ghost: &MazeEntity, maze: &Maze) -> Direction {
    let available = maze.available_directions(ghost.col, ghost.row);
    if available.is_empty() {
        return ghost.dir;
    }

    // If outside the cage, prefer to explore outer maze corridors rather than stepping back inside
    let filtered: Vec<Direction> = if !maze.cage.is_inside(ghost.col, ghost.row) {
        let outside: Vec<Direction> = available
            .iter()
            .copied()
            .filter(|&d| {
                let nc = ghost.col + d.dx();
                let nr = ghost.row + d.dy();
                !maze.cage.is_inside(nc, nr)
            })
            .collect();
        if !outside.is_empty() {
            outside
        } else {
            available.clone()
        }
    } else {
        available.clone()
    };

    // Avoid turning directly backwards unless in dead end
    let forward_choices: Vec<Direction> = filtered
        .iter()
        .copied()
        .filter(|&d| d != ghost.dir.opposite())
        .collect();

    let mut rng = rand::rng();
    if !forward_choices.is_empty() {
        forward_choices[rng.random_range(0..forward_choices.len())]
    } else {
        filtered[rng.random_range(0..filtered.len())]
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

            // Check Ghost center tiles
            for g in &sim.ghosts {
                let ghost_col = ((g.entity.x - sim.maze.offset_x) / sim.maze.cell_w).floor() as i32;
                let ghost_row = ((g.entity.y - sim.maze.offset_y) / sim.maze.cell_h).floor() as i32;

                if !g.entity.warping && ghost_col >= 0 && ghost_col < sim.maze.cols as i32 && ghost_row >= 0 && ghost_row < sim.maze.rows as i32 {
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
    }

    #[test]
    fn test_ghost_eaten_turns_to_eyes_and_kc_cheers_on_last_pill() {
        let mut sim = ChaseSimulation::new(100, 50);

        // Put simulation into KcChasing mode
        sim.phase = Phase::KcChasing { duration: 10.0 };
        for g in sim.ghosts.iter_mut() {
            g.state = GhostState::Vulnerable;
        }

        // Position KC and Ghost 0 away from the cage on the perimeter ring where no pills are placed
        let (away_c, away_r) = (sim.maze.cage.center_c as i32, sim.maze.cage.center_r as i32 - 2);
        let (ax, ay) = sim.maze.tile_to_pixel(away_c as f32, away_r as f32);
        sim.ghosts[0].entity.col = away_c;
        sim.ghosts[0].entity.row = away_r;
        sim.ghosts[0].entity.x = ax;
        sim.ghosts[0].entity.y = ay;

        sim.kc.col = away_c;
        sim.kc.row = away_r;
        sim.kc.x = ax;
        sim.kc.y = ay;

        // Step one frame: catch should NOT stop KC or enter GhostDeath/KcCheer!
        sim.update(0.02);
        assert_eq!(
            sim.ghosts[0].state,
            GhostState::Eyes,
            "Ghost should become Eyes when eaten"
        );
        assert!(
            matches!(sim.phase, Phase::KcChasing { .. }),
            "KC should continue munching as normal, phase remains KcChasing"
        );
        assert_eq!(sim.score, 5, "First ghost eaten should award 5 points");

        // Clear all remaining pills so KC beats the level!
        for p in sim.pellets.iter_mut() {
            p.active = false;
        }
        for sp in sim.super_pellets.iter_mut() {
            sp.active = false;
        }

        // Step simulation: with all pills eaten, KC must do victory dance!
        sim.update(0.02);
        assert!(
            matches!(sim.phase, Phase::KcCheer { .. }),
            "Expected KcCheer phase after eating the last pill, got {:?}",
            sim.phase
        );

        let cheer_g0_x = sim.ghosts[0].entity.x;
        let cheer_g0_y = sim.ghosts[0].entity.y;
        let cheer_score = sim.score;

        // Advance through cheer phase (~2.2s) until new maze pass resets
        let initial_pass = sim.pass_count;
        for _ in 0..10 {
            sim.update(0.02);
            assert_eq!(sim.ghosts[0].entity.x, cheer_g0_x, "Ghosts must pause at their coordinates during cheer");
            assert_eq!(sim.ghosts[0].entity.y, cheer_g0_y, "Ghosts must pause at their coordinates during cheer");
        }
        while sim.pass_count == initial_pass {
            sim.update(0.02);
        }

        // Right after cheer, maze has refreshed to a brand new pass and KC eats pills again
        assert_eq!(
            sim.pass_count,
            initial_pass + 1,
            "Expected maze pass count to increment after cheer"
        );
        assert_eq!(
            sim.score,
            cheer_score,
            "Score must be preserved across level advances"
        );
        assert_eq!(
            sim.phase,
            Phase::ApproachingPellet,
            "Simulation should resume in ApproachingPellet phase on new maze"
        );
        assert!(
            sim.pellets.iter().all(|p| p.active),
            "Fresh regular pills must be placed on the new maze"
        );
        assert!(
            sim.super_pellets.iter().all(|sp| sp.active),
            "Fresh super pills must be placed on the new maze"
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
                sim.ghosts[0].entity.col,
                sim.ghosts[0].entity.row,
            );
            assert!(
                (sim.kc.col == sim.ghosts[0].entity.col && sim.kc.row == sim.ghosts[0].entity.row) || path_to_ghost.is_some(),
                "KC at ({}, {}) cannot reach Ghost at ({}, {})!",
                sim.kc.col,
                sim.kc.row,
                sim.ghosts[0].entity.col,
                sim.ghosts[0].entity.row
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

        // Check each quadrant has one super pellet and two regular pellets
        let top_left_sp = sim.super_pellets.iter().filter(|sp| sp.col < mid_c && sp.row < mid_r).count();
        let top_right_sp = sim.super_pellets.iter().filter(|sp| sp.col > mid_c && sp.row < mid_r).count();
        let bot_left_sp = sim.super_pellets.iter().filter(|sp| sp.col < mid_c && sp.row > mid_r).count();
        let bot_right_sp = sim.super_pellets.iter().filter(|sp| sp.col > mid_c && sp.row > mid_r).count();
        assert_eq!(top_left_sp, 1);
        assert_eq!(top_right_sp, 1);
        assert_eq!(bot_left_sp, 1);
        assert_eq!(bot_right_sp, 1);

        let top_left_norm = sim.pellets.iter().filter(|p| p.col < mid_c && p.row < mid_r).count();
        let top_right_norm = sim.pellets.iter().filter(|p| p.col > mid_c && p.row < mid_r).count();
        let bot_left_norm = sim.pellets.iter().filter(|p| p.col < mid_c && p.row > mid_r).count();
        let bot_right_norm = sim.pellets.iter().filter(|p| p.col > mid_c && p.row > mid_r).count();
        assert_eq!(top_left_norm, 2);
        assert_eq!(top_right_norm, 2);
        assert_eq!(bot_left_norm, 2);
        assert_eq!(bot_right_norm, 2);

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
        sim.super_pellets[0].entity.x = px;
        sim.super_pellets[0].entity.y = py;
        sim.super_pellets[0].col = pellet_tile.0;
        sim.super_pellets[0].row = pellet_tile.1;

        // Position Ghost far away so ghost catch doesn't trigger
        sim.ghosts[0].entity.col = sim.maze.cols as i32 - 2;
        sim.ghosts[0].entity.row = sim.maze.rows as i32 - 2;
        let (gx, gy) = sim.maze.tile_to_pixel(sim.ghosts[0].entity.col as f32, sim.ghosts[0].entity.row as f32);
        sim.ghosts[0].entity.x = gx;
        sim.ghosts[0].entity.y = gy;

        // Update one step
        sim.update(0.01);

        // Pellet 0 should now be inactive
        assert!(!sim.super_pellets[0].active);
        // Phase should have switched to PelletChomp
        assert!(matches!(sim.phase, Phase::PelletChomp { .. }));
    }

    #[test]
    fn test_ghost_speed_constant_and_kc_faster_under_enhanced_pill() {
        let mut sim = ChaseSimulation::new(120, 60);

        // Record initial ghost speed
        let expected_ghost_speed = sim.ghosts[0].entity.speed;

        // Phase 1: ApproachingPellet
        sim.phase = Phase::ApproachingPellet;
        sim.update(0.01);
        assert_eq!(sim.ghosts[0].entity.speed, expected_ghost_speed, "Ghost speed must never change in ApproachingPellet");
        assert_eq!(
            sim.kc.speed, sim.ghosts[0].entity.speed,
            "KC speed must be the same as ghost speed in ApproachingPellet"
        );

        // Phase 2: KcChasing (under enhanced pill effect)
        sim.phase = Phase::KcChasing { duration: 8.0 };
        // Separate entities so catch doesn't trigger immediately
        sim.kc.col = 2;
        sim.kc.row = 2;
        sim.ghosts[0].entity.col = 10;
        sim.ghosts[0].entity.row = 10;
        sim.update(0.01);
        assert_eq!(sim.ghosts[0].entity.speed, expected_ghost_speed, "Ghost speed must never change in KcChasing");
        assert!(
            sim.kc.speed > sim.ghosts[0].entity.speed,
            "KC must be faster than the ghost when under enhanced pill effect"
        );
        assert!(
            sim.kc.speed <= sim.ghosts[0].entity.speed * 1.35,
            "KC should move noticeably faster than the ghost without being ridiculously fast (got KC: {}, Ghost: {})",
            sim.kc.speed,
            sim.ghosts[0].entity.speed
        );

        // Phase 3: PowerFading (enhanced pill effect waning)
        sim.phase = Phase::PowerFading { duration: 3.0 };
        sim.update(0.01);
        assert_eq!(sim.ghosts[0].entity.speed, expected_ghost_speed, "Ghost speed must never change in PowerFading");
        assert!(
            sim.kc.speed > sim.ghosts[0].entity.speed,
            "KC must remain slightly faster than the ghost while power is fading"
        );

        // Phase 4: GhostChasing (enhanced pill effect expired)
        sim.phase = Phase::GhostChasing { duration: 10.0 };
        sim.update(0.01);
        assert_eq!(sim.ghosts[0].entity.speed, expected_ghost_speed, "Ghost speed must never change in GhostChasing");
        assert_eq!(
            sim.kc.speed, sim.ghosts[0].entity.speed,
            "KC speed must be the same as ghost speed when not under power pill"
        );
    }

    #[test]
    fn test_central_ghost_cage_and_rotating_door() {
        let mut sim = ChaseSimulation::new(120, 60);

        // Ghost starts at the central cage
        assert_eq!(sim.ghosts[0].entity.col, sim.maze.cage.center_c as i32);
        assert_eq!(sim.ghosts[0].entity.row, sim.maze.cage.center_r as i32);

        // Initial door is Direction::Up
        assert_eq!(sim.maze.cage.door_dir, Direction::Up);
        let (up_c, up_r) = sim.maze.cage.door_tile(Direction::Up);
        assert_eq!(sim.maze.tiles[up_r * sim.maze.cols + up_c], Tile::Path);

        // Move KC and Ghost away from each other and the doorway so door rotation isn't blocked and no catch occurs
        let (kc_away_c, kc_away_r) = (1, 1);
        let ghost_away_c = sim.maze.cols as i32 - 2;
        let ghost_away_r = sim.maze.rows as i32 - 2;
        sim.kc.col = kc_away_c;
        sim.kc.row = kc_away_r;
        sim.kc.target_col = kc_away_c;
        sim.kc.target_row = kc_away_r;
        for g in sim.ghosts.iter_mut() {
            g.entity.col = ghost_away_c;
            g.entity.row = ghost_away_r;
            g.entity.target_col = ghost_away_c;
            g.entity.target_row = ghost_away_r;
        }

        // Advance 1.6s -> Door rotates to Right
        sim.update(1.6);
        assert_eq!(sim.maze.cage.door_dir, Direction::Right);
        let (rt_c, rt_r) = sim.maze.cage.door_tile(Direction::Right);
        assert_eq!(sim.maze.tiles[rt_r * sim.maze.cols + rt_c], Tile::Path);
        assert_eq!(sim.maze.tiles[up_r * sim.maze.cols + up_c], Tile::Wall);

        // Advance 1.6s -> Door rotates to Down
        sim.update(1.6);
        assert_eq!(sim.maze.cage.door_dir, Direction::Down);
        let (dn_c, dn_r) = sim.maze.cage.door_tile(Direction::Down);
        assert_eq!(sim.maze.tiles[dn_r * sim.maze.cols + dn_c], Tile::Path);
        assert_eq!(sim.maze.tiles[rt_r * sim.maze.cols + rt_c], Tile::Wall);

        // Advance 1.6s -> Door rotates to Left
        sim.update(1.6);
        assert_eq!(sim.maze.cage.door_dir, Direction::Left);
        let (lt_c, lt_r) = sim.maze.cage.door_tile(Direction::Left);
        assert_eq!(sim.maze.tiles[lt_r * sim.maze.cols + lt_c], Tile::Path);
        assert_eq!(sim.maze.tiles[dn_r * sim.maze.cols + dn_c], Tile::Wall);

        // Advance 1.6s -> Door rotates back to Up
        sim.update(1.6);
        assert_eq!(sim.maze.cage.door_dir, Direction::Up);
        assert_eq!(sim.maze.tiles[up_r * sim.maze.cols + up_c], Tile::Path);
        assert_eq!(sim.maze.tiles[lt_r * sim.maze.cols + lt_c], Tile::Wall);
    }

    #[test]
    fn test_moving_pills_speed_progression_and_pathfinding() {
        let mut sim = ChaseSimulation::new(120, 60);

        // Verify initial 12 pills (4 super + 8 normal)
        assert_eq!(sim.super_pellets.len(), 4);
        assert_eq!(sim.pellets.len(), 8);
        let total = sim.super_pellets.len() + sim.pellets.len();
        assert_eq!(total, 12);

        let base_speed = sim.ghosts[0].entity.speed;

        // When all 12 pills remain: speed is slow (~0.22 * ghost_speed)
        let initial_speed = sim.current_pill_speed(base_speed);
        assert!(
            initial_speed < base_speed * 0.30,
            "Initial pill speed should be slow (got {})",
            initial_speed
        );

        // Deactivate 5 pills (7 remain)
        for i in 0..5 {
            sim.pellets[i].active = false;
        }
        let mid_speed = sim.current_pill_speed(base_speed);
        assert!(
            mid_speed > initial_speed,
            "Speed must increase as pills are eaten (mid: {}, initial: {})",
            mid_speed,
            initial_speed
        );

        // Deactivate until exactly 2 pills remain
        for i in 5..8 {
            sim.pellets[i].active = false;
        }
        sim.super_pellets[0].active = false;
        sim.super_pellets[1].active = false;
        let active_count = sim.super_pellets.iter().filter(|sp| sp.active).count()
            + sim.pellets.iter().filter(|p| p.active).count();
        assert_eq!(active_count, 2);

        // At 2 pills: moving pretty fast, but not faster than ghost
        let two_pills_speed = sim.current_pill_speed(base_speed);
        assert!(
            two_pills_speed >= base_speed * 0.80 && two_pills_speed < base_speed,
            "Down to 2 pills, speed should be pretty fast but < ghost speed (got {})",
            two_pills_speed
        );

        // Deactivate 1 more so exactly 1 pill remains
        sim.super_pellets[2].active = false;
        let active_count = sim.super_pellets.iter().filter(|sp| sp.active).count()
            + sim.pellets.iter().filter(|p| p.active).count();
        assert_eq!(active_count, 1);

        // At 1 pill: moving about the same speed as the ghost
        let one_pill_speed = sim.current_pill_speed(base_speed);
        assert!(
            one_pill_speed >= base_speed * 0.95 && one_pill_speed <= base_speed * 1.02,
            "Final pill speed should be about the same as the ghost (got {}, ghost: {})",
            one_pill_speed,
            base_speed
        );

        // Verify that advancing simulation moves the active pill along the corridor
        let before_x = sim.super_pellets[3].entity.x;
        let before_y = sim.super_pellets[3].entity.y;
        for _ in 0..5 {
            sim.update(0.05);
        }
        let after_x = sim.super_pellets[3].entity.x;
        let after_y = sim.super_pellets[3].entity.y;
        assert!(
            (after_x - before_x).abs() > 0.001 || (after_y - before_y).abs() > 0.001,
            "Pill must move when simulation advances"
        );
    }

    #[test]
    fn test_three_ghosts_spawn_and_eyes_return_to_cage() {
        let mut sim = ChaseSimulation::new(120, 60);

        // Standard terminal should spawn all 3 ghosts with distinct red, green, and yellow colors
        assert_eq!(sim.ghosts.len(), 3);
        assert_eq!(sim.ghosts[0].config.base_color, Rgb::new(255, 30, 30));
        assert_eq!(sim.ghosts[1].config.base_color, Rgb::new(40, 220, 80));
        assert_eq!(sim.ghosts[2].config.base_color, Rgb::new(255, 220, 30));
        assert_eq!(sim.ghosts[0].state, GhostState::Patrolling);
        assert!(matches!(sim.ghosts[1].state, GhostState::InCage { .. }));
        assert!(matches!(sim.ghosts[2].state, GhostState::InCage { .. }));

        // Eating a vulnerable ghost turns it to Eyes
        let (away_c, away_r) = (1, 1);
        let (ax, ay) = sim.maze.tile_to_pixel(away_c as f32, away_r as f32);
        sim.phase = Phase::KcChasing { duration: 8.0 };
        sim.ghosts[0].state = GhostState::Vulnerable;
        sim.ghosts[0].entity.col = away_c;
        sim.ghosts[0].entity.row = away_r;
        sim.ghosts[0].entity.x = ax;
        sim.ghosts[0].entity.y = ay;

        sim.kc.col = away_c;
        sim.kc.row = away_r;
        sim.kc.x = ax;
        sim.kc.y = ay;

        sim.update(0.01);
        assert_eq!(sim.ghosts[0].state, GhostState::Eyes);
        assert!(sim.ghosts[0].entity.speed > sim.kc.speed);

        // Teleport eyes to cage center
        let cage_c = sim.maze.cage.center_c as i32;
        let cage_r = sim.maze.cage.center_r as i32;
        sim.ghosts[0].entity.col = cage_c;
        sim.ghosts[0].entity.row = cage_r;
        let (px, py) = sim.maze.tile_to_pixel(cage_c as f32, cage_r as f32);
        sim.ghosts[0].entity.x = px;
        sim.ghosts[0].entity.y = py;

        // Step one frame: eyes arrive in cage and start 2.0s timer
        sim.update(0.01);
        match sim.ghosts[0].state {
            GhostState::InCage { timer } => {
                assert!(timer > 1.8 && timer <= 2.0, "Timer should start at 2.0s");
            }
            _ => panic!("Expected InCage state, got {:?}", sim.ghosts[0].state),
        }

        // Move KC far away so KC doesn't re-eat the ghost right when it respawns
        let far_c = sim.maze.cols as i32 - 2;
        let far_r = sim.maze.rows as i32 - 2;
        let (fx, fy) = sim.maze.tile_to_pixel(far_c as f32, far_r as f32);
        sim.kc.col = far_c;
        sim.kc.row = far_r;
        sim.kc.x = fx;
        sim.kc.y = fy;

        // Advance 2.1s: ghost should fully respawn!
        for _ in 0..105 {
            sim.update(0.02);
        }
        assert!(
            matches!(sim.ghosts[0].state, GhostState::Vulnerable | GhostState::Patrolling),
            "Ghost should fully respawn after 2s in cage, got {:?}",
            sim.ghosts[0].state
        );
    }

    #[test]
    fn test_scoring_progression_and_power_reset() {
        let mut sim = ChaseSimulation::new(120, 60);
        assert_eq!(sim.score, 0);

        // Normal pill gives 1 point
        sim.kc.col = sim.pellets[0].col;
        sim.kc.row = sim.pellets[0].row;
        sim.kc.x = sim.pellets[0].entity.x;
        sim.kc.y = sim.pellets[0].entity.y;
        sim.update(0.01);
        assert_eq!(sim.score, 1);

        // Super pill gives 3 points (total 4)
        sim.kc.col = sim.super_pellets[0].col;
        sim.kc.row = sim.super_pellets[0].row;
        sim.kc.x = sim.super_pellets[0].entity.x;
        sim.kc.y = sim.super_pellets[0].entity.y;
        sim.update(0.01);
        assert_eq!(sim.score, 4);

        // Eat 1st ghost: 5 points (total 9)
        let (away_c, away_r) = (1, 1);
        let (ax, ay) = sim.maze.tile_to_pixel(away_c as f32, away_r as f32);
        sim.phase = Phase::KcChasing { duration: 8.0 };
        sim.ghosts[0].state = GhostState::Vulnerable;
        sim.ghosts[0].entity.col = away_c;
        sim.ghosts[0].entity.row = away_r;
        sim.ghosts[0].entity.x = ax;
        sim.ghosts[0].entity.y = ay;
        sim.kc.col = away_c;
        sim.kc.row = away_r;
        sim.kc.x = ax;
        sim.kc.y = ay;
        sim.update(0.01);
        assert_eq!(sim.score, 9);
        assert_eq!(sim.ghosts_eaten_this_power, 1);

        // Eat 2nd ghost: 10 points (total 19)
        sim.ghosts[1].state = GhostState::Vulnerable;
        sim.ghosts[1].entity.col = away_c;
        sim.ghosts[1].entity.row = away_r;
        sim.ghosts[1].entity.x = ax;
        sim.ghosts[1].entity.y = ay;
        sim.kc.col = away_c;
        sim.kc.row = away_r;
        sim.kc.x = ax;
        sim.kc.y = ay;
        sim.update(0.01);
        assert_eq!(sim.score, 19);
        assert_eq!(sim.ghosts_eaten_this_power, 2);

        // Eat 3rd ghost: 20 points (total 39)
        sim.ghosts[2].state = GhostState::Vulnerable;
        sim.ghosts[2].entity.col = away_c;
        sim.ghosts[2].entity.row = away_r;
        sim.ghosts[2].entity.x = ax;
        sim.ghosts[2].entity.y = ay;
        sim.kc.col = away_c;
        sim.kc.row = away_r;
        sim.kc.x = ax;
        sim.kc.y = ay;
        sim.update(0.01);
        assert_eq!(sim.score, 39);
        assert_eq!(sim.ghosts_eaten_this_power, 3);

        // Let power pill wear off: transition to GhostChasing
        sim.phase = Phase::PowerFading { duration: 0.01 };
        sim.update(0.02);
        assert_eq!(sim.ghosts_eaten_this_power, 0, "Counter must reset after power wears off");

        // Eat another super pill: next ghost should give 5 points again
        sim.super_pellets[1].active = true;
        sim.kc.col = sim.super_pellets[1].col;
        sim.kc.row = sim.super_pellets[1].row;
        sim.kc.x = sim.super_pellets[1].entity.x;
        sim.kc.y = sim.super_pellets[1].entity.y;
        sim.update(0.01);
        assert_eq!(sim.score, 39 + 3); // 42

        sim.phase = Phase::KcChasing { duration: 8.0 };
        sim.ghosts[0].state = GhostState::Vulnerable;
        sim.ghosts[0].entity.col = away_c;
        sim.ghosts[0].entity.row = away_r;
        sim.ghosts[0].entity.x = ax;
        sim.ghosts[0].entity.y = ay;
        sim.kc.col = away_c;
        sim.kc.row = away_r;
        sim.kc.x = ax;
        sim.kc.y = ay;
        sim.update(0.01);
        assert_eq!(sim.score, 42 + 5); // 47!
    }

    #[test]
    fn test_victory_dance_only_on_last_pill() {
        let mut sim = ChaseSimulation::new(120, 60);

        // Deactivate all but 1 pill
        for p in sim.pellets.iter_mut() {
            p.active = false;
        }
        for sp in sim.super_pellets.iter_mut().skip(1) {
            sp.active = false;
        }

        let remaining = sim.super_pellets.iter().filter(|sp| sp.active).count()
            + sim.pellets.iter().filter(|p| p.active).count();
        assert_eq!(remaining, 1);

        // Eat a ghost while 1 pill remains: must NOT trigger victory dance!
        let (away_c, away_r) = (sim.maze.cage.center_c as i32, sim.maze.cage.center_r as i32 - 2);
        let (ax, ay) = sim.maze.tile_to_pixel(away_c as f32, away_r as f32);
        sim.phase = Phase::KcChasing { duration: 8.0 };
        sim.ghosts[0].state = GhostState::Vulnerable;
        sim.ghosts[0].entity.col = away_c;
        sim.ghosts[0].entity.row = away_r;
        sim.ghosts[0].entity.x = ax;
        sim.ghosts[0].entity.y = ay;
        sim.kc.col = away_c;
        sim.kc.row = away_r;
        sim.kc.x = ax;
        sim.kc.y = ay;
        sim.update(0.01);

        assert_eq!(sim.ghosts[0].state, GhostState::Eyes);
        assert_ne!(
            sim.phase,
            Phase::KcCheer { timer: 2.2, cheer_time: 0.0 },
            "Eating a ghost must not trigger KcCheer"
        );
        assert!(matches!(sim.phase, Phase::KcChasing { .. }));

        // Now eat the 12th (last) pill
        sim.kc.col = sim.super_pellets[0].col;
        sim.kc.row = sim.super_pellets[0].row;
        sim.kc.x = sim.super_pellets[0].entity.x;
        sim.kc.y = sim.super_pellets[0].entity.y;
        sim.update(0.01);

        let remaining_after = sim.super_pellets.iter().filter(|sp| sp.active).count()
            + sim.pellets.iter().filter(|p| p.active).count();
        assert_eq!(remaining_after, 0);

        // NOW simulation must enter KcCheer!
        assert!(
            matches!(sim.phase, Phase::KcCheer { .. }),
            "Expected KcCheer only after eating the last pill, got {:?}",
            sim.phase
        );
    }

    #[test]
    fn test_eyes_cannot_be_intercepted_and_kc_chases_and_catches_vulnerable_ghost() {
        let mut sim = ChaseSimulation::new(120, 60);

        // Put simulation into KcChasing mode
        sim.phase = Phase::KcChasing { duration: 8.0 };
        sim.update(0.001);

        // Test 1: KC speed is 1.25x ghost speed (noticeable speed boost)
        assert!((sim.kc.speed - sim.ghosts[0].entity.speed * 1.25).abs() < 0.1);

        // Test 2: Ghost 0 is in Eyes state. KC collides directly with Ghost 0 away from the cage base.
        sim.ghosts[0].state = GhostState::Eyes;
        let test_c = 1;
        let test_r = 1;
        let (tx, ty) = sim.maze.tile_to_pixel(test_c as f32, test_r as f32);

        sim.ghosts[0].entity.col = test_c;
        sim.ghosts[0].entity.row = test_r;
        sim.ghosts[0].entity.x = tx;
        sim.ghosts[0].entity.y = ty;

        sim.kc.col = test_c;
        sim.kc.row = test_r;
        sim.kc.x = tx;
        sim.kc.y = ty;

        // Deactivate pill at (1, 1) so it doesn't give pill points
        sim.super_pellets[0].active = false;

        let initial_score = sim.score;
        let initial_eaten = sim.ghosts_eaten_this_power;

        // Step one frame: KC collides with Eyes
        sim.update(0.01);

        // Eyes must NOT be eaten or intercepted: state remains Eyes,
        // score does not change, eaten counter does not change!
        assert_eq!(sim.ghosts[0].state, GhostState::Eyes);
        assert_eq!(sim.score, initial_score, "Colliding with Eyes must not award points");
        assert_eq!(sim.ghosts_eaten_this_power, initial_eaten, "Colliding with Eyes must not increment eaten counter");

        // Test 3: KC catches a vulnerable ghost fleeing across consecutive frames
        sim.phase = Phase::KcChasing { duration: 8.0 };
        sim.ghosts[1].state = GhostState::Vulnerable;
        // Move pills away to the opposite corner so KC pursues the nearby vulnerable ghost
        let far_c = sim.maze.cols as i32 - 2;
        let far_r = sim.maze.rows as i32 - 2;
        let (fpx, fpy) = sim.maze.tile_to_pixel(far_c as f32, far_r as f32);
        for p in sim.pellets.iter_mut() {
            p.col = far_c;
            p.row = far_r;
            p.entity.x = fpx;
            p.entity.y = fpy;
        }
        for sp in sim.super_pellets.iter_mut() {
            sp.col = far_c;
            sp.row = far_r;
            sp.entity.x = fpx;
            sp.entity.y = fpy;
        }

        // Place Ghost 1 2 tiles away from KC
        sim.kc.col = test_c;
        sim.kc.row = test_r;
        sim.kc.target_col = test_c;
        sim.kc.target_row = test_r;
        sim.kc.x = tx;
        sim.kc.y = ty;
        sim.ghosts[1].entity.col = test_c;
        sim.ghosts[1].entity.row = test_r + 2;
        sim.ghosts[1].entity.target_col = test_c;
        sim.ghosts[1].entity.target_row = test_r + 2;
        let (g1x, g1y) = sim.maze.tile_to_pixel(test_c as f32, (test_r + 2) as f32);
        sim.ghosts[1].entity.x = g1x;
        sim.ghosts[1].entity.y = g1y;

        // Advance simulation: KC actively pursues Ghost 1 and catches it due to superior speed
        let mut caught = false;
        for _ in 0..120 {
            sim.update(0.02);
            if sim.ghosts[1].state == GhostState::Eyes {
                caught = true;
                break;
            }
        }
        assert!(caught, "KC must successfully catch up to and eat the fleeing vulnerable ghost!");
    }

    #[test]
    fn test_ghost_catches_kc_resets_level_after_timer() {
        let mut sim = ChaseSimulation::new(120, 60);

        // Set an active score and pass count
        sim.score = 24;
        sim.high_score = 10;
        let initial_pass = sim.pass_count;

        // Position Ghost 0 (Patrolling) right on top of KC
        let kc_c = sim.kc.col;
        let kc_r = sim.kc.row;
        let (kx, ky) = (sim.kc.x, sim.kc.y);

        sim.phase = Phase::ApproachingPellet;
        sim.ghosts[0].state = GhostState::Patrolling;
        sim.ghosts[0].entity.col = kc_c;
        sim.ghosts[0].entity.row = kc_r;
        sim.ghosts[0].entity.x = kx;
        sim.ghosts[0].entity.y = ky;

        // Step one frame: Ghost catches KC
        sim.update(0.02);

        // Ghost catch must trigger Phase::GhostCatchKc with 0.8s timer
        match sim.phase {
            Phase::GhostCatchKc { timer } => {
                assert!(timer <= 0.8 && timer > 0.7, "Expected initial timer around 0.8, got {}", timer);
            }
            ref p => panic!("Expected Phase::GhostCatchKc, got {:?}", p),
        }

        // Entity positions should freeze while defeat animation plays
        let frozen_gx = sim.ghosts[0].entity.x;
        let frozen_gy = sim.ghosts[0].entity.y;

        // Advance 0.4s (midway through defeat phase)
        for _ in 0..20 {
            sim.update(0.02);
        }

        // Must still be in GhostCatchKc with timer ~0.38, NOT reset back to 0.8!
        match sim.phase {
            Phase::GhostCatchKc { timer } => {
                assert!(
                    timer < 0.5 && timer > 0.25,
                    "Timer should have steadily decreased to ~0.38, got {}",
                    timer
                );
            }
            ref p => panic!("Expected still in Phase::GhostCatchKc midway, got {:?}", p),
        }
        assert_eq!(sim.ghosts[0].entity.x, frozen_gx, "Ghost should be frozen during defeat animation");
        assert_eq!(sim.ghosts[0].entity.y, frozen_gy, "Ghost should be frozen during defeat animation");

        // Advance past remaining defeat timer (> 0.4s)
        for _ in 0..25 {
            sim.update(0.02);
        }

        // Level must now have reset!
        assert!(
            matches!(sim.phase, Phase::ApproachingPellet),
            "Expected level reset to ApproachingPellet phase, got {:?}",
            sim.phase
        );
        assert_eq!(sim.pass_count, initial_pass + 1, "Pass count should increment after level reset");
        assert_eq!(sim.score, 0, "Score should reset to 0 after dying");
        assert_eq!(sim.high_score, 24, "High score should record KC's score prior to death");
    }

    #[test]
    fn test_kc_priority_pills_over_distant_ghosts() {
        let mut sim = ChaseSimulation::new(120, 60);

        // Put simulation into KcChasing mode
        sim.phase = Phase::KcChasing { duration: 8.0 };

        // Deactivate all pills except one normal pill 3 tiles to the right
        for sp in sim.super_pellets.iter_mut() {
            sp.active = false;
        }
        for p in sim.pellets.iter_mut().skip(1) {
            p.active = false;
        }

        // Find a path tile for KC that connects horizontally
        let (away_c, away_r) = (1, 1);
        let (ax, ay) = sim.maze.tile_to_pixel(away_c as f32, away_r as f32);
        sim.kc.col = away_c;
        sim.kc.row = away_r;
        sim.kc.target_col = away_c;
        sim.kc.target_row = away_r;
        sim.kc.x = ax;
        sim.kc.y = ay;

        // Place remaining pill in corridor ahead
        let pill_c = away_c + 2;
        let pill_r = away_r;
        sim.pellets[0].active = true;
        sim.pellets[0].col = pill_c;
        sim.pellets[0].row = pill_r;
        let (px, py) = sim.maze.tile_to_pixel(pill_c as f32, pill_r as f32);
        sim.pellets[0].entity.x = px;
        sim.pellets[0].entity.y = py;

        // Place a vulnerable ghost far away at opposite corner
        let far_c = sim.maze.cols as i32 - 2;
        let far_r = sim.maze.rows as i32 - 2;
        sim.ghosts[0].state = GhostState::Vulnerable;
        sim.ghosts[0].entity.col = far_c;
        sim.ghosts[0].entity.row = far_r;
        let (gx, gy) = sim.maze.tile_to_pixel(far_c as f32, far_r as f32);
        sim.ghosts[0].entity.x = gx;
        sim.ghosts[0].entity.y = gy;

        // Advance one decision point
        sim.update(0.01);

        // KC's first priority is always pills, so KC must head towards the pill,
        // NOT towards the distant vulnerable ghost
        let next_dir_to_pill = sim.maze.bfs_next_direction(away_c, away_r, pill_c, pill_r);
        assert_eq!(
            Some(sim.kc.dir),
            next_dir_to_pill,
            "KC must prioritize path towards pill over distant ghost"
        );
    }

    #[test]
    fn test_kc_overtakes_and_eats_ghost_in_same_corridor() {
        let mut sim = ChaseSimulation::new(120, 60);

        // Put simulation into KcChasing mode
        sim.phase = Phase::KcChasing { duration: 8.0 };
        sim.ghosts[0].state = GhostState::Vulnerable;

        // Position KC at (1, 1) and Ghost 0 at (1, 2)
        let (ax, ay) = sim.maze.tile_to_pixel(1.0, 1.0);
        let (gx, gy) = sim.maze.tile_to_pixel(1.0, 2.0);

        sim.kc.col = 1;
        sim.kc.row = 1;
        sim.kc.target_col = 1;
        sim.kc.target_row = 1;
        sim.kc.x = ax;
        sim.kc.y = ay;

        sim.ghosts[0].entity.col = 1;
        sim.ghosts[0].entity.row = 2;
        sim.ghosts[0].entity.target_col = 1;
        sim.ghosts[0].entity.target_row = 2;
        sim.ghosts[0].entity.x = gx;
        sim.ghosts[0].entity.y = gy;

        // Advance simulation: KC must close the gap and catch ghost
        let mut caught = false;
        for _ in 0..60 {
            sim.update(0.02);
            if sim.ghosts[0].state == GhostState::Eyes {
                caught = true;
                break;
            }
        }
        assert!(caught, "KC must overtake and catch the vulnerable ghost right ahead in the corridor without stalling");
    }

    #[test]
    fn test_intermission_6_cycles_and_returns_to_game() {
        let mut sim = ChaseSimulation::new_with_sequence(100, 50, StartSequence::Intermission);
        assert_eq!(sim.sequence_mode, SequenceMode::Intermission);
        assert_eq!(sim.intermission.target_cycles, 6);
        assert_eq!(sim.intermission.cycle, 0);

        // Advance simulation dt chunks until intermission completes
        let mut steps = 0;
        while sim.sequence_mode == SequenceMode::Intermission && steps < 10000 {
            sim.update(0.05);
            steps += 1;
        }

        assert_eq!(
            sim.sequence_mode,
            SequenceMode::Game,
            "After 6 cycles of intermission, simulation must transition back to game mode"
        );
        assert!(
            sim.intermission.finished,
            "Intermission must be marked finished after 6 cycles"
        );
        assert_eq!(
            sim.levels_in_game_sequence, 0,
            "Game sequence level count must reset to 0 after returning from intermission"
        );
        assert!(
            !sim.pellets.is_empty(),
            "Game mode must have active pellets populated"
        );
    }

    #[test]
    fn test_intermission_lines_change_between_sweeps() {
        let mut inter = IntermissionState::new(100, 50, 6);
        let first_y = inter.current_y;
        assert_eq!(inter.phase, IntermissionPhase::KcChasingGhosts);

        // Advance until phase transitions to GhostsChasingKc (off right edge)
        let mut steps = 0;
        while inter.phase == IntermissionPhase::KcChasingGhosts && steps < 2000 {
            inter.update(0.05, 100, 50, 8.0);
            steps += 1;
        }
        assert_eq!(inter.phase, IntermissionPhase::GhostsChasingKc);
        let second_y = inter.current_y;
        assert_ne!(
            first_y, second_y,
            "Horizontal line must change when returning running away from ghosts"
        );

        // Advance until phase transitions back to KcChasingGhosts (off left edge)
        while inter.phase == IntermissionPhase::GhostsChasingKc && steps < 4000 {
            inter.update(0.05, 100, 50, 8.0);
            steps += 1;
        }
        assert_eq!(inter.phase, IntermissionPhase::KcChasingGhosts);
        let third_y = inter.current_y;
        assert_ne!(
            second_y, third_y,
            "Horizontal line must change again for the next cycle"
        );
        assert_eq!(inter.cycle, 1);
    }

    #[test]
    fn test_game_sequence_switches_to_intermission_after_two_levels_on_defeat() {
        let mut sim = ChaseSimulation::new_with_sequence(100, 50, StartSequence::Game);
        assert_eq!(sim.sequence_mode, SequenceMode::Game);

        // Level 1 defeat
        sim.phase = Phase::GhostCatchKc { timer: 0.01 };
        sim.update(0.02);
        assert_eq!(sim.sequence_mode, SequenceMode::Game);
        assert_eq!(sim.levels_in_game_sequence, 1);

        // Level 2 defeat -> After >= 2 levels and not winning, switches to Intermission
        sim.phase = Phase::GhostCatchKc { timer: 0.01 };
        sim.update(0.02);
        assert_eq!(sim.sequence_mode, SequenceMode::Intermission);
        assert_eq!(sim.intermission.cycle, 0);
    }

    #[test]
    fn test_game_sequence_winning_streak_continues_levels() {
        let mut sim = ChaseSimulation::new_with_sequence(100, 50, StartSequence::Game);

        // Level 1 victory: K.C. won, so continue playing
        sim.phase = Phase::KcCheer { timer: 0.01, cheer_time: 2.0 };
        sim.update(0.02);
        assert_eq!(sim.sequence_mode, SequenceMode::Game);
        assert_eq!(sim.levels_in_game_sequence, 1);

        // Level 2 victory: K.C. won again, winning streak continues without intermission
        sim.phase = Phase::KcCheer { timer: 0.01, cheer_time: 2.0 };
        sim.update(0.02);
        assert_eq!(sim.sequence_mode, SequenceMode::Game);
        assert_eq!(sim.levels_in_game_sequence, 2);

        // Level 3 defeat: Now after >= 2 levels, K.C. is defeated -> triggers Intermission to rest edges
        sim.phase = Phase::GhostCatchKc { timer: 0.01 };
        sim.update(0.02);
        assert_eq!(sim.sequence_mode, SequenceMode::Intermission);
    }
}
