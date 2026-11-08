use crate::color::{ColorMode, Rgb};
use std::fmt::Write as FmtWrite;

/// Unicode quadrant block lookup table based on a 4-bit mask:
/// bit 0 (1): Top-Left
/// bit 1 (2): Top-Right
/// bit 2 (4): Bottom-Left
/// bit 3 (8): Bottom-Right
const QUADRANT_CHARS: [char; 16] = [
    ' ',  // 0000: None
    '▘',  // 0001: Top-Left
    '▝',  // 0010: Top-Right
    '▀',  // 0011: Top-Left + Top-Right (Upper Half Block)
    '▖',  // 0100: Bottom-Left
    '▌',  // 0101: Top-Left + Bottom-Left (Left Half Block)
    '▞',  // 0110: Top-Right + Bottom-Left
    '▛',  // 0111: Top-Left + Top-Right + Bottom-Left
    '▗',  // 1000: Bottom-Right
    '▚',  // 1001: Top-Left + Bottom-Right
    '▐',  // 1010: Top-Right + Bottom-Right (Right Half Block)
    '▜',  // 1011: Top-Left + Top-Right + Bottom-Right
    '▄',  // 1100: Bottom-Left + Bottom-Right (Lower Half Block)
    '▙',  // 1101: Top-Left + Bottom-Left + Bottom-Right
    '▟',  // 1110: Top-Right + Bottom-Left + Bottom-Right
    '█',  // 1111: All 4 (Full Block)
];

#[derive(Debug, Clone)]
pub struct Canvas {
    /// Terminal width in character columns
    pub term_width: u16,
    /// Terminal height in character rows
    pub term_height: u16,
    /// High-resolution pixel width (term_width * 2)
    pub width: usize,
    /// High-resolution pixel height (term_height * 2)
    pub height: usize,
    /// Pixel buffer storing RGB colors
    pixels: Vec<Rgb>,
    /// Background default color
    pub bg_color: Rgb,
    /// Render output buffer reused across frames to prevent allocations
    render_buf: String,
}

impl Canvas {
    pub fn new(term_width: u16, term_height: u16) -> Self {
        let width = (term_width as usize) * 2;
        let height = (term_height as usize) * 2;
        let total = width * height;
        let bg_color = Rgb::BLACK;
        Canvas {
            term_width,
            term_height,
            width,
            height,
            pixels: vec![bg_color; total],
            bg_color,
            render_buf: String::with_capacity(total * 4),
        }
    }

    pub fn resize(&mut self, term_width: u16, term_height: u16) {
        self.term_width = term_width;
        self.term_height = term_height;
        self.width = (term_width as usize) * 2;
        self.height = (term_height as usize) * 2;
        let total = self.width * self.height;
        self.pixels.resize(total, self.bg_color);
        self.pixels.fill(self.bg_color);
        self.render_buf.reserve(total * 4);
    }

    pub fn clear(&mut self) {
        self.pixels.fill(self.bg_color);
    }

    #[inline(always)]
    pub fn set_pixel(&mut self, x: i32, y: i32, color: Rgb) {
        if x >= 0 && (x as usize) < self.width && y >= 0 && (y as usize) < self.height {
            let idx = (y as usize) * self.width + (x as usize);
            self.pixels[idx] = color;
        }
    }

    #[inline(always)]
    pub fn get_pixel(&self, x: i32, y: i32) -> Rgb {
        if x >= 0 && (x as usize) < self.width && y >= 0 && (y as usize) < self.height {
            let idx = (y as usize) * self.width + (x as usize);
            self.pixels[idx]
        } else {
            self.bg_color
        }
    }

    /// Blend pixel with alpha (0.0 to 1.0)
    pub fn blend_pixel(&mut self, x: i32, y: i32, color: Rgb, alpha: f32) {
        if alpha <= 0.0 {
            return;
        }
        if alpha >= 1.0 {
            self.set_pixel(x, y, color);
            return;
        }
        let current = self.get_pixel(x, y);
        let blended = current.lerp(color, alpha);
        self.set_pixel(x, y, blended);
    }

