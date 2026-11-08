use crossterm::{
    cursor::{Hide, Show},
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, MouseEventKind},
    execute,
    terminal::{
        disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
    },
};
use std::io::{self, stdout, Write};
use std::panic;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// RAII Guard that ensures the terminal is cleanly restored on drop or panic.
pub struct TerminalGuard {
    active: Arc<AtomicBool>,
}

impl TerminalGuard {
    pub fn enter() -> io::Result<Self> {
        let active = Arc::new(AtomicBool::new(true));
        let active_clone = Arc::clone(&active);

        // Set panic hook so unexpected panics still restore the terminal
        let default_hook = panic::take_hook();
        panic::set_hook(Box::new(move |info| {
            if active_clone.swap(false, Ordering::SeqCst) {
                let _ = write!(io::stdout(), "\x1b[?7h"); // Re-enable autowrap
                let _ = disable_raw_mode();
                let _ = execute!(
                    io::stdout(),
                    LeaveAlternateScreen,
                    DisableMouseCapture,
                    Show
                );
            }
            default_hook(info);
        }));

        enable_raw_mode()?;
        let mut out = stdout();
        execute!(out, EnterAlternateScreen, EnableMouseCapture, Hide)?;
        // Disable autowrap so full-width rows and bottom-right corner never trigger extra line wraps or scrolling
        write!(out, "\x1b[?7l")?;
        out.flush()?;

        Ok(TerminalGuard { active })
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        if self.active.swap(false, Ordering::SeqCst) {
            let _ = write!(io::stdout(), "\x1b[?7h"); // Re-enable autowrap
            let _ = disable_raw_mode();
            let _ = execute!(
                io::stdout(),
                LeaveAlternateScreen,
                DisableMouseCapture,
                Show
            );
            let _ = io::stdout().flush();
        }
    }
}

pub struct InputWatcher {
    start_time: Instant,
    initial_mouse_pos: Option<(u16, u16)>,
}

impl InputWatcher {
    pub fn new() -> Self {
        InputWatcher {
            start_time: Instant::now(),
            initial_mouse_pos: None,
        }
    }

    /// Check for input events. Returns:
    /// - Ok(Some(true)) if an exit trigger occurred (key press or mouse move)
    /// - Ok(Some(false)) if a non-exit event occurred (e.g. terminal resize)
    /// - Ok(None) if no event occurred within timeout
    pub fn poll_event(&mut self, timeout: Duration) -> io::Result<Option<PollResult>> {
        if !event::poll(timeout)? {
            return Ok(None);
        }

        match event::read()? {
            Event::Key(key_event) => {
                // Any key press exits the screensaver
                Ok(Some(PollResult::Exit(format!(
                    "Key {:?} pressed",
                    key_event.code
                ))))
            }
            Event::Mouse(mouse_event) => {
                let (col, row) = (mouse_event.column, mouse_event.row);

                match self.initial_mouse_pos {
                    None => {
                        // First mouse event: record initial position
                        self.initial_mouse_pos = Some((col, row));

                        // If user clicked or scrolled immediately, exit
                        if matches!(
                            mouse_event.kind,
                            MouseEventKind::Down(_)
                                | MouseEventKind::Up(_)
                                | MouseEventKind::ScrollDown
                                | MouseEventKind::ScrollUp
                                | MouseEventKind::Drag(_)
                        ) {
                            return Ok(Some(PollResult::Exit("Mouse click/scroll".into())));
                        }
                        Ok(Some(PollResult::Ignored))
                    }
                    Some((init_col, init_row)) => {
                        // Allow small grace period (150ms) to ignore terminal initial coordinate noise
                        let elapsed = self.start_time.elapsed();
                        let dx = (col as i32 - init_col as i32).abs();
                        let dy = (row as i32 - init_row as i32).abs();

                        if elapsed < Duration::from_millis(150) {
                            if dx > 1 || dy > 1 {
                                return Ok(Some(PollResult::Exit("Mouse moved".into())));
                            }
                            Ok(Some(PollResult::Ignored))
                        } else {
                            if dx > 0 || dy > 0 {
                                return Ok(Some(PollResult::Exit(format!(
                                    "Mouse moved from ({}, {}) to ({}, {})",
                                    init_col, init_row, col, row
                                ))));
                            }
                            if matches!(
                                mouse_event.kind,
                                MouseEventKind::Down(_)
                                    | MouseEventKind::Up(_)
                                    | MouseEventKind::ScrollDown
                                    | MouseEventKind::ScrollUp
                                    | MouseEventKind::Drag(_)
                            ) {
                                return Ok(Some(PollResult::Exit("Mouse click/scroll".into())));
                            }
                            Ok(Some(PollResult::Ignored))
                        }
                    }
                }
            }
            Event::Resize(cols, rows) => Ok(Some(PollResult::Resize(cols, rows))),
            _ => Ok(Some(PollResult::Ignored)),
        }
    }
}

pub enum PollResult {
    Exit(String),
    Resize(u16, u16),
    Ignored,
}
