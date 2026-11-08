use crate::canvas::Canvas;
use crate::color::Rgb;
use rand::RngExt;
use std::collections::VecDeque;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tile {
    Wall,
    Path,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    pub const ALL: [Direction; 4] = [
        Direction::Up,
        Direction::Right,
        Direction::Down,
        Direction::Left,
    ];

    pub fn dx(&self) -> i32 {
        match self {
            Direction::Left => -1,
            Direction::Right => 1,
            _ => 0,
        }
    }

    pub fn dy(&self) -> i32 {
        match self {
            Direction::Up => -1,
            Direction::Down => 1,
            _ => 0,
        }
    }

    pub fn opposite(&self) -> Direction {
        match self {
            Direction::Up => Direction::Down,
            Direction::Down => Direction::Up,
            Direction::Left => Direction::Right,
            Direction::Right => Direction::Left,
        }
    }

    pub fn to_angle(&self) -> f32 {
        match self {
            Direction::Right => 0.0,
            Direction::Down => std::f32::consts::FRAC_PI_2,
            Direction::Left => std::f32::consts::PI,
            Direction::Up => -std::f32::consts::FRAC_PI_2,
        }
    }
}

pub struct Maze {
    pub cols: usize,
    pub rows: usize,
    pub tiles: Vec<Tile>,
    pub cell_w: f32,
    pub cell_h: f32,
    pub offset_x: f32,
    pub offset_y: f32,
    pub wall_color: Rgb,
    pub wall_glow: Rgb,
    pub warp_row: Option<usize>,
}

impl Maze {
    /// Generate a fresh random arcade maze fitted to the canvas dimensions
    pub fn generate(canvas_w: usize, canvas_h: usize) -> Self {
        let mut rng = rand::rng();

        // Target corridor size in sub-pixels (~12 to 20 px)
        let min_dim = canvas_w.min(canvas_h);
        let desired_cell_size = (min_dim as f32 / 16.0).clamp(10.0, 24.0);

        // Force odd number of columns and rows for clean classic maze walls
        let mut cols = (canvas_w as f32 / desired_cell_size).round() as usize;
        let mut rows = (canvas_h as f32 / desired_cell_size).round() as usize;

        if cols % 2 == 0 {
            cols += 1;
        }
        if rows % 2 == 0 {
            rows += 1;
        }
        cols = cols.clamp(13, 41);
        rows = rows.clamp(9, 29);

        let cell_w = canvas_w as f32 / cols as f32;
        let cell_h = canvas_h as f32 / rows as f32;

        let mut tiles = vec![Tile::Wall; cols * rows];

        // Pick retro neon wall color scheme per maze
        let (wall_color, wall_glow) = match rng.random_range(0..6) {
            0 => (Rgb::new(40, 110, 255), Rgb::new(120, 180, 255)), // Odyssey blue
            1 => (Rgb::new(230, 40, 180), Rgb::new(255, 140, 220)), // Hot magenta
            2 => (Rgb::new(20, 220, 120), Rgb::new(140, 255, 180)), // Alien green
            3 => (Rgb::new(240, 170, 20), Rgb::new(255, 230, 100)), // Gold amber
            4 => (Rgb::new(140, 60, 240), Rgb::new(200, 150, 255)), // Purple byte
            _ => (Rgb::new(0, 210, 240), Rgb::new(160, 245, 255)),  // Cyan
        };

        let half_cols = (cols + 1) / 2;

        // Step 1: Initialize paths on odd grid coordinates in the left half
        for r in (1..rows - 1).step_by(2) {
            for c in (1..half_cols).step_by(2) {
                tiles[r * cols + c] = Tile::Path;
            }
        }

        // Step 2: Carve random connecting corridors with left-right symmetry
        // Using randomized Kruskal's / Prim's style carving on the odd grid
        let num_nodes_x = (half_cols - 1) / 2;
        let num_nodes_y = (rows - 1) / 2;

        // List of all internal edges between adjacent path nodes in left half
        let mut edges = Vec::new();
        for ny in 0..num_nodes_y {
            for nx in 0..num_nodes_x {
                let c = 1 + nx * 2;
                let r = 1 + ny * 2;
                // Horizontal edge to right
                if c + 2 < cols - 1 {
                    edges.push((c, r, c + 1, r, c + 2, r));
                }
                // Vertical edge down
                if r + 2 < rows - 1 {
                    edges.push((c, r, c, r + 1, c, r + 2));
                }
            }
        }

        // Shuffle edges
        for i in (1..edges.len()).rev() {
            let j = rng.random_range(0..=i);
            edges.swap(i, j);
        }

        // Carve edges ensuring connectivity
        for (c1, r1, mid_c, mid_r, _c2, _r2) in edges {
            // Also occasionally add cycles (arcade mazes have loops, not strict trees!)
            let should_carve = rng.random_bool(0.72);
            if should_carve {
                tiles[r1 * cols + c1] = Tile::Path;
                tiles[mid_r * cols + mid_c] = Tile::Path;
            }
        }

        // Mirror left half to right half to create arcade symmetry!
        for r in 1..rows - 1 {
            for c in 1..half_cols {
                let sym_c = cols - 1 - c;
                tiles[r * cols + sym_c] = tiles[r * cols + c];
            }
        }

        // Open center vertical column connections
        let mid_c = cols / 2;
        for r in (1..rows - 1).step_by(2) {
            tiles[r * cols + mid_c] = Tile::Path;
            if rng.random_bool(0.6) && r + 1 < rows - 1 {
                tiles[(r + 1) * cols + mid_c] = Tile::Path;
            }
        }

        // Clear a central chamber or hub
        let cr = rows / 2;
        for dr in -1..=1 {
            for dc in -1..=1 {
                let r = (cr as i32 + dr) as usize;
                let c = (mid_c as i32 + dc) as usize;
                if r > 0 && r < rows - 1 && c > 0 && c < cols - 1 {
                    tiles[r * cols + c] = Tile::Path;
                }
            }
        }

        // Add side warp tunnels on a middle row!
        let warp_row = rows / 2;
        tiles[warp_row * cols + 0] = Tile::Path;
        tiles[warp_row * cols + 1] = Tile::Path;
        tiles[warp_row * cols + (cols - 2)] = Tile::Path;
        tiles[warp_row * cols + (cols - 1)] = Tile::Path;

        // Ensure overall connectivity using flood fill from center
        ensure_connected(&mut tiles, cols, rows, mid_c, cr);

        Maze {
            cols,
            rows,
            tiles,
            cell_w,
            cell_h,
            offset_x: 0.0,
            offset_y: 0.0,
            wall_color,
            wall_glow,
            warp_row: Some(warp_row),
        }
    }

