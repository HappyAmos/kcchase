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
    capture_mouse: bool,
}

impl TerminalGuard {
    pub fn enter(capture_mouse: bool) -> io::Result<Self> {
        let active = Arc::new(AtomicBool::new(true));
        let active_clone = Arc::clone(&active);

        // Set panic hook so unexpected panics still restore the terminal
        let default_hook = panic::take_hook();
        panic::set_hook(Box::new(move |info| {
            if active_clone.swap(false, Ordering::SeqCst) {
                let _ = write!(io::stdout(), "\x1b[?7h"); // Re-enable autowrap
                let _ = disable_raw_mode();
                if capture_mouse {
                    let _ = execute!(
                        io::stdout(),
                        LeaveAlternateScreen,
                        DisableMouseCapture,
                        Show
                    );
                } else {
                    let _ = execute!(
                        io::stdout(),
                        LeaveAlternateScreen,
                        Show
                    );
                }
            }
            default_hook(info);
        }));

        enable_raw_mode()?;
        let mut out = stdout();
        if capture_mouse {
            execute!(out, EnterAlternateScreen, EnableMouseCapture, Hide)?;
        } else {
            execute!(out, EnterAlternateScreen, Hide)?;
        }
        // Disable autowrap so full-width rows and bottom-right corner never trigger extra line wraps or scrolling
        write!(out, "\x1b[?7l")?;
        out.flush()?;

        Ok(TerminalGuard {
            active,
            capture_mouse,
        })
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        if self.active.swap(false, Ordering::SeqCst) {
            let _ = write!(io::stdout(), "\x1b[?7h"); // Re-enable autowrap
            let _ = disable_raw_mode();
            if self.capture_mouse {
                let _ = execute!(
                    io::stdout(),
                    LeaveAlternateScreen,
                    DisableMouseCapture,
                    Show
                );
            } else {
                let _ = execute!(
                    io::stdout(),
                    LeaveAlternateScreen,
                    Show
                );
            }
            let _ = io::stdout().flush();
        }
    }
}

pub struct InputWatcher {
    start_time: Instant,
    initial_mouse_pos: Option<(u16, u16)>,
    exit_on_keyboard: bool,
    exit_on_mouse: bool,
}

impl InputWatcher {
    pub fn new(exit_on_keyboard: bool, exit_on_mouse: bool) -> Self {
        InputWatcher {
            start_time: Instant::now(),
            initial_mouse_pos: None,
            exit_on_keyboard,
            exit_on_mouse,
        }
    }

    pub fn handle_key_event(&self, key_event: event::KeyEvent) -> PollResult {
        // If keyboard exit is enabled, any key press exits the screensaver
        if self.exit_on_keyboard {
            return PollResult::Exit(format!("Key {:?} pressed", key_event.code));
        }

        // Even when general keyboard exit is disabled (e.g. --mouse),
        // allow emergency fail-safes (Ctrl+C, Esc) to exit safely.
        if key_event.modifiers.contains(event::KeyModifiers::CONTROL)
            && matches!(key_event.code, event::KeyCode::Char('c') | event::KeyCode::Char('C'))
        {
            return PollResult::Exit("Ctrl+C pressed".into());
        }
        if key_event.code == event::KeyCode::Esc {
            return PollResult::Exit("Escape pressed".into());
        }

        PollResult::Ignored
    }

    /// Check for input events. Returns:
    /// - Ok(Some(PollResult::Exit(...))) if an exit trigger occurred
    /// - Ok(Some(PollResult::Resize(...))) if a terminal resize occurred
    /// - Ok(Some(PollResult::Ignored)) if an event was handled/ignored
    /// - Ok(None) if no event occurred within timeout
    pub fn poll_event(&mut self, timeout: Duration) -> io::Result<Option<PollResult>> {
        if !event::poll(timeout)? {
            return Ok(None);
        }

        match event::read()? {
            Event::Key(key_event) => Ok(Some(self.handle_key_event(key_event))),
            Event::Mouse(mouse_event) => {
                if !self.exit_on_mouse {
                    return Ok(Some(PollResult::Ignored));
                }

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

#[derive(Debug, PartialEq, Eq)]
pub enum PollResult {
    Exit(String),
    Resize(u16, u16),
    Ignored,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    #[test]
    fn test_input_watcher_keyboard_mode() {
        let watcher = InputWatcher::new(true, false);
        let normal_key = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE);
        let space_key = KeyEvent::new(KeyCode::Char(' '), KeyModifiers::NONE);

        assert!(matches!(watcher.handle_key_event(normal_key), PollResult::Exit(_)));
        assert!(matches!(watcher.handle_key_event(space_key), PollResult::Exit(_)));
    }

    #[test]
    fn test_input_watcher_mouse_only_mode_ignores_normal_keys_but_keeps_fail_safes() {
        let watcher = InputWatcher::new(false, true);

        // Regular keys should be ignored
        let normal_key = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE);
        let space_key = KeyEvent::new(KeyCode::Char(' '), KeyModifiers::NONE);
        assert_eq!(watcher.handle_key_event(normal_key), PollResult::Ignored);
        assert_eq!(watcher.handle_key_event(space_key), PollResult::Ignored);

        // Fail-safe keys must exit
        let ctrl_c = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL);
        let esc = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
        assert!(matches!(watcher.handle_key_event(ctrl_c), PollResult::Exit(_)));
        assert!(matches!(watcher.handle_key_event(esc), PollResult::Exit(_)));
    }
}
