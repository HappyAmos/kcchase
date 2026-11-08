use crate::canvas::Canvas;
use crate::color::Rgb;
use std::f32::consts::PI;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GhostState {
    Patrolling,
    Vulnerable,
    Fading { time_remaining: f32 },
    Hunting,
}

#[derive(Debug, Clone, Copy)]
pub struct GhostConfig {
    pub base_color: Rgb,
    pub eye_color: Rgb,
    pub pupil_color: Rgb,
}

impl GhostConfig {
    pub fn red_bit() -> Self {
        GhostConfig {
            base_color: Rgb::new(255, 30, 30),
            eye_color: Rgb::WHITE,
            pupil_color: Rgb::new(0, 40, 200),
        }
    }

    pub fn purple_byte() -> Self {
        GhostConfig {
            base_color: Rgb::new(180, 50, 240),
            eye_color: Rgb::WHITE,
            pupil_color: Rgb::new(0, 40, 200),
        }
    }

    pub fn green_middie() -> Self {
        GhostConfig {
            base_color: Rgb::new(40, 220, 80),
            eye_color: Rgb::WHITE,
            pupil_color: Rgb::new(0, 40, 200),
        }
    }

    pub fn orange_muncher() -> Self {
        GhostConfig {
            base_color: Rgb::new(255, 140, 20),
            eye_color: Rgb::WHITE,
            pupil_color: Rgb::new(0, 40, 200),
        }
    }
}

/// Draw KC Munchkin at (cx, cy) with given radius, direction angle (radians), mouth opening (0.0 to 1.0),
/// whether super-charged, whether terrified (chased), and animation time.
pub fn draw_kc_munchkin(
    canvas: &mut Canvas,
    cx: f32,
    cy: f32,
    radius: f32,
    direction: f32,
    mouth_open: f32,
    is_supercharged: bool,
    is_terrified: bool,
    anim_time: f32,
) {
    let kc_yellow = Rgb::new(255, 230, 20);
    let aura_glow = Rgb::new(255, 100, 220);

    // Draw glowing aura if supercharged
    if is_supercharged {
        let aura_radius = radius + 2.5 + 1.2 * (anim_time * 8.0).sin();
        let ring_pts = 16;
        for i in 0..ring_pts {
            let angle = (i as f32 / ring_pts as f32) * 2.0 * PI + anim_time * 4.0;
            let px = cx + angle.cos() * aura_radius;
            let py = cy + angle.sin() * aura_radius;
            let sparkle_color = if i % 2 == 0 {
                aura_glow
            } else {
                Rgb::new(255, 255, 180)
            };
            canvas.fill_circle_smooth(px, py, 1.0, sparkle_color);
        }
    }

    // Horns / antennae on top of head (classic Odyssey 2 K.C. Munchkin hallmark!)
    let horn_len = radius * 0.75;
    let horn_spread = radius * 0.55;
    let horn_color = if is_supercharged {
        Rgb::new(255, 120, 240)
    } else {
        Rgb::new(240, 190, 10)
    };

    // Left horn
    let lx0 = cx - horn_spread;
    let ly0 = cy - radius * 0.7;
    let lx1 = cx - horn_spread * 1.35;
    let ly1 = cy - radius - horn_len;
    draw_line_smooth(canvas, lx0, ly0, lx1, ly1, horn_color, (radius * 0.22).max(1.0));
    canvas.fill_circle_smooth(lx1, ly1, (radius * 0.22).max(1.2), Rgb::new(255, 255, 255));

    // Right horn
    let rx0 = cx + horn_spread;
    let ry0 = cy - radius * 0.7;
    let rx1 = cx + horn_spread * 1.35;
    let ry1 = cy - radius - horn_len;
    draw_line_smooth(canvas, rx0, ry0, rx1, ry1, horn_color, (radius * 0.22).max(1.0));
    canvas.fill_circle_smooth(rx1, ry1, (radius * 0.22).max(1.2), Rgb::new(255, 255, 255));

    // Munchkin Body: Round sphere with wedge mouth cutout
    let half_mouth_angle = (mouth_open * 0.75).clamp(0.02, 0.95);
    let min_x = (cx - radius - 2.0).floor() as i32;
    let max_x = (cx + radius + 2.0).ceil() as i32;
    let min_y = (cy - radius - 2.0).floor() as i32;
    let max_y = (cy + radius + 2.0).ceil() as i32;

    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let dx = x as f32 + 0.5 - cx;
            let dy = y as f32 + 0.5 - cy;
            let dist2 = dx * dx + dy * dy;

            if dist2 <= (radius + 0.6) * (radius + 0.6) {
                // Check if inside mouth wedge
                let angle_to_center = dy.atan2(dx);
                let mut diff = angle_to_center - direction;
                while diff > PI {
                    diff -= 2.0 * PI;
                }
                while diff < -PI {
                    diff += 2.0 * PI;
                }

                if mouth_open > 0.08 && diff.abs() < half_mouth_angle {
                    // Inside open mouth: empty/background
                    continue;
                }

                // Antialiased edge
                let dist = dist2.sqrt();
                if dist <= radius - 0.5 {
                    canvas.set_pixel(x, y, kc_yellow);
                } else if dist <= radius + 0.5 {
                    let alpha = ((radius + 0.5 - dist) / 1.0).clamp(0.0, 1.0);
                    canvas.blend_pixel(x, y, kc_yellow, alpha);
                }
            }
        }
    }

    // Eyes: KC has big expressive eyes
    let eye_offset_x = (direction.cos() * radius * 0.35) - (direction.sin() * radius * 0.35);
    let eye_offset_y = (direction.sin() * radius * 0.35) + (direction.cos() * radius * 0.35);
    let eye_radius = (radius * 0.3).max(1.2);

    // Primary eye
    let ex1 = cx + eye_offset_x * 0.5 + direction.cos() * radius * 0.2;
    let ey1 = cy - radius * 0.25 + eye_offset_y * 0.3;

    canvas.fill_circle_smooth(ex1, ey1, eye_radius, Rgb::WHITE);

    // Pupil
    let pupil_radius = (eye_radius * 0.45).max(0.7);
    let pupil_dx = if is_terrified {
        -direction.cos() * eye_radius * 0.4
    } else {
        direction.cos() * eye_radius * 0.45
    };
    let pupil_dy = if is_terrified {
        -direction.sin() * eye_radius * 0.4
    } else {
        direction.sin() * eye_radius * 0.45
    };
    let pupil_color = if is_terrified {
        Rgb::new(20, 20, 80)
    } else {
        Rgb::new(0, 0, 120)
    };
    canvas.fill_circle_smooth(ex1 + pupil_dx, ey1 + pupil_dy, pupil_radius, pupil_color);

    // If terrified, draw sweat drops flinging behind him!
    if is_terrified {
        let sweat_angle = direction + PI + 0.3 * (anim_time * 12.0).sin();
        let sweat_dist = radius * 1.5;
        let sx = cx + sweat_angle.cos() * sweat_dist;
        let sy = cy + sweat_angle.sin() * sweat_dist;
        canvas.fill_circle_smooth(sx, sy, (radius * 0.25).max(1.0), Rgb::new(100, 200, 255));
    }
}

