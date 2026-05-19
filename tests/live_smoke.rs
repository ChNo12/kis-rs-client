#![cfg(feature = "reqwest-client")]

use std::env;

use rust_decimal::Decimal;

use kis_rs_client::rest::domestic_stock::{
    AllQuantityOrder, InquireDailyCcldPeriod, InquireDailyCcldRequest, InquirePriceRequest,
    InquireTimeDailyChartPriceRequest, InquireTimeItemChartPriceRequest, MarketDivision,
    OrderCashRequest, OrderRvsecnclRequest as DomesticOrderRvsecnclRequest, StockCode,
};
use kis_rs_client::rest::overseas_stock::{
    InquireCcnlRequest as OverseasInquireCcnlRequest, OrderRequest as OverseasOrderRequest,
    OrderRvsecnclRequest as OverseasOrderRvsecnclRequest, OverseasExchange, OverseasStockCode,
};
use kis_rs_client::{Client, ClientBuilder, Error, ReqwestHttpClient, Result};

#[tokio::test]
#[ignore = "requires KIS_APP_KEY and KIS_APP_SECRET; read-only live KIS API calls"]
async fn live_smoke_readonly_domestic_quote_and_today_minute() -> Result<()> {
    let client = client_without_account()?;
    let token = client.issue_token().await?;
    let stock_code = stock_code()?;

    let quote = client
        .domestic_stock()
        .quotations()
        .inquire_price(
            &token.access_token,
            InquirePriceRequest::new(MarketDivision::Krx, stock_code.clone()),
        )
        .await?;
    assert!(quote.current_price().is_some());

    let minute = client
        .domestic_stock()
        .quotations()
        .inquire_time_item_chart_price(
            &token.access_token,
            InquireTimeItemChartPriceRequest::new(MarketDivision::Krx, stock_code, smoke_time())
                .include_past_data(true),
        )
        .await?;
    assert!(!minute.output1.is_null());
    assert!(!minute.output2.is_null());

    Ok(())
}

#[tokio::test]
#[ignore = "requires KIS_APP_KEY, KIS_APP_SECRET, and KIS_SMOKE_DATE; read-only live KIS API calls"]
async fn live_smoke_readonly_domestic_previous_day_minute() -> Result<()> {
    let client = client_without_account()?;
    let token = client.issue_token().await?;
    let stock_code = stock_code()?;
    let smoke_date = required_env("KIS_SMOKE_DATE")?;

    let minute = client
        .domestic_stock()
        .quotations()
        .inquire_time_daily_chart_price(
            &token.access_token,
            InquireTimeDailyChartPriceRequest::new(
                MarketDivision::Krx,
                stock_code,
                smoke_time(),
                smoke_date,
            )
            .include_past_data(true),
        )
        .await?;
    assert!(!minute.output1.is_null());
    assert!(!minute.output2.is_null());

    Ok(())
}

#[tokio::test]
#[ignore = "requires KIS account env vars; read-only live KIS API calls"]
async fn live_smoke_readonly_order_conclusions() -> Result<()> {
    let client = client_with_account()?;
    let token = client.issue_token().await?;
    let start_date = required_env("KIS_SMOKE_START_DATE")?;
    let end_date = env::var("KIS_SMOKE_END_DATE").unwrap_or_else(|_| start_date.clone());

    let domestic = client
        .domestic_stock()
        .trading()
        .inquire_daily_ccld(
            &token.access_token,
            InquireDailyCcldRequest::new(
                InquireDailyCcldPeriod::Inner3Months,
                &start_date,
                &end_date,
                "00",
                "00",
                "00",
                "00",
            )?
            .with_stock_code(optional_env("KIS_STOCK_CODE", "005930")),
        )
        .await?;
    assert!(!domestic.output1.is_null());
    assert!(!domestic.output2.is_null());

    let overseas_exchange =
        OverseasExchange::from_kis_code(&optional_env("KIS_OVERSEAS_EXCHANGE", "NASD"))?;
    let overseas = client
        .overseas_stock()
        .trading()
        .inquire_ccnl(
            &token.access_token,
            OverseasInquireCcnlRequest::new(&start_date, &end_date, "00", "00", "DS")?
                .with_stock_code(optional_env("KIS_OVERSEAS_STOCK_CODE", "AAPL"))
                .with_exchange(overseas_exchange),
        )
        .await?;
    assert!(!overseas.output.is_null());

    Ok(())
}