    /// Draw a filled rectangle
    #[allow(dead_code)]
    pub fn fill_rect(&mut self, x0: i32, y0: i32, w: i32, h: i32, color: Rgb) {
        for dy in 0..h {
            for dx in 0..w {
                self.set_pixel(x0 + dx, y0 + dy, color);
            }
        }
    }

    /// Draw a filled circle
    #[allow(dead_code)]
    pub fn fill_circle(&mut self, cx: i32, cy: i32, radius: i32, color: Rgb) {
        let r2 = radius * radius;
        for dy in -radius..=radius {
            for dx in -radius..=radius {
                if dx * dx + dy * dy <= r2 {
                    self.set_pixel(cx + dx, cy + dy, color);
                }
            }
        }
    }

    /// Draw an antialiased / smooth filled circle
    pub fn fill_circle_smooth(&mut self, cx: f32, cy: f32, radius: f32, color: Rgb) {
        let min_x = (cx - radius - 1.0).floor() as i32;
        let max_x = (cx + radius + 1.0).ceil() as i32;
        let min_y = (cy - radius - 1.0).floor() as i32;
        let max_y = (cy + radius + 1.0).ceil() as i32;

        for y in min_y..=max_y {
            for x in min_x..=max_x {
                let dx = x as f32 + 0.5 - cx;
                let dy = y as f32 + 0.5 - cy;
                let dist2 = dx * dx + dy * dy;
                if dist2 <= (radius - 0.5) * (radius - 0.5) {
                    self.set_pixel(x, y, color);
                } else if dist2 <= (radius + 0.5) * (radius + 0.5) {
                    let dist = dist2.sqrt();
                    let alpha = ((radius + 0.5 - dist) / 1.0).clamp(0.0, 1.0);
                    self.blend_pixel(x, y, color, alpha);
                }
            }
        }
    }

    /// Render the virtual pixel buffer to a string with ANSI escape codes
    /// using quadrant and half blocks. Optimizes escape codes by only emitting
    /// fg/bg sequences when they change.
    pub fn render_to_string(&mut self, mode: ColorMode) -> &str {
        self.render_buf.clear();

        // Cursor home: move to 1, 1 without clearing entire screen to eliminate flicker
        self.render_buf.push_str("\x1b[H");

        let mut last_fg: Option<Rgb> = None;
        let mut last_bg: Option<Rgb> = None;

        let term_w = self.term_width as usize;
        let term_h = self.term_height as usize;

        for cy in 0..term_h {
            // Position cursor explicitly at row (cy + 1), col 1.
            // This prevents automatic terminal line-wrapping from inserting an extra blank line.
            let _ = write!(self.render_buf, "\x1b[{};1H", cy + 1);

            let py0 = cy * 2;
            let py1 = py0 + 1;

            let row0_offset = py0 * self.width;
            let row1_offset = py1 * self.width;

            for cx in 0..term_w {
                let px0 = cx * 2;
                let px1 = px0 + 1;

                let tl = self.pixels[row0_offset + px0];
                let tr = self.pixels[row0_offset + px1];
                let bl = self.pixels[row1_offset + px0];
                let br = self.pixels[row1_offset + px1];

                // Check uniform cell
                if tl == tr && tr == bl && bl == br {
                    if tl == self.bg_color {
                        // Cell is completely empty/background
                        if last_bg.is_some() || last_fg.is_some() {
                            self.render_buf.push_str("\x1b[0m");
                            last_fg = None;
                            last_bg = None;
                        }
                        self.render_buf.push(' ');
                    } else {
                        // All 4 subpixels same color
                        if last_fg != Some(tl) {
                            tl.write_fg_ansi(&mut self.render_buf, mode);
                            last_fg = Some(tl);
                        }
                        // Use full block
                        self.render_buf.push('█');
                    }
                    continue;
                }

                // Identify the two dominant colors in this 2x2 cell
                let (fg_color, bg_color, mask) = quantize_2x2(tl, tr, bl, br, self.bg_color);

                if mask == 0 {
                    // Empty cell
                    if last_bg != Some(bg_color) {
                        bg_color.write_bg_ansi(&mut self.render_buf, mode);
                        last_bg = Some(bg_color);
                    }
                    self.render_buf.push(' ');
                } else if mask == 15 {
                    // Solid full block
                    if last_fg != Some(fg_color) {
                        fg_color.write_fg_ansi(&mut self.render_buf, mode);
                        last_fg = Some(fg_color);
                    }
                    self.render_buf.push('█');
                } else {
                    // Quadrant / half block
                    if last_fg != Some(fg_color) {
                        fg_color.write_fg_ansi(&mut self.render_buf, mode);
                        last_fg = Some(fg_color);
                    }
                    if last_bg != Some(bg_color) {
                        bg_color.write_bg_ansi(&mut self.render_buf, mode);
                        last_bg = Some(bg_color);
                    }
                    let ch = QUADRANT_CHARS[mask as usize];
                    self.render_buf.push(ch);
                }
            }
        }

        // Final reset at end of frame
        self.render_buf.push_str("\x1b[0m");
        &self.render_buf
    }
}

