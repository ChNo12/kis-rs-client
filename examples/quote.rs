use std::env;

use kis_rs_client::Client;
use kis_rs_client::rest::domestic_stock::{InquirePriceRequest, MarketDivision, StockCode};

#[tokio::main]
async fn main() -> kis_rs_client::Result<()> {
    let app_key = env::var("KIS_APP_KEY")
        .map_err(|_| kis_rs_client::Error::config("KIS_APP_KEY is required"))?;
    let app_secret = env::var("KIS_APP_SECRET")
        .map_err(|_| kis_rs_client::Error::config("KIS_APP_SECRET is required"))?;
    let use_virtual = env::var("KIS_USE_VIRTUAL").unwrap_or_else(|_| "true".to_string());
    let stock_code = env::var("KIS_STOCK_CODE").unwrap_or_else(|_| "005930".to_string());

    let builder = Client::builder().credentials(app_key, app_secret)?;
    let builder = if use_virtual == "false" {
        builder.real()
    } else {
        builder.virtual_trading()
    };
    let client = builder.build_reqwest()?;

    let token = client.issue_token().await?;
    let request = InquirePriceRequest::new(MarketDivision::Krx, StockCode::new(stock_code)?);
    let response = client
        .domestic_stock()
        .quotations()
        .inquire_price(&token.access_token, request)
        .await?;
    let quote = response.typed()?;

    println!("current price: {}", quote.current_price);
    println!("previous day rate: {}", quote.previous_day_rate);

    Ok(())
}
