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

#[cfg(test)]
mod tests {
    use super::*;

    fn full_milestone() -> Milestone {
        Milestone {
            id: Some("MS-1".to_string()),
            title: Some("Q1 Goals".to_string()),
            category: Some("finance".to_string()),
            start_date: Some("2025-01-01".to_string()),
            status: Some("active".to_string()),
            extra: Default::default(),
        }
    }

    fn empty_milestone() -> Milestone {
        Milestone {
            id: None,
            title: None,
            category: None,
            start_date: None,
            status: None,
            extra: Default::default(),
        }
    }

    #[test]
    fn headers_returns_five_columns() {
        let h = Milestone::headers();
        assert_eq!(h.len(), 5);
        assert_eq!(h, vec!["ID", "Title", "Category", "Start Date", "Status"]);
    }

    #[test]
    fn row_formats_all_some_fields() {
        let m = full_milestone();
        let row = m.row();
        assert_eq!(row.len(), 5);
        assert_eq!(row[0], "MS-1");
        assert_eq!(row[1], "Q1 Goals");
        assert_eq!(row[2], "finance");
        assert_eq!(row[3], "2025-01-01");
        assert_eq!(row[4], "active");
    }

    #[test]
    fn row_formats_none_as_dash() {
        let m = empty_milestone();
        let row = m.row();
        for field in &row {
            assert_eq!(field, "-");
        }
    }

    #[test]
    fn row_mixed_some_and_none() {
        let m = Milestone {
            id: Some("MS-2".to_string()),
            title: None,
            category: Some("tech".to_string()),
            start_date: None,
            status: Some("completed".to_string()),
            extra: Default::default(),
        };
        let row = m.row();
        assert_eq!(row[0], "MS-2");
        assert_eq!(row[1], "-");
        assert_eq!(row[2], "tech");
        assert_eq!(row[3], "-");
        assert_eq!(row[4], "completed");
    }
}
