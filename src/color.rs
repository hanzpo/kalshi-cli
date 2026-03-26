use crossterm::style::Stylize;

pub fn green(s: &str, enabled: bool) -> String {
    if enabled {
        s.green().to_string()
    } else {
        s.to_string()
    }
}

pub fn red(s: &str, enabled: bool) -> String {
    if enabled {
        s.red().to_string()
    } else {
        s.to_string()
    }
}

pub fn yellow(s: &str, enabled: bool) -> String {
    if enabled {
        s.yellow().to_string()
    } else {
        s.to_string()
    }
}

pub fn color_pnl(cents: i64, enabled: bool) -> String {
    let dollars = cents as f64 / 100.0;
    let formatted = if cents >= 0 {
        format!("+${:.2}", dollars)
    } else {
        format!("-${:.2}", -dollars)
    };
    if cents > 0 {
        green(&formatted, enabled)
    } else if cents < 0 {
        red(&formatted, enabled)
    } else {
        formatted
    }
}

pub fn color_status(status: &str, enabled: bool) -> String {
    match status.to_lowercase().as_str() {
        "open" | "active" => green(status, enabled),
        "closed" | "finalized" => red(status, enabled),
        "settled" | "initialized" => yellow(status, enabled),
        _ => status.to_string(),
    }
}
