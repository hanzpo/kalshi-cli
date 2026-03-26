use crate::alerts::Alert;
use crate::output::TableDisplay;

impl TableDisplay for Alert {
    fn headers() -> Vec<&'static str> {
        vec!["ID", "Ticker", "Above", "Below", "Created"]
    }

    fn row(&self) -> Vec<String> {
        vec![
            self.id[..8].to_string(),
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
