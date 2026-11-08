mod canvas;
mod chase;
mod color;
mod maze;
mod sprites;
mod terminal;

use canvas::Canvas;
use chase::{ChaseSimulation, StartSequence};
use color::ColorMode;
use std::env;
use std::io::{self, stdout, Write};
use std::time::{Duration, Instant};
use terminal::{InputWatcher, PollResult, TerminalGuard};

struct AppArgs {
    color_mode: Option<ColorMode>,
    fps: u64,
    speed: f32,
    exit_keyboard: bool,
    exit_mouse: bool,
    start_sequence: StartSequence,
    show_info: bool,
    show_help: bool,
}

fn parse_args() -> AppArgs {
    parse_args_from(env::args())
}

fn parse_args_from<I>(iter: I) -> AppArgs
where
    I: IntoIterator,
    I::Item: Into<String>,
{
    let args: Vec<String> = iter.into_iter().map(Into::into).collect();
    let mut color_mode = None;
    let mut fps = 60;
    let mut speed = 1.0f32;
    let mut only_keyboard = false;
    let mut only_mouse = false;
    let mut start_sequence = StartSequence::Game;
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
            "-k" | "--keyboard" => {
                only_keyboard = true;
            }
            "-m" | "--mouse" => {
                only_mouse = true;
            }
            "--intermission" | "--cutscene" => {
                start_sequence = StartSequence::Intermission;
            }
            "--game" => {
                start_sequence = StartSequence::Game;
            }
            "--sequence" | "--mode" => {
                if i + 1 < args.len() {
                    i += 1;
                    match args[i].to_lowercase().as_str() {
                        "intermission" | "cutscene" | "rest" | "chase" => {
                            start_sequence = StartSequence::Intermission;
                        }
                        "game" | "play" | "normal" => {
                            start_sequence = StartSequence::Game;
                        }
                        other => {
                            eprintln!("Unknown sequence mode '{}'. Options: game, intermission", other);
                        }
                    }
                }
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

    let (exit_keyboard, exit_mouse) = match (only_keyboard, only_mouse) {
        (true, false) => (true, false),
        (false, true) => (false, true),
        _ => (true, true),
    };

    AppArgs {
        color_mode,
        fps,
        speed,
        exit_keyboard,
        exit_mouse,
        start_sequence,
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
    println!("    -k, --keyboard           Exit on keyboard input only (ignores mouse movement)");
    println!("    -m, --mouse              Exit on mouse movement only (ignores keyboard, except Ctrl+C/Esc)");
    println!("    --intermission, --cutscene Start with the intermission animation sequence");
    println!("    --game                     Start with the game sequence (default)");
    println!("    --sequence <MODE>          Select starting sequence: game (default), intermission");
    println!();
    println!("EXIT:");
    println!("    Press any key or move the mouse to exit the screensaver.");
    println!("    Use --keyboard or --mouse to restrict the exit trigger.");
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

    // Set up RAII terminal guard (raw mode, alternate screen, mouse capture if enabled, hide cursor)
    let _guard = TerminalGuard::enter(args.exit_mouse)?;

    let mut canvas = Canvas::new(term_cols, term_rows);
    let mut sim = ChaseSimulation::new_with_sequence(canvas.width, canvas.height, args.start_sequence);
    let mut watcher = InputWatcher::new(args.exit_keyboard, args.exit_mouse);

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_args_exits_on_both() {
        let args = parse_args_from(vec!["kcchase"]);
        assert!(args.exit_keyboard);
        assert!(args.exit_mouse);
    }

    #[test]
    fn test_keyboard_flag_disables_mouse() {
        let args = parse_args_from(vec!["kcchase", "--keyboard"]);
        assert!(args.exit_keyboard);
        assert!(!args.exit_mouse);

        let args_short = parse_args_from(vec!["kcchase", "-k"]);
        assert!(args_short.exit_keyboard);
        assert!(!args_short.exit_mouse);
    }

    #[test]
    fn test_mouse_flag_disables_keyboard() {
        let args = parse_args_from(vec!["kcchase", "--mouse"]);
        assert!(!args.exit_keyboard);
        assert!(args.exit_mouse);

        let args_short = parse_args_from(vec!["kcchase", "-m"]);
        assert!(!args_short.exit_keyboard);
        assert!(args_short.exit_mouse);
    }

    #[test]
    fn test_both_flags_enables_both() {
        let args = parse_args_from(vec!["kcchase", "--mouse", "--keyboard"]);
        assert!(args.exit_keyboard);
        assert!(args.exit_mouse);

        let args_short = parse_args_from(vec!["kcchase", "-k", "-m"]);
        assert!(args_short.exit_keyboard);
        assert!(args_short.exit_mouse);
    }

    #[test]
    fn test_default_sequence_is_game() {
        let args = parse_args_from(vec!["kcchase"]);
        assert_eq!(args.start_sequence, StartSequence::Game);
    }

    #[test]
    fn test_sequence_flags_intermission_and_cutscene() {
        let args_intermission = parse_args_from(vec!["kcchase", "--intermission"]);
        assert_eq!(args_intermission.start_sequence, StartSequence::Intermission);

        let args_cutscene = parse_args_from(vec!["kcchase", "--cutscene"]);
        assert_eq!(args_cutscene.start_sequence, StartSequence::Intermission);
    }

    #[test]
    fn test_sequence_flag_game() {
        let args_game = parse_args_from(vec!["kcchase", "--intermission", "--game"]);
        assert_eq!(args_game.start_sequence, StartSequence::Game);
    }

    #[test]
    fn test_sequence_mode_arguments() {
        let args_seq_inter = parse_args_from(vec!["kcchase", "--sequence", "intermission"]);
        assert_eq!(args_seq_inter.start_sequence, StartSequence::Intermission);

        let args_mode_cut = parse_args_from(vec!["kcchase", "--mode", "cutscene"]);
        assert_eq!(args_mode_cut.start_sequence, StartSequence::Intermission);

        let args_seq_game = parse_args_from(vec!["kcchase", "--sequence", "game"]);
        assert_eq!(args_seq_game.start_sequence, StartSequence::Game);
    }
}