    pub fn is_passable(&self, c: i32, r: i32) -> bool {
        // Horizontal tunnel wrap
        if let Some(wr) = self.warp_row {
            if r == wr as i32 {
                if c < 0 || c >= self.cols as i32 {
                    return true;
                }
            }
        }
        if c < 0 || c >= self.cols as i32 || r < 0 || r >= self.rows as i32 {
            return false;
        }
        self.tiles[(r as usize) * self.cols + (c as usize)] == Tile::Path
    }

    /// Convert tile grid coordinate to pixel center
    pub fn tile_to_pixel(&self, col: f32, row: f32) -> (f32, f32) {
        let x = self.offset_x + (col + 0.5) * self.cell_w;
        let y = self.offset_y + (row + 0.5) * self.cell_h;
        (x, y)
    }

    /// Convert pixel coordinate to tile grid coordinate
    #[allow(dead_code)]
    pub fn pixel_to_tile(&self, x: f32, y: f32) -> (f32, f32) {
        let col = (x - self.offset_x) / self.cell_w - 0.5;
        let row = (y - self.offset_y) / self.cell_h - 0.5;
        (col, row)
    }

    /// Find valid directions to move from a tile position
    pub fn available_directions(&self, c: i32, r: i32) -> Vec<Direction> {
        let mut dirs = Vec::new();
        for &d in &Direction::ALL {
            let nc = c + d.dx();
            let nr = r + d.dy();
            if self.is_passable(nc, nr) {
                dirs.push(d);
            }
        }
        dirs
    }

    /// Breadth-First-Search next step from (sc, sr) towards target (tc, tr)
    pub fn bfs_next_direction(&self, sc: i32, sr: i32, tc: i32, tr: i32) -> Option<Direction> {
        if sc == tc && sr == tr {
            return None;
        }

        let mut queue = VecDeque::new();
        let total = self.cols * self.rows;
        let mut visited = vec![false; total];
        let mut first_step = vec![None; total];

        let start_idx = (sr as usize) * self.cols + (sc as usize);
        visited[start_idx] = true;

        for &d in &self.available_directions(sc, sr) {
            let mut nc = sc + d.dx();
            let nr = sr + d.dy();
            // Handle wrap
            if nc < 0 {
                nc = self.cols as i32 - 1;
            } else if nc >= self.cols as i32 {
                nc = 0;
            }
            if nc >= 0 && nc < self.cols as i32 && nr >= 0 && nr < self.rows as i32 {
                let nidx = (nr as usize) * self.cols + (nc as usize);
                visited[nidx] = true;
                first_step[nidx] = Some(d);
                queue.push_back((nc, nr));
            }
        }

        while let Some((c, r)) = queue.pop_front() {
            let idx = (r as usize) * self.cols + (c as usize);
            if c == tc && r == tr {
                return first_step[idx];
            }

            let initial_dir = first_step[idx];

            for &d in &self.available_directions(c, r) {
                let mut nc = c + d.dx();
                let nr = r + d.dy();
                if nc < 0 {
                    nc = self.cols as i32 - 1;
                } else if nc >= self.cols as i32 {
                    nc = 0;
                }
                if nc >= 0 && nc < self.cols as i32 && nr >= 0 && nr < self.rows as i32 {
                    let nidx = (nr as usize) * self.cols + (nc as usize);
                    if !visited[nidx] {
                        visited[nidx] = true;
                        first_step[nidx] = initial_dir;
                        queue.push_back((nc, nr));
                    }
                }
            }
        }

        None
    }

