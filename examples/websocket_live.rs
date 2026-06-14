#[cfg(all(feature = "reqwest-client", feature = "websocket-client"))]
use std::env;

#[cfg(all(feature = "reqwest-client", feature = "websocket-client"))]
use kis_rs_client::websocket::{
    DOMESTIC_EXECUTION_NOTICE_REAL_TR_ID, DOMESTIC_EXECUTION_NOTICE_VIRTUAL_TR_ID,
    DomesticExecutionNotice, DomesticRealtimePrice, DomesticRealtimePriceMarket,
    ExecutionNoticeCipher, IncomingFrame, OVERSEAS_EXECUTION_NOTICE_REAL_TR_ID,
    OVERSEAS_EXECUTION_NOTICE_VIRTUAL_TR_ID, OverseasExecutionNotice, RealtimeDataFrame,
    SubscriptionAction, SubscriptionBook, SystemMessage, WebSocketClient, domestic, overseas,
};
#[cfg(all(feature = "reqwest-client", feature = "websocket-client"))]
use kis_rs_client::{Client, Environment, Error, Result};

#[cfg(all(feature = "reqwest-client", feature = "websocket-client"))]
#[tokio::main]
async fn main() -> Result<()> {
    let app_key = required_env("KIS_APP_KEY")?;
    let app_secret = required_env("KIS_APP_SECRET")?;
    let hts_id = required_env("KIS_HTS_ID")?;
    let stock_code = env::var("KIS_STOCK_CODE").unwrap_or_else(|_| "005930".to_string());
    let environment = environment_from_env();
    let price_market = price_market_from_env()?;
    let max_frames = max_frames_from_env()?;

    let client = Client::builder()
        .environment(environment)
        .credentials(app_key, app_secret)?
        .build_reqwest()?;

    let approval = client.issue_approval_key().await?;
    let mut book = SubscriptionBook::new();
    book.add(domestic::realtime_price_subscription(
        SubscriptionAction::Subscribe,
        price_market,
        stock_code.clone(),
    )?);
    book.add(domestic::execution_notice_subscription(
        SubscriptionAction::Subscribe,
        environment,
        hts_id.clone(),
    )?);
    book.add(overseas::execution_notice_subscription(
        SubscriptionAction::Subscribe,
        environment,
        hts_id,
    )?);

    println!(
        "connecting websocket url={} environment={environment:?} stock_code={stock_code} subscriptions={}",
        client.websocket_base_url(),
        book.len()
    );

    let websocket = WebSocketClient::new(client.websocket_base_url());
    let mut session = websocket
        .connect_with_subscriptions(&approval.approval_key, &book)
        .await?;
    let mut domestic_cipher = None;
    let mut overseas_cipher = None;
    let mut received = 0usize;

    loop {
        if max_frames.is_some_and(|max_frames| received >= max_frames) {
            break;
        }

        let Some(raw) = session.next_text().await? else {
            println!("websocket closed by server");
            break;
        };
        received += 1;

        let frame = IncomingFrame::parse(&raw)?;
        match frame {
            IncomingFrame::System(message) => {
                if message.is_ping_pong() {
                    session.send_pong_text(&raw).await?;
                    println!("[system] PINGPONG pong sent");
                    continue;
                }

                update_execution_cipher(&message, &mut domestic_cipher, &mut overseas_cipher)?;
                print_system_message(&message);
            }
            IncomingFrame::Data(frame) => {
                print_data_frame(&frame, domestic_cipher.as_ref(), overseas_cipher.as_ref());
            }
        }
    }

    session.close().await
}

#[cfg(all(feature = "reqwest-client", feature = "websocket-client"))]
fn print_data_frame(
    frame: &RealtimeDataFrame,
    domestic_cipher: Option<&ExecutionNoticeCipher>,
    overseas_cipher: Option<&ExecutionNoticeCipher>,
) {
    if frame.is_domestic_realtime_price() {
        match DomesticRealtimePrice::from_frame(frame) {
            Ok(Some(price)) => print_domestic_price(&price),
            Ok(None) => {}
            Err(error) => eprintln!("[domestic price parse error] {error}"),
        }
        return;
    }

    if frame.is_domestic_execution_notice() {
        let Some(cipher) = domestic_cipher else {
            eprintln!("[domestic execution] cipher is not ready yet");
            return;
        };

        match cipher.decrypt_base64(&frame.payload) {
            Ok(payload) => {
                print_raw_execution_payload("domestic execution", &payload);
                match DomesticExecutionNotice::parse(&payload) {
                    Ok(notice) => println!("[domestic execution] {notice:?}"),
                    Err(error) => eprintln!("[domestic execution parse error] {error}"),
                }
            }
            Err(error) => eprintln!("[domestic execution decrypt error] {error}"),
        }
        return;
    }

    if frame.is_overseas_execution_notice() {
        let Some(cipher) = overseas_cipher else {
            eprintln!("[overseas execution] cipher is not ready yet");
            return;
        };

        match cipher.decrypt_base64(&frame.payload) {
            Ok(payload) => {
                print_raw_execution_payload("overseas execution", &payload);
                match OverseasExecutionNotice::parse(&payload) {
                    Ok(notice) => println!("[overseas execution] {notice:?}"),
                    Err(error) => eprintln!("[overseas execution parse error] {error}"),
                }
            }
            Err(error) => eprintln!("[overseas execution decrypt error] {error}"),
        }
        return;
    }

    println!(
        "[data] tr_id={} tr_type={} record_count={} payload={}",
        frame.tr_id, frame.tr_type, frame.record_count, frame.payload
    );
}

