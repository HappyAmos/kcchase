use std::env;
use std::fmt::Write as FmtWrite;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorMode {
    Truecolor,
    Ansi256,
    Ansi16,
    Ansi8,
}

impl ColorMode {
    /// Detect the terminal color capabilities from environment variables.
    pub fn detect() -> Self {
        // Check COLORTERM first
        if let Ok(colorterm) = env::var("COLORTERM") {
            let lower = colorterm.to_lowercase();
            if lower == "truecolor" || lower == "24bit" {
                return ColorMode::Truecolor;
            }
        }

        // Check TERM
        if let Ok(term) = env::var("TERM") {
            let lower = term.to_lowercase();
            if lower.contains("truecolor") || lower.contains("24bit") || lower.contains("direct") {
                return ColorMode::Truecolor;
            }
            if lower.contains("256color") || lower.contains("256") {
                return ColorMode::Ansi256;
            }
            if lower.contains("16color") {
                return ColorMode::Ansi16;
            }
            if lower == "linux" || lower.contains("8color") || lower == "vt100" {
                return ColorMode::Ansi8;
            }
            if lower.contains("color") || lower.contains("xterm") || lower.contains("screen") || lower.contains("tmux") {
                // Modern terminals commonly support at least 256 colors
                return ColorMode::Ansi256;
            }
        }

        // Conservative fallback
        ColorMode::Ansi16
    }

