use anyhow::Result;

use crate::cli::ExchangeCmd;
use crate::client::KalshiClient;
use crate::models::exchange::{
    ExchangeStatus, ExchangeStatusResponse, ScheduleResponse, UserDataTimestampResponse,
};
use crate::output::{OutputConfig, OutputFormat, output_one, paged_print, print_json};

pub async fn execute(client: &KalshiClient, cmd: ExchangeCmd, out: &OutputConfig) -> Result<()> {
    match cmd {
        ExchangeCmd::Status => {
            let resp: ExchangeStatusResponse = client.get("/exchange/status", &[]).await?;
            let status = ExchangeStatus {
                exchange_active: resp.exchange_active,
                trading_active: resp.trading_active,
            };
            output_one(&status, out)?;
        }
        ExchangeCmd::Announcement => {
            let resp: serde_json::Value = client.get("/exchange/announcements", &[]).await?;
            print_json(&resp, out.no_pager)?;
        }
        ExchangeCmd::Schedule => {
            let resp: ScheduleResponse = client.get("/exchange/schedule", &[]).await?;
            if matches!(out.format, OutputFormat::Json) {
                print_json(&resp, out.no_pager)?;
            } else {
                // Render a human-readable schedule table
                let mut text = String::from("=== Exchange Schedule ===\n\n");
                if let Some(ref schedule) = resp.schedule {
                    if let Some(ref hours) = schedule.standard_hours {
                        let days = ["monday", "tuesday", "wednesday", "thursday", "friday", "saturday", "sunday"];
                        if let Some(entries) = hours.as_array() {
                            for entry in entries {
                                for day in &days {
                                    if let Some(slots) = entry.get(day).and_then(|v| v.as_array()) {
                                        let windows: Vec<String> = slots.iter().filter_map(|slot| {
                                            let open = slot.get("open_time")?.as_str()?;
                                            let close = slot.get("close_time")?.as_str()?;
                                            if open == close {
                                                Some("24h".to_string())
                                            } else {
                                                Some(format!("{} – {}", open, close))
                                            }
                                        }).collect();
                                        let label = format!("{:<11}", format!("{}:", day[..1].to_uppercase().to_string() + &day[1..]));
                                        text.push_str(&format!("  {} {}\n", label, windows.join(", ")));
                                    }
                                }
                            }
                        }
                    }
                }
                paged_print(&text, out.no_pager);
            }
        }
        ExchangeCmd::UserDataTimestamp => {
            client.require_auth()?;
            let resp: UserDataTimestampResponse =
                client.get("/exchange/user_data_timestamp", &[]).await?;
            println!(
                "User data as of: {}",
                resp.as_of_time.unwrap_or_else(|| "-".to_string())
            );
        }
    }
    Ok(())
}
