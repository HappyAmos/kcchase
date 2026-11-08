use crate::canvas::Canvas;
use crate::color::Rgb;
use crate::sprites::{
    draw_ghost, draw_kc_munchkin, draw_pellet, draw_super_pellet, GhostConfig, GhostState,
};
use rand::RngExt;
use std::f32::consts::PI;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Phase {
    /// KC approaches the super pellet, ghost is wandering
    ApproachingPellet,
    /// Pellet just eaten, flash/shockwave effect
    PelletChomp { timer: f32 },
    /// KC is supercharged and actively chasing the scared ghost
    KcChasing { duration: f32 },
    /// Super pellet effect is wearing off, ghost flashing rapidly
    PowerFading { duration: f32 },
    /// Ghost is chasing KC, KC is fleeing in panic
    GhostChasing { duration: f32 },
    /// Characters run off screen, transition to next pass
    ExitPass { timer: f32 },
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
pub struct Pellet {
    pub x: f32,
    pub y: f32,
    pub eaten: bool,
}

pub struct ChaseSimulation {
    // Entities
    pub kc_x: f32,
    pub kc_y: f32,
    pub kc_dir: f32,
    pub kc_mouth: f32,

    pub ghost_x: f32,
    pub ghost_y: f32,
    pub ghost_dir: f32,
    pub ghost_config: GhostConfig,

    pub super_pellet_x: f32,
    pub super_pellet_y: f32,
    pub super_pellet_active: bool,

    pub normal_pellets: Vec<Pellet>,
    pub particles: Vec<Particle>,

    // Sizing
    pub character_scale: f32,
    pub base_radius: f32,

    // State machine
    pub phase: Phase,
    pub anim_time: f32,
    pub pass_count: usize,

    // Arena boundary
    pub arena_w: f32,
    pub arena_h: f32,
}

impl ChaseSimulation {
    pub fn new(canvas_w: usize, canvas_h: usize) -> Self {
        let mut sim = ChaseSimulation {
            kc_x: 0.0,
            kc_y: 0.0,
            kc_dir: 0.0,
            kc_mouth: 0.5,

            ghost_x: 0.0,
            ghost_y: 0.0,
            ghost_dir: PI,
            ghost_config: GhostConfig::red_bit(),

            super_pellet_x: 0.0,
            super_pellet_y: 0.0,
            super_pellet_active: true,

            normal_pellets: Vec::new(),
            particles: Vec::new(),

            character_scale: 1.0,
            base_radius: 5.0,

            phase: Phase::ApproachingPellet,
            anim_time: 0.0,
            pass_count: 0,

            arena_w: canvas_w as f32,
            arena_h: canvas_h as f32,
        };

        sim.reset_pass(canvas_w, canvas_h);
        sim
    }

    /// Resize simulation arena when terminal size changes
    pub fn resize(&mut self, canvas_w: usize, canvas_h: usize) {
        let old_w = self.arena_w;
        let old_h = self.arena_h;
        self.arena_w = canvas_w as f32;
        self.arena_h = canvas_h as f32;

        if old_w > 0.0 && old_h > 0.0 {
            let sx = self.arena_w / old_w;
            let sy = self.arena_h / old_h;
            self.kc_x *= sx;
            self.kc_y *= sy;
            self.ghost_x *= sx;
            self.ghost_y *= sy;
            self.super_pellet_x *= sx;
            self.super_pellet_y *= sy;
        }

        // Clamp inside new bounds
        self.kc_x = self.kc_x.clamp(10.0, (self.arena_w - 10.0).max(10.0));
        self.kc_y = self.kc_y.clamp(10.0, (self.arena_h - 10.0).max(10.0));
        self.ghost_x = self.ghost_x.clamp(10.0, (self.arena_w - 10.0).max(10.0));
        self.ghost_y = self.ghost_y.clamp(10.0, (self.arena_h - 10.0).max(10.0));
    }

    /// Initialize a new pass with varied size, positions, and ghost character
    pub fn reset_pass(&mut self, canvas_w: usize, canvas_h: usize) {
        self.arena_w = canvas_w as f32;
        self.arena_h = canvas_h as f32;
        let mut rng = rand::rng();

        self.pass_count += 1;

        // Variety of scales on each pass!
        // Adjust scale choices based on available screen space
        let min_dim = self.arena_w.min(self.arena_h);
        let max_allowed_scale = (min_dim / 30.0).clamp(0.8, 3.2);

        let scale_options = [0.85, 1.0, 1.35, 1.7, 2.1, 2.6];
        let viable_scales: Vec<f32> = scale_options
            .iter()
            .copied()
            .filter(|&s| s <= max_allowed_scale)
            .collect();

        self.character_scale = if !viable_scales.is_empty() {
            viable_scales[rng.random_range(0..viable_scales.len())]
        } else {
            1.0
        };

        // Alternate or randomize ghost personality
        self.ghost_config = match rng.random_range(0..4) {
            0 => GhostConfig::red_bit(),
            1 => GhostConfig::purple_byte(),
            2 => GhostConfig::green_middie(),
            _ => GhostConfig::orange_muncher(),
        };

        // Pick initial positions
        let margin = (self.current_radius() * 2.5).max(12.0);
        let left_x = margin + 10.0;
        let right_x = (self.arena_w - margin - 10.0).max(left_x + 20.0);
        let mid_y = self.arena_h * 0.5;

        // Choose sides
        let kc_starts_left = rng.random_bool(0.5);
        if kc_starts_left {
            self.kc_x = left_x;
            self.kc_y = mid_y + rng.random_range(-10.0..10.0);
            self.kc_dir = 0.0;

            self.ghost_x = right_x;
            self.ghost_y = mid_y + rng.random_range(-15.0..15.0);
            self.ghost_dir = PI;

            // Place super pellet in KC's path, between left and center
            self.super_pellet_x = self.arena_w * rng.random_range(0.35..0.55);
            self.super_pellet_y = mid_y + rng.random_range(-12.0..12.0);
        } else {
            self.kc_x = right_x;
            self.kc_y = mid_y + rng.random_range(-10.0..10.0);
            self.kc_dir = PI;

            self.ghost_x = left_x;
            self.ghost_y = mid_y + rng.random_range(-15.0..15.0);
            self.ghost_dir = 0.0;

            self.super_pellet_x = self.arena_w * rng.random_range(0.45..0.65);
            self.super_pellet_y = mid_y + rng.random_range(-12.0..12.0);
        }

        self.super_pellet_active = true;
        self.phase = Phase::ApproachingPellet;

        // Spawn scattered regular pellets across the arena
        self.normal_pellets.clear();
        let pellet_cols = ((self.arena_w - 20.0) / 14.0).max(4.0) as usize;
        let pellet_rows = ((self.arena_h - 20.0) / 8.0).max(3.0) as usize;

        let col_step = (self.arena_w - 30.0) / (pellet_cols as f32);
        let row_step = (self.arena_h - 24.0) / (pellet_rows as f32);

        for r in 0..pellet_rows {
            for c in 0..pellet_cols {
                if rng.random_bool(0.7) {
                    let px = 15.0 + (c as f32 + 0.5) * col_step + rng.random_range(-2.0..2.0);
                    let py = 12.0 + (r as f32 + 0.5) * row_step + rng.random_range(-1.5..1.5);
                    // Don't spawn right on top of super pellet
                    let dx = px - self.super_pellet_x;
                    let dy = py - self.super_pellet_y;
                    if dx * dx + dy * dy > 64.0 {
                        self.normal_pellets.push(Pellet {
                            x: px,
                            y: py,
                            eaten: false,
                        });
                    }
                }
            }
        }
    }

    pub fn current_radius(&self) -> f32 {
        self.base_radius * self.character_scale
    }

    /// Advance physics, AI, and animations by dt (seconds)
    pub fn update(&mut self, dt: f32) {
        self.anim_time += dt;

        // Animate mouth chomping
        let chomp_speed = match self.phase {
            Phase::ApproachingPellet => 10.0,
            Phase::KcChasing { .. } => 18.0,
            Phase::PowerFading { .. } => 14.0,
            Phase::GhostChasing { .. } => 24.0, // Chomping frantically in panic!
            _ => 8.0,
        };
        self.kc_mouth = ((self.anim_time * chomp_speed).sin() + 1.0) * 0.5;

        // Update active particles
        self.update_particles(dt);

        // Check normal pellet chomping by KC
        let kc_r = self.current_radius();
        let mut eaten_pellets = Vec::new();
        for p in self.normal_pellets.iter_mut() {
            if !p.eaten {
                let dx = p.x - self.kc_x;
                let dy = p.y - self.kc_y;
                if dx * dx + dy * dy < (kc_r + 2.0) * (kc_r + 2.0) {
                    p.eaten = true;
                    eaten_pellets.push((p.x, p.y));
                }
            }
        }
        for (px, py) in eaten_pellets {
            self.spawn_munch_sparkles(px, py, Rgb::new(255, 230, 80));
        }

        let margin = (kc_r + 4.0).max(8.0);
        let arena_w = self.arena_w;
        let arena_h = self.arena_h;

        // State machine logic
        let mut next_phase = self.phase;

        match self.phase {
            Phase::ApproachingPellet => {
                // KC moves purposefully towards super pellet
                let speed = 40.0 * self.character_scale.sqrt();
                steer_towards_target(
                    &mut self.kc_x,
                    &mut self.kc_y,
                    &mut self.kc_dir,
                    self.super_pellet_x,
                    self.super_pellet_y,
                    speed,
                    dt,
                    margin,
                    arena_w,
                    arena_h,
                );

                // Ghost patrols or wanders casually
                self.ghost_wander(dt, 26.0 * self.character_scale.sqrt());

                // Check if KC reaches super pellet!
                let dx = self.kc_x - self.super_pellet_x;
                let dy = self.kc_y - self.super_pellet_y;
                let dist = (dx * dx + dy * dy).sqrt();
                if dist < kc_r + 3.0 && self.super_pellet_active {
                    self.super_pellet_active = false;
                    next_phase = Phase::PelletChomp { timer: 0.6 };
                    self.spawn_shockwave(self.super_pellet_x, self.super_pellet_y);
                }
            }

            Phase::PelletChomp { timer } => {
                let new_timer = timer - dt;
                if new_timer <= 0.0 {
                    // KC starts chasing ghost! (lasts 7 seconds)
                    next_phase = Phase::KcChasing { duration: 7.0 };
                } else {
                    next_phase = Phase::PelletChomp { timer: new_timer };
                }
            }

            Phase::KcChasing { duration } => {
                let new_duration = duration - dt;

                // KC chases Ghost at high speed!
                let kc_speed = 52.0 * self.character_scale.sqrt();
                steer_towards_target(
                    &mut self.kc_x,
                    &mut self.kc_y,
                    &mut self.kc_dir,
                    self.ghost_x,
                    self.ghost_y,
                    kc_speed,
                    dt,
                    margin,
                    arena_w,
                    arena_h,
                );

                // Ghost flees away from KC!
                let ghost_speed = 43.0 * self.character_scale.sqrt();
                self.ghost_flee_from_kc(dt, ghost_speed);

                // If KC catches the ghost, or duration expires
                let dx = self.kc_x - self.ghost_x;
                let dy = self.kc_y - self.ghost_y;
                let dist = (dx * dx + dy * dy).sqrt();

                if new_duration <= 0.0 {
                    // Transition to fading power warning
                    next_phase = Phase::PowerFading { duration: 3.5 };
                } else if dist < kc_r * 1.2 {
                    // Chomp impact! Bonus sparkles, transition to fading
                    self.spawn_munch_sparkles(self.ghost_x, self.ghost_y, Rgb::new(80, 160, 255));
                    next_phase = Phase::PowerFading { duration: 2.5 };
                } else {
                    next_phase = Phase::KcChasing {
                        duration: new_duration,
                    };
                }
            }

            Phase::PowerFading { duration } => {
                let new_duration = duration - dt;

                // Speeds level off
                let speed = 40.0 * self.character_scale.sqrt();
                steer_towards_target(
                    &mut self.kc_x,
                    &mut self.kc_y,
                    &mut self.kc_dir,
                    self.ghost_x,
                    self.ghost_y,
                    speed * 0.85,
                    dt,
                    margin,
                    arena_w,
                    arena_h,
                );
                self.ghost_flee_from_kc(dt, speed * 0.8);

                if new_duration <= 0.0 {
                    // Effect completely faded! Ghost turns around and hunts KC!
                    next_phase = Phase::GhostChasing { duration: 8.0 };
                    // Visual boom as ghost regains strength
                    self.spawn_shockwave(self.ghost_x, self.ghost_y);
                } else {
                    next_phase = Phase::PowerFading {
                        duration: new_duration,
                    };
                }
            }

            Phase::GhostChasing { duration } => {
                let new_duration = duration - dt;

                // Ghost chases KC aggressively!
                let ghost_speed = 54.0 * self.character_scale.sqrt();
                steer_towards_target(
                    &mut self.ghost_x,
                    &mut self.ghost_y,
                    &mut self.ghost_dir,
                    self.kc_x,
                    self.kc_y,
                    ghost_speed,
                    dt,
                    margin,
                    arena_w,
                    arena_h,
                );

                // KC flees away from Ghost in panic!
                let kc_speed = 47.0 * self.character_scale.sqrt();
                self.kc_flee_from_ghost(dt, kc_speed);

                // If ghost gets close to KC or duration ends, transition to exit
                let dx = self.ghost_x - self.kc_x;
                let dy = self.ghost_y - self.kc_y;
                let dist = (dx * dx + dy * dy).sqrt();

                if new_duration <= 0.0 || dist < kc_r * 1.5 {
                    next_phase = Phase::ExitPass { timer: 2.0 };
                } else {
                    next_phase = Phase::GhostChasing {
                        duration: new_duration,
                    };
                }
            }

            Phase::ExitPass { timer } => {
                let new_timer = timer - dt;

                // Both run off the edges
                let speed = 65.0 * self.character_scale.sqrt();
                self.kc_x += self.kc_dir.cos() * speed * dt;
                self.kc_y += self.kc_dir.sin() * speed * dt;

                self.ghost_x += self.ghost_dir.cos() * speed * dt;
                self.ghost_y += self.ghost_dir.sin() * speed * dt;

                if new_timer <= 0.0 {
                    // Start next pass with new size, character, and layout!
                    let w = self.arena_w as usize;
                    let h = self.arena_h as usize;
                    self.reset_pass(w, h);
                    return;
                } else {
                    next_phase = Phase::ExitPass { timer: new_timer };
                }
            }
        }

        self.phase = next_phase;
    }

    /// Render the full screensaver frame onto canvas
    pub fn draw(&self, canvas: &mut Canvas) {
        canvas.clear();

        // Draw retro arena maze border
        self.draw_arena_border(canvas);

        // Draw scattered pellets
        for p in &self.normal_pellets {
            if !p.eaten {
                draw_pellet(canvas, p.x, p.y, self.anim_time);
            }
        }

        // Draw Super Pellet if active
        if self.super_pellet_active {
            let pellet_rad = (self.current_radius() * 0.7).max(2.2);
            draw_super_pellet(
                canvas,
                self.super_pellet_x,
                self.super_pellet_y,
                pellet_rad,
                self.anim_time,
            );
        }

        // Draw particles
        for p in &self.particles {
            let alpha = (p.life / p.max_life).clamp(0.0, 1.0);
            let col = p.color.lerp(Rgb::BLACK, 1.0 - alpha);
            canvas.fill_circle_smooth(p.x, p.y, p.size, col);
        }

        let radius = self.current_radius();

        // Draw KC Munchkin
        let is_supercharged =
            matches!(self.phase, Phase::KcChasing { .. } | Phase::PelletChomp { .. });
        let is_terrified =
            matches!(self.phase, Phase::GhostChasing { .. } | Phase::ExitPass { .. });

        draw_kc_munchkin(
            canvas,
            self.kc_x,
            self.kc_y,
            radius,
            self.kc_dir,
            self.kc_mouth,
            is_supercharged,
            is_terrified,
            self.anim_time,
        );

        // Determine Ghost state and draw Ghost
        let ghost_state = match self.phase {
            Phase::ApproachingPellet => GhostState::Patrolling,
            Phase::PelletChomp { .. } | Phase::KcChasing { .. } => GhostState::Vulnerable,
            Phase::PowerFading { duration } => GhostState::Fading {
                time_remaining: duration,
            },
            Phase::GhostChasing { .. } | Phase::ExitPass { .. } => GhostState::Hunting,
        };

        draw_ghost(
            canvas,
            self.ghost_x,
            self.ghost_y,
            radius,
            self.ghost_dir,
            ghost_state,
            &self.ghost_config,
            self.anim_time,
        );
    }

    /// Draw a retro neon Odyssey 2 arena border
    fn draw_arena_border(&self, canvas: &mut Canvas) {
        let border_color = match self.phase {
            Phase::KcChasing { .. } => Rgb::new(20, 100, 220), // Odyssey blue
            Phase::PowerFading { .. } => {
                let flash = (self.anim_time * 10.0).sin() > 0.0;
                if flash {
                    Rgb::new(240, 60, 60)
                } else {
                    Rgb::new(20, 100, 220)
                }
            }
            Phase::GhostChasing { .. } => Rgb::new(200, 30, 30), // Menacing red
            _ => Rgb::new(40, 60, 160),
        };

        let w = self.arena_w as i32;
        let h = self.arena_h as i32;
        if w < 10 || h < 10 {
            return;
        }

        // Draw top and bottom borders
        for x in 0..w {
            canvas.set_pixel(x, 0, border_color);
            canvas.set_pixel(x, 1, border_color);
            canvas.set_pixel(x, h - 2, border_color);
            canvas.set_pixel(x, h - 1, border_color);
        }

        // Draw left and right borders
        for y in 0..h {
            canvas.set_pixel(0, y, border_color);
            canvas.set_pixel(1, y, border_color);
            canvas.set_pixel(w - 2, y, border_color);
            canvas.set_pixel(w - 1, y, border_color);
        }

        // Odyssey 2 maze corner accents
        let accent_len = (w.min(h) / 8).clamp(4, 20);
        let accent_color = Rgb::new(120, 180, 255);
        for i in 0..accent_len {
            canvas.set_pixel(2 + i, 2, accent_color);
            canvas.set_pixel(2, 2 + i, accent_color);
            canvas.set_pixel(w - 3 - i, 2, accent_color);
            canvas.set_pixel(w - 3, 2 + i, accent_color);
            canvas.set_pixel(2 + i, h - 3, accent_color);
            canvas.set_pixel(2, h - 3 - i, accent_color);
            canvas.set_pixel(w - 3 - i, h - 3, accent_color);
            canvas.set_pixel(w - 3, h - 3 - i, accent_color);
        }
    }

    // Steering and movement helpers

    fn ghost_flee_from_kc(&mut self, dt: f32, speed: f32) {
        let margin = (self.current_radius() + 6.0).max(8.0);

        // Vector away from KC
        let mut flee_dx = self.ghost_x - self.kc_x;
        let mut flee_dy = self.ghost_y - self.kc_y;
        let dist = (flee_dx * flee_dx + flee_dy * flee_dy).sqrt().max(1.0);
        flee_dx /= dist;
        flee_dy /= dist;

        // Add wall avoidance repulsive forces
        let wall_repulse_dist = 18.0;
        if self.ghost_x < margin + wall_repulse_dist {
            flee_dx += (margin + wall_repulse_dist - self.ghost_x) / wall_repulse_dist * 2.0;
        }
        if self.ghost_x > self.arena_w - margin - wall_repulse_dist {
            flee_dx -= (self.ghost_x - (self.arena_w - margin - wall_repulse_dist))
                / wall_repulse_dist
                * 2.0;
        }
        if self.ghost_y < margin + wall_repulse_dist {
            flee_dy += (margin + wall_repulse_dist - self.ghost_y) / wall_repulse_dist * 2.0;
        }
        if self.ghost_y > self.arena_h - margin - wall_repulse_dist {
            flee_dy -= (self.ghost_y - (self.arena_h - margin - wall_repulse_dist))
                / wall_repulse_dist
                * 2.0;
        }

        // Add small serpentine wobble for classic ghost evasion
        let wobble = (self.anim_time * 6.0).sin() * 0.4;
        let target_dir = flee_dy.atan2(flee_dx) + wobble;

        let mut diff = target_dir - self.ghost_dir;
        while diff > PI {
            diff -= 2.0 * PI;
        }
        while diff < -PI {
            diff += 2.0 * PI;
        }
        self.ghost_dir += diff * (7.0 * dt).min(1.0);

        self.ghost_x += self.ghost_dir.cos() * speed * dt;
        self.ghost_y += self.ghost_dir.sin() * speed * dt;

        self.ghost_x = self.ghost_x.clamp(margin, self.arena_w - margin);
        self.ghost_y = self.ghost_y.clamp(margin, self.arena_h - margin);
    }

    fn kc_flee_from_ghost(&mut self, dt: f32, speed: f32) {
        let margin = (self.current_radius() + 6.0).max(8.0);

        // Vector away from Ghost
        let mut flee_dx = self.kc_x - self.ghost_x;
        let mut flee_dy = self.kc_y - self.ghost_y;
        let dist = (flee_dx * flee_dx + flee_dy * flee_dy).sqrt().max(1.0);
        flee_dx /= dist;
        flee_dy /= dist;

        // Wall avoidance
        let wall_repulse_dist = 20.0;
        if self.kc_x < margin + wall_repulse_dist {
            flee_dx += (margin + wall_repulse_dist - self.kc_x) / wall_repulse_dist * 2.0;
        }
        if self.kc_x > self.arena_w - margin - wall_repulse_dist {
            flee_dx -= (self.kc_x - (self.arena_w - margin - wall_repulse_dist))
                / wall_repulse_dist
                * 2.0;
        }
        if self.kc_y < margin + wall_repulse_dist {
            flee_dy += (margin + wall_repulse_dist - self.kc_y) / wall_repulse_dist * 2.0;
        }
        if self.kc_y > self.arena_h - margin - wall_repulse_dist {
            flee_dy -= (self.kc_y - (self.arena_h - margin - wall_repulse_dist))
                / wall_repulse_dist
                * 2.0;
        }

        let target_dir = flee_dy.atan2(flee_dx);
        let mut diff = target_dir - self.kc_dir;
        while diff > PI {
            diff -= 2.0 * PI;
        }
        while diff < -PI {
            diff += 2.0 * PI;
        }
        self.kc_dir += diff * (8.0 * dt).min(1.0);

        self.kc_x += self.kc_dir.cos() * speed * dt;
        self.kc_y += self.kc_dir.sin() * speed * dt;

        self.kc_x = self.kc_x.clamp(margin, self.arena_w - margin);
        self.kc_y = self.kc_y.clamp(margin, self.arena_h - margin);
    }

    fn ghost_wander(&mut self, dt: f32, speed: f32) {
        let margin = (self.current_radius() + 6.0).max(8.0);

        // Gentle wandering
        let wander_noise = (self.anim_time * 1.5).sin() * 0.8;
        self.ghost_dir += wander_noise * dt;

        // Bounce away from edges
        if self.ghost_x < margin + 10.0 && self.ghost_dir.cos() < 0.0 {
            self.ghost_dir = PI - self.ghost_dir;
        }
        if self.ghost_x > self.arena_w - margin - 10.0 && self.ghost_dir.cos() > 0.0 {
            self.ghost_dir = PI - self.ghost_dir;
        }
        if self.ghost_y < margin + 10.0 && self.ghost_dir.sin() < 0.0 {
            self.ghost_dir = -self.ghost_dir;
        }
        if self.ghost_y > self.arena_h - margin - 10.0 && self.ghost_dir.sin() > 0.0 {
            self.ghost_dir = -self.ghost_dir;
        }

        self.ghost_x += self.ghost_dir.cos() * speed * dt;
        self.ghost_y += self.ghost_dir.sin() * speed * dt;

        self.ghost_x = self.ghost_x.clamp(margin, self.arena_w - margin);
        self.ghost_y = self.ghost_y.clamp(margin, self.arena_h - margin);
    }

    // Particle FX

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

    fn spawn_munch_sparkles(&mut self, cx: f32, cy: f32, color: Rgb) {
        let mut rng = rand::rng();
        for _ in 0..4 {
            let angle = rng.random_range(0.0..2.0 * PI);
            let speed = rng.random_range(10.0..25.0);
            self.particles.push(Particle {
                x: cx,
                y: cy,
                vx: angle.cos() * speed,
                vy: angle.sin() * speed,
                life: 0.35,
                max_life: 0.35,
                color,
                size: 0.8,
            });
        }
    }

    fn update_particles(&mut self, dt: f32) {
        for p in self.particles.iter_mut() {
            p.x += p.vx * dt;
            p.y += p.vy * dt;
            p.life -= dt;
        }
        self.particles.retain(|p| p.life > 0.0);
    }
}

fn steer_towards_target(
    x: &mut f32,
    y: &mut f32,
    dir: &mut f32,
    tx: f32,
    ty: f32,
    speed: f32,
    dt: f32,
    margin: f32,
    arena_w: f32,
    arena_h: f32,
) {
    let dx = tx - *x;
    let dy = ty - *y;
    let target_dir = dy.atan2(dx);

    // Smooth angle turning
    let mut diff = target_dir - *dir;
    while diff > PI {
        diff -= 2.0 * PI;
    }
    while diff < -PI {
        diff += 2.0 * PI;
    }
    let turn_rate = 6.0;
    *dir += diff * (turn_rate * dt).min(1.0);

    *x += dir.cos() * speed * dt;
    *y += dir.sin() * speed * dt;

    // Keep inside arena boundaries with soft bounce
    *x = x.clamp(margin, arena_w - margin);
    *y = y.clamp(margin, arena_h - margin);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simulation_lifecycle() {
        let mut sim = ChaseSimulation::new(100, 50);
        assert_eq!(sim.phase, Phase::ApproachingPellet);

        // Simulate a few seconds
        for _ in 0..100 {
            sim.update(0.05);
        }

        // Draw onto a canvas without panic
        let mut canvas = Canvas::new(50, 25);
        sim.draw(&mut canvas);
    }
}