/// Draw Ghost / Muncher at (cx, cy) with given state, animation time, direction and config.
pub fn draw_ghost(
    canvas: &mut Canvas,
    cx: f32,
    cy: f32,
    radius: f32,
    direction: f32,
    state: GhostState,
    config: &GhostConfig,
    anim_time: f32,
) {
    let (body_color, eye_color, pupil_color) = match state {
        GhostState::Patrolling | GhostState::Hunting => {
            (config.base_color, config.eye_color, config.pupil_color)
        }
        GhostState::Vulnerable => (
            Rgb::new(30, 60, 220),
            Rgb::new(240, 240, 255),
            Rgb::new(255, 140, 140),
        ),
        GhostState::Fading { time_remaining: _ } => {
            // Rapid blinking between vulnerable blue and bright flashing white/cyan
            let flash_rate = 14.0;
            let flash = (anim_time * flash_rate).sin() > 0.0;
            if flash {
                (Rgb::new(240, 240, 255), Rgb::new(255, 40, 40), Rgb::new(0, 0, 255))
            } else {
                (Rgb::new(30, 60, 220), Rgb::new(240, 240, 255), Rgb::new(255, 140, 140))
            }
        }
    };

    let height = radius * 2.1;
    let dome_center_y = cy - height * 0.15;
    let skirt_bottom_y = cy + height * 0.5;

    let min_x = (cx - radius - 2.0).floor() as i32;
    let max_x = (cx + radius + 2.0).ceil() as i32;
    let min_y = (dome_center_y - radius - 2.0).floor() as i32;
    let max_y = (skirt_bottom_y + 3.0).ceil() as i32;

    // Draw ghost body (dome top + wavy skirt bottom)
    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let px = x as f32 + 0.5;
            let py = y as f32 + 0.5;
            let dx = px - cx;

            if dx.abs() > radius + 0.5 {
                continue;
            }

            let in_body = if py < dome_center_y {
                // Top dome
                let dy = py - dome_center_y;
                let d2 = dx * dx + dy * dy;
                d2 <= (radius + 0.5) * (radius + 0.5)
            } else {
                // Skirt with wavy ripple
                // Wave formula: 3 ripple feet
                let normalized_x = (dx / radius).clamp(-1.0, 1.0);
                let wave_freq = 3.0 * PI;
                let wave_offset = anim_time * 8.0;
                let ripple = (normalized_x * wave_freq + wave_offset).cos() * (radius * 0.28);
                let bottom_y = skirt_bottom_y + ripple;
                py <= bottom_y && dx.abs() <= radius
            };

            if in_body {
                canvas.set_pixel(x, y, body_color);
            }
        }
    }

    // Ghost Eyes
    let eye_spacing = radius * 0.45;
    let eye_y = dome_center_y - radius * 0.05;
    let eye_rx = (radius * 0.32).max(1.5);
    let eye_ry = (radius * 0.42).max(1.8);

    let left_eye_x = cx - eye_spacing;
    let right_eye_x = cx + eye_spacing;

    // Left and Right eyes
    draw_ellipse_filled(canvas, left_eye_x, eye_y, eye_rx, eye_ry, eye_color);
    draw_ellipse_filled(canvas, right_eye_x, eye_y, eye_rx, eye_ry, eye_color);

    // Pupils: look towards direction of travel
    let p_offset_x = (direction.cos() * eye_rx * 0.5).clamp(-eye_rx * 0.5, eye_rx * 0.5);
    let p_offset_y = (direction.sin() * eye_ry * 0.4).clamp(-eye_ry * 0.4, eye_ry * 0.4);
    let p_rad = (eye_rx * 0.45).max(0.8);

    canvas.fill_circle_smooth(left_eye_x + p_offset_x, eye_y + p_offset_y, p_rad, pupil_color);
    canvas.fill_circle_smooth(right_eye_x + p_offset_x, eye_y + p_offset_y, p_rad, pupil_color);

    // If vulnerable, draw classic squiggly scared mouth!
    if matches!(state, GhostState::Vulnerable | GhostState::Fading { .. }) {
        let mouth_y = cy + radius * 0.3;
        let mouth_w = radius * 0.8;
        let segments = 5;
        let mut prev_x = cx - mouth_w * 0.5;
        let mut prev_y = mouth_y;
        for s in 1..=segments {
            let next_x = cx - mouth_w * 0.5 + (s as f32 / segments as f32) * mouth_w;
            let dy = if s % 2 == 1 { -1.2 } else { 1.2 };
            let next_y = mouth_y + dy;
            draw_line_smooth(canvas, prev_x, prev_y, next_x, next_y, Rgb::new(255, 180, 180), 0.8);
            prev_x = next_x;
            prev_y = next_y;
        }
    }
}