    /// Pick the direction at an intersection that maximizes distance to the chaser (evasion)
    pub fn flee_direction(&self, sc: i32, sr: i32, chaser_c: i32, chaser_r: i32, current_dir: Direction) -> Direction {
        let available = self.available_directions(sc, sr);
        if available.is_empty() {
            return current_dir;
        }

        // Avoid turning back directly into chaser unless dead end
        let candidates: Vec<Direction> = if available.len() > 1 {
            available.into_iter().filter(|&d| d != current_dir.opposite()).collect()
        } else {
            available
        };

        let mut best_dir = candidates[0];
        let mut max_dist = -1.0;

        for &d in &candidates {
            let nc = sc + d.dx();
            let nr = sr + d.dy();
            let dc = nc - chaser_c;
            let dr = nr - chaser_r;
            let dist = (dc * dc + dr * dr) as f32;
            if dist > max_dist {
                max_dist = dist;
                best_dir = d;
            }
        }

        best_dir
    }

    /// Draw the maze walls with neon arcade borders onto the canvas
    pub fn draw(&self, canvas: &mut Canvas, anim_time: f32) {
        // Dynamic wall glow pulse
        let pulse = (anim_time * 2.0).sin() * 0.15 + 0.85;
        let glow = self.wall_glow.lerp(self.wall_color, 1.0 - pulse);

        for r in 0..self.rows {
            for c in 0..self.cols {
                if self.tiles[r * self.cols + c] == Tile::Wall {
                    let px0 = (c as f32 * self.cell_w).floor() as i32;
                    let py0 = (r as f32 * self.cell_h).floor() as i32;
                    let px1 = ((c as f32 + 1.0) * self.cell_w).ceil() as i32;
                    let py1 = ((r as f32 + 1.0) * self.cell_h).ceil() as i32;

                    // Check neighbors to draw beautiful arcade wall edges
                    let top_open = r == 0 || self.tiles[(r - 1) * self.cols + c] == Tile::Path;
                    let btm_open = r + 1 == self.rows || self.tiles[(r + 1) * self.cols + c] == Tile::Path;
                    let left_open = c == 0 || self.tiles[r * self.cols + (c - 1)] == Tile::Path;
                    let right_open = c + 1 == self.cols || self.tiles[r * self.cols + (c + 1)] == Tile::Path;

                    // Fill wall core softly
                    for y in py0..py1 {
                        for x in px0..px1 {
                            let is_edge = (top_open && y == py0)
                                || (btm_open && y == py1 - 1)
                                || (left_open && x == px0)
                                || (right_open && x == px1 - 1);

                            if is_edge {
                                canvas.set_pixel(x, y, glow);
                            } else {
                                canvas.set_pixel(x, y, self.wall_color);
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Helper to ensure all path tiles are connected to the central chamber
fn ensure_connected(tiles: &mut [Tile], cols: usize, rows: usize, start_c: usize, start_r: usize) {
    let total = cols * rows;
    let mut visited = vec![false; total];
    let mut queue = VecDeque::new();

    let start_idx = start_r * cols + start_c;
    visited[start_idx] = true;
    queue.push_back((start_c, start_r));

    while let Some((c, r)) = queue.pop_front() {
        for &d in &Direction::ALL {
            let nc = c as i32 + d.dx();
            let nr = r as i32 + d.dy();
            if nc >= 0 && nc < cols as i32 && nr >= 0 && nr < rows as i32 {
                let nidx = (nr as usize) * cols + (nc as usize);
                if tiles[nidx] == Tile::Path && !visited[nidx] {
                    visited[nidx] = true;
                    queue.push_back((nc as usize, nr as usize));
                }
            }
        }
    }

    // Connect any unvisited paths to the main component
    for r in 1..rows - 1 {
        for c in 1..cols - 1 {
            let idx = r * cols + c;
            if tiles[idx] == Tile::Path && !visited[idx] {
                // Turn into wall or tunnel to neighbor
                tiles[idx] = Tile::Wall;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_maze_connectivity() {
        let maze = Maze::generate(120, 60);
        assert!(maze.cols >= 13);
        assert!(maze.rows >= 9);

        // Find two path tiles
        let mut path_tiles = Vec::new();
        for r in 1..maze.rows - 1 {
            for c in 1..maze.cols - 1 {
                if maze.tiles[r * maze.cols + c] == Tile::Path {
                    path_tiles.push((c as i32, r as i32));
                }
            }
        }

        assert!(path_tiles.len() >= 4);
        let start = path_tiles[0];
        let end = path_tiles[path_tiles.len() - 1];

        // Verify BFS pathfinding finds a path
        let step = maze.bfs_next_direction(start.0, start.1, end.0, end.1);
        assert!(step.is_some());
    }
}