#[tokio::test]
#[ignore = "requires KIS_ENABLE_VIRTUAL_ORDER_SMOKE=true; creates a virtual domestic stock order"]
async fn live_smoke_virtual_domestic_buy_order_and_best_effort_cancel() -> Result<()> {
    ensure_virtual_order_smoke_enabled()?;

    let client = virtual_client_with_account()?;
    let token = client.issue_token().await?;
    let stock_code = stock_code()?;
    let quantity = limited_quantity("KIS_VIRTUAL_ORDER_QTY")?;
    let price = positive_price_env("KIS_VIRTUAL_DOMESTIC_ORDER_PRICE")?;

    let order = client
        .domestic_stock()
        .trading()
        .order_cash(
            &token.access_token,
            OrderCashRequest::buy(stock_code, "00", &quantity, &price, "KRX")?,
        )
        .await?;
    let order_no = required_output_field(&order.output, &["ODNO"])?;
    assert!(!order_no.is_empty());

    if let Some(org_no) = optional_output_field(&order.output, &["KRX_FWDG_ORD_ORGNO"]) {
        client
            .domestic_stock()
            .trading()
            .order_rvsecncl(
                &token.access_token,
                DomesticOrderRvsecnclRequest::cancel(
                    org_no,
                    order_no,
                    "00",
                    &quantity,
                    "0",
                    AllQuantityOrder::All,
                    "KRX",
                )?,
            )
            .await?;
    } else {
        eprintln!("domestic virtual cancel skipped: KRX_FWDG_ORD_ORGNO is missing in order output");
    }

    Ok(())
}

#[tokio::test]
#[ignore = "requires KIS_ENABLE_VIRTUAL_ORDER_SMOKE=true; creates a virtual overseas stock order"]
async fn live_smoke_virtual_overseas_buy_order_and_cancel() -> Result<()> {
    ensure_virtual_order_smoke_enabled()?;

    let client = virtual_client_with_account()?;
    let token = client.issue_token().await?;
    let exchange = OverseasExchange::from_kis_code(&optional_env("KIS_OVERSEAS_EXCHANGE", "NASD"))?;
    let stock_code = OverseasStockCode::new(optional_env("KIS_OVERSEAS_STOCK_CODE", "AAPL"))?;
    let quantity = limited_quantity("KIS_VIRTUAL_ORDER_QTY")?;
    let price = positive_price_env("KIS_VIRTUAL_OVERSEAS_ORDER_PRICE")?;

    let order = client
        .overseas_stock()
        .trading()
        .order(
            &token.access_token,
            OverseasOrderRequest::buy(exchange, stock_code.clone(), &quantity, &price, "00")?,
        )
        .await?;
    let order_no = required_output_field(&order.output, &["ODNO"])?;
    assert!(!order_no.is_empty());

    client
        .overseas_stock()
        .trading()
        .order_rvsecncl(
            &token.access_token,
            OverseasOrderRvsecnclRequest::cancel(exchange, stock_code, order_no, quantity, "0")?,
        )
        .await?;

    Ok(())
}

#[cfg(feature = "websocket-client")]
#[tokio::test]
#[ignore = "requires KIS_ENABLE_WS_SMOKE=true; opens a live KIS WebSocket connection"]
async fn live_smoke_websocket_domestic_price_first_frame() -> Result<()> {
    use std::time::Duration;

    use kis_rs_client::websocket::{
        DomesticRealtimePriceMarket, IncomingFrame, SubscriptionAction, SubscriptionBook,
        WebSocketClient, domestic,
    };
    use tokio::time::timeout;

    if env::var("KIS_ENABLE_WS_SMOKE").as_deref() != Ok("true") {
        eprintln!("set KIS_ENABLE_WS_SMOKE=true to run the live WebSocket smoke test");
        return Ok(());
    }

    let client = client_without_account()?;
    let approval = client.issue_approval_key().await?;
    let mut book = SubscriptionBook::new();
    book.add(domestic::realtime_price_subscription(
        SubscriptionAction::Subscribe,
        DomesticRealtimePriceMarket::Krx,
        optional_env("KIS_STOCK_CODE", "005930"),
    )?);

    let websocket = WebSocketClient::new(client.websocket_base_url());
    let mut session = websocket
        .connect_with_subscriptions(&approval.approval_key, &book)
        .await?;

    let frame = timeout(Duration::from_secs(10), session.next_frame())
        .await
        .map_err(|_| Error::http("websocket smoke timed out waiting for first frame"))??;
    assert!(matches!(
        frame,
        Some(IncomingFrame::Data(_) | IncomingFrame::System(_))
    ));

    session.close().await?;

    Ok(())
}