/// Draw a glowing Super Pellet with pulsing corona and sparkle rings
pub fn draw_super_pellet(canvas: &mut Canvas, cx: f32, cy: f32, radius: f32, anim_time: f32) {
    let pulse = ((anim_time * 5.0).sin() * 0.35) + 1.0; // 0.65 to 1.35
    let current_radius = radius * pulse;

    // Glowing outer corona
    let corona_radius = current_radius + 2.0;
    canvas.fill_circle_smooth(cx, cy, corona_radius, Rgb::new(255, 160, 40));

    // Core bright orb
    canvas.fill_circle_smooth(cx, cy, current_radius, Rgb::new(255, 255, 150));
    // Hot center
    canvas.fill_circle_smooth(cx, cy, current_radius * 0.5, Rgb::WHITE);

    // Orbiting sparkles
    let num_sparkles = 4;
    for i in 0..num_sparkles {
        let angle = (i as f32 / num_sparkles as f32) * 2.0 * PI + anim_time * 3.5;
        let dist = corona_radius + 2.0 + 1.0 * (anim_time * 8.0 + i as f32).sin();
        let sx = cx + angle.cos() * dist;
        let sy = cy + angle.sin() * dist;
        canvas.fill_circle_smooth(sx, sy, 0.8, Rgb::new(255, 240, 200));
    }
}

/// Draw small regular maze pellet
pub fn draw_pellet(canvas: &mut Canvas, cx: f32, cy: f32, anim_time: f32) {
    let sparkle = (anim_time * 2.0 + (cx * 0.1) + (cy * 0.1)).sin();
    let col = if sparkle > 0.6 {
        Rgb::new(255, 255, 230)
    } else {
        Rgb::new(255, 180, 100)
    };
    canvas.fill_circle_smooth(cx, cy, 1.0, col);
}

/// Helper to draw a smooth antialiased line
fn draw_line_smooth(
    canvas: &mut Canvas,
    x0: f32,
    y0: f32,
    x1: f32,
    y1: f32,
    color: Rgb,
    thickness: f32,
) {
    let dx = x1 - x0;
    let dy = y1 - y0;
    let len = (dx * dx + dy * dy).sqrt();
    if len < 0.1 {
        return;
    }
    let steps = (len * 2.0).ceil() as usize;
    let half_thick = (thickness * 0.5).max(0.6);
    for i in 0..=steps {
        let t = i as f32 / steps as f32;
        let px = x0 + dx * t;
        let py = y0 + dy * t;
        canvas.fill_circle_smooth(px, py, half_thick, color);
    }
}

/// Helper to draw an antialiased filled ellipse
fn draw_ellipse_filled(canvas: &mut Canvas, cx: f32, cy: f32, rx: f32, ry: f32, color: Rgb) {
    let min_x = (cx - rx - 1.0).floor() as i32;
    let max_x = (cx + rx + 1.0).ceil() as i32;
    let min_y = (cy - ry - 1.0).floor() as i32;
    let max_y = (cy + ry + 1.0).ceil() as i32;

    let rx2 = rx * rx;
    let ry2 = ry * ry;

    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let dx = x as f32 + 0.5 - cx;
            let dy = y as f32 + 0.5 - cy;
            let norm = (dx * dx) / rx2 + (dy * dy) / ry2;
            if norm <= 1.0 {
                canvas.set_pixel(x, y, color);
            }
        }
    }
}
