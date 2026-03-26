use serde::{Deserialize, Serialize};

use crate::models::common::format_opt;
use crate::output::TableDisplay;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Milestone {
    pub id: Option<String>,
    pub title: Option<String>,
    pub category: Option<String>,
    pub start_date: Option<String>,
    pub status: Option<String>,
    #[serde(flatten)]
    pub extra: std::collections::HashMap<String, serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct MilestonesResponse {
    pub milestones: Option<Vec<Milestone>>,
    pub cursor: Option<String>,
}

impl TableDisplay for Milestone {
    fn headers() -> Vec<&'static str> {
        vec!["ID", "Title", "Category", "Start Date", "Status"]
    }

    fn row(&self) -> Vec<String> {
        vec![
            format_opt(&self.id),
            format_opt(&self.title),
            format_opt(&self.category),
            format_opt(&self.start_date),
            format_opt(&self.status),
        ]
    }
}