fn client_without_account() -> Result<Client<ReqwestHttpClient>> {
    base_builder()?.build_reqwest()
}

fn client_with_account() -> Result<Client<ReqwestHttpClient>> {
    base_builder()?
        .account(
            required_env("KIS_ACCOUNT_NO")?,
            required_env("KIS_ACCOUNT_PRODUCT_CODE")?,
        )?
        .build_reqwest()
}

fn virtual_client_with_account() -> Result<Client<ReqwestHttpClient>> {
    Client::builder()
        .virtual_trading()
        .credentials(
            required_env("KIS_APP_KEY")?,
            required_env("KIS_APP_SECRET")?,
        )?
        .account(
            required_env("KIS_ACCOUNT_NO")?,
            required_env("KIS_ACCOUNT_PRODUCT_CODE")?,
        )?
        .build_reqwest()
}

fn base_builder() -> Result<ClientBuilder> {
    let builder = Client::builder().credentials(
        required_env("KIS_APP_KEY")?,
        required_env("KIS_APP_SECRET")?,
    )?;

    Ok(
        if env::var("KIS_USE_VIRTUAL").map_or(true, |value| value != "false") {
            builder.virtual_trading()
        } else {
            builder.real()
        },
    )
}

fn stock_code() -> Result<StockCode> {
    StockCode::new(optional_env("KIS_STOCK_CODE", "005930"))
}

fn smoke_time() -> String {
    optional_env("KIS_SMOKE_TIME", "153000")
}

fn required_env(name: &'static str) -> Result<String> {
    env::var(name).map_err(|_| Error::config(format!("{name} is required")))
}

fn optional_env(name: &str, default: &str) -> String {
    env::var(name).unwrap_or_else(|_| default.to_string())
}

fn ensure_virtual_order_smoke_enabled() -> Result<()> {
    if env::var("KIS_ENABLE_VIRTUAL_ORDER_SMOKE").as_deref() == Ok("true") {
        Ok(())
    } else {
        Err(Error::config(
            "KIS_ENABLE_VIRTUAL_ORDER_SMOKE=true is required for virtual order smoke tests",
        ))
    }
}

fn limited_quantity(name: &'static str) -> Result<String> {
    let value = env::var(name).unwrap_or_else(|_| "1".to_string());
    let quantity = value
        .parse::<u32>()
        .map_err(|_| Error::config(format!("{name} must be a positive integer")))?;

    if (1..=10).contains(&quantity) {
        Ok(value)
    } else {
        Err(Error::config(format!("{name} must be between 1 and 10")))
    }
}

fn positive_price_env(name: &'static str) -> Result<String> {
    let value = required_env(name)?;
    let price = value
        .parse::<Decimal>()
        .map_err(|_| Error::config(format!("{name} must be a positive decimal")))?;

    if price > Decimal::ZERO {
        Ok(value)
    } else {
        Err(Error::config(format!("{name} must be greater than 0")))
    }
}

fn required_output_field(output: &serde_json::Value, names: &[&str]) -> Result<String> {
    optional_output_field(output, names).ok_or_else(|| {
        Error::parse(format!(
            "virtual order output is missing one of required fields: {}",
            names.join(", ")
        ))
    })
}

fn optional_output_field(output: &serde_json::Value, names: &[&str]) -> Option<String> {
    names
        .iter()
        .find_map(|name| output.get(*name).and_then(serde_json::Value::as_str))
        .map(str::to_string)
}
