mod canvas;
mod chase;
mod color;
mod maze;
mod sprites;
mod terminal;

use canvas::Canvas;
use chase::ChaseSimulation;
use color::ColorMode;
use std::env;
use std::io::{self, stdout, Write};
use std::time::{Duration, Instant};
use terminal::{InputWatcher, PollResult, TerminalGuard};

struct AppArgs {
    color_mode: Option<ColorMode>,
    fps: u64,
    speed: f32,
    show_info: bool,
    show_help: bool,
}

fn parse_args() -> AppArgs {
    let args: Vec<String> = env::args().collect();
    let mut color_mode = None;
    let mut fps = 60;
    let mut speed = 1.0f32;
    let mut show_info = false;
    let mut show_help = false;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-h" | "--help" => {
                show_help = true;
            }
            "-i" | "--info" => {
                show_info = true;
            }
            "-c" | "--color" => {
                if i + 1 < args.len() {
                    i += 1;
                    if let Some(m) = ColorMode::from_str_opt(&args[i]) {
                        color_mode = Some(m);
                    } else {
                        eprintln!("Unknown color mode '{}'. Options: truecolor, 256, 16, 8", args[i]);
                    }
                }
            }
            "--fps" => {
                if i + 1 < args.len() {
                    i += 1;
                    if let Ok(val) = args[i].parse::<u64>() {
                        fps = val.clamp(15, 120);
                    }
                }
            }
            "--speed" => {
                if i + 1 < args.len() {
                    i += 1;
                    if let Ok(val) = args[i].parse::<f32>() {
                        speed = val.clamp(0.2, 5.0);
                    }
                }
            }
            _ => {}
        }
        i += 1;
    }

    AppArgs {
        color_mode,
        fps,
        speed,
        show_info,
        show_help,
    }
}

fn print_help() {
    println!("kcchase - K.C. Munchkin Terminal Screensaver");
    println!();
    println!("USAGE:");
    println!("    kcchase [OPTIONS]");
    println!();
    println!("OPTIONS:");
    println!("    -h, --help               Print this help message and exit");
    println!("    -i, --info               Display detected terminal color capabilities and exit");
    println!("    -c, --color <MODE>       Select color mode: truecolor, 256, 16, 8 (default: auto)");
    println!("    --fps <NUMBER>           Set target frame rate (default: 60, range: 15-120)");
    println!("    --speed <FLOAT>          Set animation speed multiplier (default: 1.0)");
    println!();
    println!("EXIT:");
    println!("    Press any key or move the mouse to exit the screensaver.");
}

fn print_info(detected: ColorMode) {
    println!("Terminal Color Capability Detection:");
    println!("  Active Mode: {}", detected.name());
    println!("  COLORTERM:   {:?}", env::var("COLORTERM").ok());
    println!("  TERM:        {:?}", env::var("TERM").ok());
    println!();
    println!("Available Modes: truecolor (24-bit), 256 (8-bit), 16 (4-bit), 8 (3-bit)");
}

fn main() -> io::Result<()> {
    let args = parse_args();

    if args.show_help {
        print_help();
        return Ok(());
    }

    let detected_color = ColorMode::detect();
    if args.show_info {
        print_info(detected_color);
        return Ok(());
    }

    let active_color_mode = args.color_mode.unwrap_or(detected_color);

    // Initial terminal size
    let (mut term_cols, mut term_rows) = crossterm::terminal::size().unwrap_or((80, 24));
    term_cols = term_cols.max(20);
    term_rows = term_rows.max(10);

    // Set up RAII terminal guard (raw mode, alternate screen, mouse capture, hide cursor)
    let _guard = TerminalGuard::enter()?;

    let mut canvas = Canvas::new(term_cols, term_rows);
    let mut sim = ChaseSimulation::new(canvas.width, canvas.height);
    let mut watcher = InputWatcher::new();

    let frame_duration = Duration::from_micros(1_000_000 / args.fps);
    let mut last_frame = Instant::now();
    let stdout_handle = stdout();

    loop {
        let frame_start = Instant::now();
        let dt = (frame_start - last_frame).as_secs_f32().min(0.1);
        last_frame = frame_start;

        // Poll for input events or exit signals
        // Remaining time in frame budget for poll timeout
        let poll_budget = frame_duration.saturating_sub(frame_start.elapsed());
        if let Some(event) = watcher.poll_event(poll_budget)? {
            match event {
                PollResult::Exit(_reason) => {
                    break;
                }
                PollResult::Resize(cols, rows) => {
                    let cols = cols.max(20);
                    let rows = rows.max(10);
                    term_cols = cols;
                    term_rows = rows;
                    canvas.resize(term_cols, term_rows);
                    sim.resize(canvas.width, canvas.height);
                }
                PollResult::Ignored => {}
            }
        }

        // Advance simulation physics and state machine
        sim.update(dt * args.speed);

        // Draw simulation onto canvas
        sim.draw(&mut canvas);

        // Render virtual canvas to terminal ANSI string
        let output = canvas.render_to_string(active_color_mode);

        // Atomic write to terminal and flush
        {
            let mut lock = stdout_handle.lock();
            lock.write_all(output.as_bytes())?;
            lock.flush()?;
        }

        // Maintain consistent frame rate
        let elapsed = frame_start.elapsed();
        if elapsed < frame_duration {
            std::thread::sleep(frame_duration - elapsed);
        }
    }

    Ok(())
}
