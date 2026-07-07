use std::io::IsTerminal;
use std::sync::OnceLock;

/// Color is emitted only when stdout is a terminal and NO_COLOR is unset.
fn enabled() -> bool {
    static ENABLED: OnceLock<bool> = OnceLock::new();
    *ENABLED.get_or_init(|| {
        std::env::var_os("NO_COLOR").is_none() && std::io::stdout().is_terminal()
    })
}

fn code(c: &'static str) -> &'static str {
    if enabled() { c } else { "" }
}

pub fn red() -> &'static str {
    code("\x1b[31m")
}

pub fn green() -> &'static str {
    code("\x1b[32m")
}

pub fn cyan() -> &'static str {
    code("\x1b[36m")
}

pub fn bold() -> &'static str {
    code("\x1b[1m")
}

pub fn reset() -> &'static str {
    code("\x1b[0m")
}