#[cfg(all(feature = "reqwest-client", feature = "websocket-client"))]
fn print_raw_execution_payload(label: &str, payload: &str) {
    if env::var("KIS_DEBUG_EXECUTION_NOTICE_RAW").as_deref() != Ok("true") {
        return;
    }

    let fields = payload
        .split('^')
        .enumerate()
        .map(|(index, value)| format!("{index}:{value:?}"))
        .collect::<Vec<_>>()
        .join(", ");
    println!(
        "[{label} raw] field_count={} [{fields}]",
        payload.split('^').count()
    );
}

#[cfg(all(feature = "reqwest-client", feature = "websocket-client"))]
fn print_domestic_price(price: &DomesticRealtimePrice) {
    println!(
        "[domestic price] code={} time={} price={} diff={} rate={} volume={} ask={} bid={}",
        price.stock_code,
        price.stock_conclusion_time,
        option_text(&price.current_price),
        option_text(&price.previous_day_difference),
        option_text(&price.previous_day_rate),
        option_text(&price.accumulated_volume),
        option_text(&price.ask_price1),
        option_text(&price.bid_price1)
    );
}

#[cfg(all(feature = "reqwest-client", feature = "websocket-client"))]
fn update_execution_cipher(
    message: &SystemMessage,
    domestic_cipher: &mut Option<ExecutionNoticeCipher>,
    overseas_cipher: &mut Option<ExecutionNoticeCipher>,
) -> Result<()> {
    let Some(cipher) = ExecutionNoticeCipher::from_system_message(message)? else {
        return Ok(());
    };

    match message.header.tr_id.as_str() {
        DOMESTIC_EXECUTION_NOTICE_REAL_TR_ID | DOMESTIC_EXECUTION_NOTICE_VIRTUAL_TR_ID => {
            *domestic_cipher = Some(cipher);
            println!("[system] domestic execution cipher ready");
        }
        OVERSEAS_EXECUTION_NOTICE_REAL_TR_ID | OVERSEAS_EXECUTION_NOTICE_VIRTUAL_TR_ID => {
            *overseas_cipher = Some(cipher);
            println!("[system] overseas execution cipher ready");
        }
        _ => {}
    }

    Ok(())
}

#[cfg(all(feature = "reqwest-client", feature = "websocket-client"))]
fn print_system_message(message: &SystemMessage) {
    let status = if message.is_success() { "ok" } else { "not-ok" };
    let message_text = message
        .body
        .as_ref()
        .and_then(|body| body.msg1.as_deref())
        .unwrap_or("");

    println!(
        "[system] status={} tr_id={} tr_key={} message={}",
        status,
        message.header.tr_id,
        message.header.tr_key.as_deref().unwrap_or(""),
        message_text
    );
}

#[cfg(all(feature = "reqwest-client", feature = "websocket-client"))]
fn required_env(name: &'static str) -> Result<String> {
    env::var(name).map_err(|_| Error::config(format!("{name} is required")))
}

#[cfg(all(feature = "reqwest-client", feature = "websocket-client"))]
fn environment_from_env() -> Environment {
    if env::var("KIS_USE_VIRTUAL").map_or(true, |value| value != "false") {
        Environment::Virtual
    } else {
        Environment::Real
    }
}

#[cfg(all(feature = "reqwest-client", feature = "websocket-client"))]
fn price_market_from_env() -> Result<DomesticRealtimePriceMarket> {
    let value = env::var("KIS_WS_PRICE_MARKET").unwrap_or_else(|_| "KRX".to_string());

    match value.to_ascii_uppercase().as_str() {
        "KRX" => Ok(DomesticRealtimePriceMarket::Krx),
        "NXT" => Ok(DomesticRealtimePriceMarket::Nxt),
        "UNIFIED" => Ok(DomesticRealtimePriceMarket::Unified),
        _ => Err(Error::config(
            "KIS_WS_PRICE_MARKET must be one of KRX, NXT, UNIFIED",
        )),
    }
}

#[cfg(all(feature = "reqwest-client", feature = "websocket-client"))]
fn max_frames_from_env() -> Result<Option<usize>> {
    let Some(value) = env::var("KIS_WS_MAX_FRAMES").ok() else {
        return Ok(None);
    };

    if value == "0" {
        return Ok(None);
    }

    let max_frames = value
        .parse::<usize>()
        .map_err(|_| Error::config("KIS_WS_MAX_FRAMES must be a positive integer"))?;

    if max_frames == 0 {
        Ok(None)
    } else {
        Ok(Some(max_frames))
    }
}

#[cfg(all(feature = "reqwest-client", feature = "websocket-client"))]
fn option_text<T: std::fmt::Display>(value: &Option<T>) -> String {
    value
        .as_ref()
        .map(ToString::to_string)
        .unwrap_or_else(|| "-".to_string())
}

#[cfg(not(all(feature = "reqwest-client", feature = "websocket-client")))]
fn main() {
    eprintln!("enable reqwest-client and websocket-client features to run this example");
}
