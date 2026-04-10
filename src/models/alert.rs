use crate::alerts::Alert;
use crate::output::TableDisplay;

impl TableDisplay for Alert {
    fn headers() -> Vec<&'static str> {
        vec!["ID", "Ticker", "Above", "Below", "Created"]
    }

    fn row(&self) -> Vec<String> {
        vec![
            self.id.chars().take(8).collect::<String>(),
            self.ticker.clone(),
            self.above
                .map(|v| format!("{}c", v))
                .unwrap_or("-".to_string()),
            self.below
                .map(|v| format!("{}c", v))
                .unwrap_or("-".to_string()),
            self.created_at.clone(),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alert_headers() {
        let headers = Alert::headers();
        assert_eq!(headers, vec!["ID", "Ticker", "Above", "Below", "Created"]);
    }

    #[test]
    fn test_alert_row_id_truncated_to_8() {
        let alert = Alert {
            id: "abcdef1234567890".to_string(),
            ticker: "MKT-1".to_string(),
            above: Some(70.0),
            below: Some(30.0),
            created_at: "2026-01-01".to_string(),
        };
        let row = alert.row();
        assert_eq!(row[0], "abcdef12");
        assert_eq!(row[0].len(), 8);
    }

    #[test]
    fn test_alert_row_above_below_formatted() {
        let alert = Alert {
            id: "12345678abcdef".to_string(),
            ticker: "T1".to_string(),
            above: Some(65.5),
            below: Some(35.0),
            created_at: "2026-03-01".to_string(),
        };
        let row = alert.row();
        assert_eq!(row[1], "T1");
        assert_eq!(row[2], "65.5c");
        assert_eq!(row[3], "35c");
        assert_eq!(row[4], "2026-03-01");
    }

    #[test]
    fn test_alert_row_above_none() {
        let alert = Alert {
            id: "12345678abcdef".to_string(),
            ticker: "T1".to_string(),
            above: None,
            below: Some(40.0),
            created_at: "2026-03-01".to_string(),
        };
        let row = alert.row();
        assert_eq!(row[2], "-");
        assert_eq!(row[3], "40c");
    }

    #[test]
    fn test_alert_row_below_none() {
        let alert = Alert {
            id: "12345678abcdef".to_string(),
            ticker: "T1".to_string(),
            above: Some(80.0),
            below: None,
            created_at: "2026-03-01".to_string(),
        };
        let row = alert.row();
        assert_eq!(row[2], "80c");
        assert_eq!(row[3], "-");
    }

    #[test]
    fn test_alert_row_both_none() {
        let alert = Alert {
            id: "12345678abcdef".to_string(),
            ticker: "T1".to_string(),
            above: None,
            below: None,
            created_at: "2026-03-01".to_string(),
        };
        let row = alert.row();
        assert_eq!(row[2], "-");
        assert_eq!(row[3], "-");
    }
}