    pub fn from_str_opt(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "truecolor" | "24bit" | "rgb" => Some(ColorMode::Truecolor),
            "256" | "256color" | "ansi256" => Some(ColorMode::Ansi256),
            "16" | "16color" | "ansi16" => Some(ColorMode::Ansi16),
            "8" | "8color" | "ansi8" => Some(ColorMode::Ansi8),
            _ => None,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            ColorMode::Truecolor => "Truecolor (24-bit)",
            ColorMode::Ansi256 => "256 Color (8-bit)",
            ColorMode::Ansi16 => "16 Color (4-bit ANSI)",
            ColorMode::Ansi8 => "8 Color (3-bit ANSI)",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Rgb {
    pub const BLACK: Rgb = Rgb { r: 0, g: 0, b: 0 };
    pub const WHITE: Rgb = Rgb { r: 255, g: 255, b: 255 };

    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Rgb { r, g, b }
    }

    /// Linear interpolation between two RGB colors
    pub fn lerp(self, other: Rgb, t: f32) -> Rgb {
        let t = t.clamp(0.0, 1.0);
        let r = (self.r as f32 + (other.r as f32 - self.r as f32) * t).round() as u8;
        let g = (self.g as f32 + (other.g as f32 - self.g as f32) * t).round() as u8;
        let b = (self.b as f32 + (other.b as f32 - self.b as f32) * t).round() as u8;
        Rgb { r, g, b }
    }

    /// Color distance metric (weighted Euclidean for human eye sensitivity)
    fn distance_sq(self, other: Rgb) -> u32 {
        let r_mean = ((self.r as i32) + (other.r as i32)) / 2;
        let dr = (self.r as i32) - (other.r as i32);
        let dg = (self.g as i32) - (other.g as i32);
        let db = (self.b as i32) - (other.b as i32);
        let weight_r = 2 + (r_mean / 256);
        let weight_g = 4;
        let weight_b = 2 + ((255 - r_mean) / 256);
        (weight_r * dr * dr + weight_g * dg * dg + weight_b * db * db) as u32
    }

    /// Convert to 256-color palette index
    pub fn to_ansi256(self) -> u8 {
        // Check standard 16 colors first for pure matches
        // Check grayscale ramp (232..=255)
        let is_gray = (self.r as i16 - self.g as i16).abs() < 8
            && (self.g as i16 - self.b as i16).abs() < 8
            && (self.r as i16 - self.b as i16).abs() < 8;

        if is_gray {
            let avg = ((self.r as u16 + self.g as u16 + self.b as u16) / 3) as u8;
            if avg < 8 {
                return 16; // black in 6x6x6 cube
            } else if avg > 248 {
                return 231; // white in 6x6x6 cube
            } else {
                let gray_idx = ((avg as u16 - 8) * 24 / 240) as u8;
                return 232 + gray_idx.min(23);
            }
        }

        // 6x6x6 color cube: 16 + 36*r + 6*g + b where values are 0..=5
        let r_idx = (self.r as u16 * 5 + 127) / 255;
        let g_idx = (self.g as u16 * 5 + 127) / 255;
        let b_idx = (self.b as u16 * 5 + 127) / 255;
        16 + (36 * r_idx + 6 * g_idx + b_idx) as u8
    }

    /// Convert to 16 ANSI color palette index (0..15)
    pub fn to_ansi16(self) -> u8 {
        const ANSI16_COLORS: [Rgb; 16] = [
            Rgb::new(0, 0, 0),       // 0: Black
            Rgb::new(170, 0, 0),     // 1: Dark Red
            Rgb::new(0, 170, 0),     // 2: Dark Green
            Rgb::new(170, 85, 0),    // 3: Dark Yellow
            Rgb::new(0, 0, 170),     // 4: Dark Blue
            Rgb::new(170, 0, 170),   // 5: Dark Magenta
            Rgb::new(0, 170, 170),   // 6: Dark Cyan
            Rgb::new(170, 170, 170), // 7: Light Gray
            Rgb::new(85, 85, 85),    // 8: Dark Gray
            Rgb::new(255, 85, 85),   // 9: Bright Red
            Rgb::new(85, 255, 85),   // 10: Bright Green
            Rgb::new(255, 255, 85),  // 11: Bright Yellow
            Rgb::new(85, 85, 255),   // 12: Bright Blue
            Rgb::new(255, 85, 255),  // 13: Bright Magenta
            Rgb::new(85, 255, 255),  // 14: Bright Cyan
            Rgb::new(255, 255, 255), // 15: Bright White
        ];

        let mut best_idx = 0;
        let mut min_dist = u32::MAX;
        for (i, &palette_color) in ANSI16_COLORS.iter().enumerate() {
            let dist = self.distance_sq(palette_color);
            if dist < min_dist {
                min_dist = dist;
                best_idx = i as u8;
            }
        }
        best_idx
    }

    /// Convert to 8 ANSI color palette index (0..7)
    pub fn to_ansi8(self) -> u8 {
        const ANSI8_COLORS: [Rgb; 8] = [
            Rgb::new(0, 0, 0),       // 0: Black
            Rgb::new(200, 0, 0),     // 1: Red
            Rgb::new(0, 200, 0),     // 2: Green
            Rgb::new(200, 200, 0),   // 3: Yellow
            Rgb::new(0, 0, 200),     // 4: Blue
            Rgb::new(200, 0, 200),   // 5: Magenta
            Rgb::new(0, 200, 200),   // 6: Cyan
            Rgb::new(200, 200, 200), // 7: White
        ];

        let mut best_idx = 0;
        let mut min_dist = u32::MAX;
        for (i, &palette_color) in ANSI8_COLORS.iter().enumerate() {
            let dist = self.distance_sq(palette_color);
            if dist < min_dist {
                min_dist = dist;
                best_idx = i as u8;
            }
        }
        best_idx
    }

    /// Write foreground ANSI escape code to a string buffer
    pub fn write_fg_ansi(&self, out: &mut String, mode: ColorMode) {
        match mode {
            ColorMode::Truecolor => {
                let _ = write!(out, "\x1b[38;2;{};{};{}m", self.r, self.g, self.b);
            }
            ColorMode::Ansi256 => {
                let idx = self.to_ansi256();
                let _ = write!(out, "\x1b[38;5;{}m", idx);
            }
            ColorMode::Ansi16 => {
                let idx = self.to_ansi16();
                if idx < 8 {
                    let _ = write!(out, "\x1b[{}m", 30 + idx);
                } else {
                    let _ = write!(out, "\x1b[{}m", 90 + (idx - 8));
                }
            }
            ColorMode::Ansi8 => {
                let idx = self.to_ansi8();
                let _ = write!(out, "\x1b[{}m", 30 + idx);
            }
        }
    }

    /// Write background ANSI escape code to a string buffer
    pub fn write_bg_ansi(&self, out: &mut String, mode: ColorMode) {
        match mode {
            ColorMode::Truecolor => {
                let _ = write!(out, "\x1b[48;2;{};{};{}m", self.r, self.g, self.b);
            }
            ColorMode::Ansi256 => {
                let idx = self.to_ansi256();
                let _ = write!(out, "\x1b[48;5;{}m", idx);
            }
            ColorMode::Ansi16 => {
                let idx = self.to_ansi16();
                if idx < 8 {
                    let _ = write!(out, "\x1b[{}m", 40 + idx);
                } else {
                    let _ = write!(out, "\x1b[{}m", 100 + (idx - 8));
                }
            }
            ColorMode::Ansi8 => {
                let idx = self.to_ansi8();
                let _ = write!(out, "\x1b[{}m", 40 + idx);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_conversions() {
        let yellow = Rgb::new(255, 255, 0);
        let dark_blue = Rgb::new(0, 0, 170);
        let bright_blue = Rgb::new(85, 85, 255);
        let bright_red = Rgb::new(255, 85, 85);
        let dark_red = Rgb::new(170, 0, 0);

        assert_eq!(yellow.to_ansi16(), 11);      // Bright yellow
        assert_eq!(bright_red.to_ansi16(), 9);   // Bright red
        assert_eq!(dark_red.to_ansi16(), 1);     // Dark red
        assert_eq!(bright_blue.to_ansi16(), 12); // Bright blue
        assert_eq!(dark_blue.to_ansi16(), 4);    // Dark blue

        assert_eq!(yellow.to_ansi8(), 3);        // Yellow
        assert_eq!(dark_red.to_ansi8(), 1);      // Red
        assert_eq!(dark_blue.to_ansi8(), 4);     // Blue

        let mut buf = String::new();
        yellow.write_fg_ansi(&mut buf, ColorMode::Truecolor);
        assert_eq!(buf, "\x1b[38;2;255;255;0m");

        let mut buf2 = String::new();
        yellow.write_fg_ansi(&mut buf2, ColorMode::Ansi8);
        assert_eq!(buf2, "\x1b[33m");
    }
}
