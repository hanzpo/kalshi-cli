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

pub fn color_result(result: &str, enabled: bool) -> String {
    match result.to_lowercase().as_str() {
        "yes" => green(result, enabled),
        "no" => red(result, enabled),
        _ => result.to_string(),
    }
}

pub fn color_order_status(status: &str, enabled: bool) -> String {
    match status.to_lowercase().as_str() {
        "resting" | "pending" => yellow(status, enabled),
        "executed" | "filled" => green(status, enabled),
        "canceled" | "cancelled" => red(status, enabled),
        _ => status.to_string(),
    }
}

pub fn color_side(side: &str, enabled: bool) -> String {
    match side.to_lowercase().as_str() {
        "yes" => green(side, enabled),
        "no" => red(side, enabled),
        _ => side.to_string(),
    }
}

pub fn color_action(action: &str, enabled: bool) -> String {
    match action.to_lowercase().as_str() {
        "buy" => green(action, enabled),
        "sell" => red(action, enabled),
        _ => action.to_string(),
    }
}

pub fn color_bool(val: bool, enabled: bool) -> String {
    if val {
        green(&val.to_string(), enabled)
    } else {
        red(&val.to_string(), enabled)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_green_disabled() {
        assert_eq!(green("hello", false), "hello");
    }

    #[test]
    fn test_green_enabled() {
        let result = green("hello", true);
        assert!(result.contains("hello"));
        assert_ne!(result, "hello");
    }

    #[test]
    fn test_red_disabled() {
        assert_eq!(red("error", false), "error");
    }

    #[test]
    fn test_red_enabled() {
        let result = red("error", true);
        assert!(result.contains("error"));
        assert_ne!(result, "error");
    }

    #[test]
    fn test_yellow_disabled() {
        assert_eq!(yellow("warn", false), "warn");
    }

    #[test]
    fn test_yellow_enabled() {
        let result = yellow("warn", true);
        assert!(result.contains("warn"));
        assert_ne!(result, "warn");
    }

    #[test]
    fn test_green_empty_string() {
        assert_eq!(green("", false), "");
    }

    #[test]
    fn test_color_pnl_positive() {
        let result = color_pnl(150, false);
        assert_eq!(result, "+$1.50");
    }

    #[test]
    fn test_color_pnl_negative() {
        let result = color_pnl(-250, false);
        assert_eq!(result, "-$2.50");
    }

    #[test]
    fn test_color_pnl_zero() {
        let result = color_pnl(0, false);
        assert_eq!(result, "+$0.00");
    }

    #[test]
    fn test_color_pnl_positive_colored() {
        let result = color_pnl(100, true);
        assert!(result.contains("+$1.00"));
        assert_ne!(result, "+$1.00");
    }

    #[test]
    fn test_color_pnl_negative_colored() {
        let result = color_pnl(-100, true);
        assert!(result.contains("-$1.00"));
        assert_ne!(result, "-$1.00");
    }

    #[test]
    fn test_color_pnl_zero_colored() {
        // zero is not colored
        let result = color_pnl(0, true);
        assert_eq!(result, "+$0.00");
    }

    #[test]
    fn test_color_pnl_one_cent() {
        assert_eq!(color_pnl(1, false), "+$0.01");
    }

    #[test]
    fn test_color_pnl_minus_one_cent() {
        assert_eq!(color_pnl(-1, false), "-$0.01");
    }

    #[test]
    fn test_color_status_open() {
        assert_eq!(color_status("open", false), "open");
        let colored = color_status("open", true);
        assert!(colored.contains("open"));
        assert_ne!(colored, "open");
    }

    #[test]
    fn test_color_status_active() {
        assert_eq!(color_status("active", false), "active");
    }

    #[test]
    fn test_color_status_closed() {
        assert_eq!(color_status("closed", false), "closed");
        let colored = color_status("closed", true);
        assert!(colored.contains("closed"));
        assert_ne!(colored, "closed");
    }

    #[test]
    fn test_color_status_finalized() {
        assert_eq!(color_status("finalized", false), "finalized");
    }

    #[test]
    fn test_color_status_settled() {
        assert_eq!(color_status("settled", false), "settled");
        let colored = color_status("settled", true);
        assert!(colored.contains("settled"));
        assert_ne!(colored, "settled");
    }

    #[test]
    fn test_color_status_initialized() {
        assert_eq!(color_status("initialized", false), "initialized");
    }

    #[test]
    fn test_color_status_unknown() {
        assert_eq!(color_status("unknown", false), "unknown");
        assert_eq!(color_status("unknown", true), "unknown");
    }

    #[test]
    fn test_color_status_case_insensitive() {
        assert_eq!(color_status("Open", false), "Open");
        assert_eq!(color_status("CLOSED", false), "CLOSED");
    }
}