/// Quantizes 4 subpixels into a foreground color, background color, and 4-bit mask.
/// Bit 0: TL, Bit 1: TR, Bit 2: BL, Bit 3: BR
#[inline]
fn quantize_2x2(tl: Rgb, tr: Rgb, bl: Rgb, br: Rgb, default_bg: Rgb) -> (Rgb, Rgb, u8) {
    let colors = [tl, tr, bl, br];

    // Check if any pixels match the default background
    let mut bg_count = 0;
    for &c in &colors {
        if c == default_bg {
            bg_count += 1;
        }
    }

    let bg_color = if bg_count > 0 {
        default_bg
    } else {
        // Pick the least common or first color as candidate bg
        tl
    };

    // Find the most prominent non-bg color for fg
    let mut fg_candidates: [(Rgb, usize); 4] = [(Rgb::BLACK, 0); 4];
    let mut num_candidates = 0;

    for &c in &colors {
        if c == bg_color {
            continue;
        }
        let mut found = false;
        for i in 0..num_candidates {
            if fg_candidates[i].0 == c {
                fg_candidates[i].1 += 1;
                found = true;
                break;
            }
        }
        if !found {
            fg_candidates[num_candidates] = (c, 1);
            num_candidates += 1;
        }
    }

    let fg_color = if num_candidates > 0 {
        // Pick the one with highest frequency
        let mut best = fg_candidates[0].0;
        let mut max_count = fg_candidates[0].1;
        for i in 1..num_candidates {
            if fg_candidates[i].1 > max_count {
                max_count = fg_candidates[i].1;
                best = fg_candidates[i].0;
            }
        }
        best
    } else {
        tl
    };

    // Form 4-bit mask: bit is 1 if closer to fg_color than bg_color
    let mut mask = 0u8;
    for (i, &c) in colors.iter().enumerate() {
        if c == bg_color {
            // 0
        } else if c == fg_color {
            mask |= 1 << i;
        } else {
            // Distance check
            let d_fg = rgb_dist_sq(c, fg_color);
            let d_bg = rgb_dist_sq(c, bg_color);
            if d_fg <= d_bg {
                mask |= 1 << i;
            }
        }
    }

    (fg_color, bg_color, mask)
}

#[inline]
fn rgb_dist_sq(c1: Rgb, c2: Rgb) -> i32 {
    let dr = c1.r as i32 - c2.r as i32;
    let dg = c1.g as i32 - c2.g as i32;
    let db = c1.b as i32 - c2.b as i32;
    dr * dr + dg * dg + db * db
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_canvas_quadrant_masks() {
        let mut canvas = Canvas::new(2, 2);
        let yellow = Rgb::new(255, 255, 0);

        // Top-left pixel only -> '▘'
        canvas.set_pixel(0, 0, yellow);
        let rendered = canvas.render_to_string(ColorMode::Truecolor);
        assert!(rendered.contains('▘'));

        // Top half -> '▀'
        canvas.clear();
        canvas.set_pixel(0, 0, yellow);
        canvas.set_pixel(1, 0, yellow);
        let rendered = canvas.render_to_string(ColorMode::Truecolor);
        assert!(rendered.contains('▀'));

        // Bottom half -> '▄'
        canvas.clear();
        canvas.set_pixel(0, 1, yellow);
        canvas.set_pixel(1, 1, yellow);
        let rendered = canvas.render_to_string(ColorMode::Truecolor);
        assert!(rendered.contains('▄'));
    }
}
