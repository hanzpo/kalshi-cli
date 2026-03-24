use anyhow::Result;

use crate::cli::{QuoteCmd, RfqCmd};
use crate::client::KalshiClient;
use crate::models::communications::{
    CreateQuoteRequest, CreateRfqRequest, QuoteResponse, QuotesResponse, RfqResponse,
    RfqsResponse,
};
use crate::output::{OutputFormat, output, output_one, print_json};

pub async fn execute_rfq(
    client: &KalshiClient,
    cmd: RfqCmd,
    format: &OutputFormat,
) -> Result<()> {
    client.require_auth()?;

    match cmd {
        RfqCmd::List { limit, cursor } => {
            let mut query = Vec::new();
            let limit_str = limit.map(|l| l.to_string());
            if let Some(ref l) = limit_str {
                query.push(("limit", l.as_str()));
            }
            if let Some(ref c) = cursor {
                query.push(("cursor", c.as_str()));
            }
            let resp: RfqsResponse = client.get("/communications/rfqs", &query).await?;
            output(&resp.rfqs.unwrap_or_default(), format)?;
        }
        RfqCmd::Create {
            ticker,
            quantity,
            side,
        } => {
            let req = CreateRfqRequest {
                ticker,
                count: quantity,
                side,
            };
            let resp: RfqResponse = client.post("/communications/rfqs", &req).await?;
            if let Some(rfq) = resp.rfq {
                output_one(&rfq, format)?;
            } else {
                print_json(&resp)?;
            }
        }
        RfqCmd::Get { rfq_id } => {
            let path = format!("/communications/rfqs/{}", rfq_id);
            let resp: RfqResponse = client.get(&path, &[]).await?;
            if let Some(rfq) = resp.rfq {
                output_one(&rfq, format)?;
            } else {
                println!("RFQ not found.");
            }
        }
        RfqCmd::Cancel { rfq_id } => {
            let path = format!("/communications/rfqs/{}", rfq_id);
            let resp: serde_json::Value = client.delete(&path).await?;
            print_json(&resp)?;
        }
    }
    Ok(())
}

pub async fn execute_quote(
    client: &KalshiClient,
    cmd: QuoteCmd,
    format: &OutputFormat,
) -> Result<()> {
    client.require_auth()?;

    match cmd {
        QuoteCmd::List { rfq_id } => {
            let query = [("rfq_id", rfq_id.as_str())];
            let resp: QuotesResponse = client.get("/communications/quotes", &query).await?;
            output(&resp.quotes.unwrap_or_default(), format)?;
        }
        QuoteCmd::Create { rfq_id, price } => {
            let req = CreateQuoteRequest { rfq_id, price };
            let resp: QuoteResponse = client.post("/communications/quotes", &req).await?;
            if let Some(quote) = resp.quote {
                output_one(&quote, format)?;
            } else {
                print_json(&resp)?;
            }
        }
        QuoteCmd::Accept { quote_id } => {
            let path = format!("/communications/quotes/{}/accept", quote_id);
            let resp: serde_json::Value = client.put(&path, &serde_json::json!({})).await?;
            print_json(&resp)?;
        }
        QuoteCmd::Cancel { quote_id } => {
            let path = format!("/communications/quotes/{}", quote_id);
            let resp: serde_json::Value = client.delete(&path).await?;
            print_json(&resp)?;
        }
    }
    Ok(())
}
