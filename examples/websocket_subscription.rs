#[cfg(feature = "websocket-client")]
use std::env;

#[cfg(feature = "websocket-client")]
use kis_rs_client::Client;
#[cfg(feature = "websocket-client")]
use kis_rs_client::websocket::{
    DomesticRealtimePriceMarket, SubscriptionAction, SubscriptionBook, SubscriptionMessage,
    domestic,
};

#[cfg(feature = "websocket-client")]
#[tokio::main]
async fn main() -> kis_rs_client::Result<()> {
    let app_key = env::var("KIS_APP_KEY")
        .map_err(|_| kis_rs_client::Error::config("KIS_APP_KEY is required"))?;
    let app_secret = env::var("KIS_APP_SECRET")
        .map_err(|_| kis_rs_client::Error::config("KIS_APP_SECRET is required"))?;
    let use_mock = env::var("KIS_USE_MOCK").unwrap_or_else(|_| "true".to_string());
    let stock_code = env::var("KIS_STOCK_CODE").unwrap_or_else(|_| "005930".to_string());

    let builder = Client::builder().credentials(app_key, app_secret)?;
    let builder = if use_mock == "false" {
        builder.real()
    } else {
        builder.mock()
    };
    let client = builder.build_reqwest()?;

    let approval = client.issue_approval_key().await?;
    let subscription = domestic::realtime_price_subscription(
        SubscriptionAction::Subscribe,
        DomesticRealtimePriceMarket::Krx,
        stock_code,
    )?;
    let message = SubscriptionMessage::new(&approval.approval_key, subscription.clone());

    let mut book = SubscriptionBook::new();
    book.add(subscription);

    println!(
        "websocket url: {}, subscriptions: {}",
        client.websocket_base_url(),
        book.len()
    );
    println!("{}", serde_json::to_string_pretty(&message).unwrap());

    Ok(())
}

#[cfg(not(feature = "websocket-client"))]
fn main() {
    eprintln!("enable the websocket-client feature to run this example");
}
