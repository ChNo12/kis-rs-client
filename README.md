# kis-rs-client

한국투자증권(KIS) REST API와 WebSocket API를 Rust에서 호출하기 위한 async-first client crate다.

현재 v1 구현 범위는 필요한 기능에 맞춰 좁혀져 있다.
구현 현황은 [`REST_CATALOG.md`](REST_CATALOG.md)와
[`WEBSOCKET_CATALOG.md`](WEBSOCKET_CATALOG.md)에서 추적한다.

- OAuth access token 발급: `/oauth2/tokenP`
- WebSocket approval key 발급: `/oauth2/Approval`
- 국내주식 조회:
  - 현재가 시세: `/uapi/domestic-stock/v1/quotations/inquire-price`
  - 호가/예상체결: `/uapi/domestic-stock/v1/quotations/inquire-asking-price-exp-ccn`
  - 현재가 체결: `/uapi/domestic-stock/v1/quotations/inquire-ccnl`
  - 현재가 일자별: `/uapi/domestic-stock/v1/quotations/inquire-daily-price`
  - 기간별 시세: `/uapi/domestic-stock/v1/quotations/inquire-daily-itemchartprice`
  - 당일 분봉: `/uapi/domestic-stock/v1/quotations/inquire-time-itemchartprice`
  - 이전일자 분봉: `/uapi/domestic-stock/v1/quotations/inquire-time-dailychartprice`
- 국내주식 주문/조회:
  - 현금 주문: `/uapi/domestic-stock/v1/trading/order-cash`
  - 주문 정정/취소: `/uapi/domestic-stock/v1/trading/order-rvsecncl`
  - 정정취소 가능주문조회: `/uapi/domestic-stock/v1/trading/inquire-psbl-rvsecncl`
  - 일별주문체결조회: `/uapi/domestic-stock/v1/trading/inquire-daily-ccld`
- 해외주식 주문/조회:
  - 미국 `NASD`, `NYSE`, `AMEX` 주문
  - 미국 주문 정정/취소
  - 해외 주문체결내역 조회
- WebSocket:
  - 국내 실시간 가격 수신 구독 메시지, 수신 frame parser, typed view
  - 국내 실시간 체결통보 수신, AES-256-CBC/base64 복호화, typed view
  - 해외 실시간 체결통보 수신, AES-256-CBC/base64 복호화, typed view
- KIS 공통 오류 응답 처리: `rt_cd`, `msg_cd`, `msg1`
- KIS 연속조회 헤더 보존: `tr_cont`, `ctx_area_fk`, `ctx_area_nk`
- secret/account/debug 마스킹

토큰 저장과 자동 갱신은 구현하지 않는다. 호출자가 발급된 토큰과 approval key를 보관하고 필요한 API 호출에 명시적으로 넘긴다.

## Install

```toml
[dependencies]
kis-rs-client = { path = "." }
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

기본 feature는 `reqwest-client`를 포함한다. 실제 WebSocket 연결과 체결통보 복호화까지 사용하려면 `websocket-client` feature를 켠다.

```toml
[dependencies]
kis-rs-client = { path = ".", features = ["websocket-client"] }
```

## Quote Example

```rust
use kis_rs_client::rest::domestic_stock::{
    InquirePriceRequest, MarketDivision, StockCode,
};
use kis_rs_client::Client;

#[tokio::main]
async fn main() -> kis_rs_client::Result<()> {
    let client = Client::builder()
        .virtual_trading()
        .credentials("app-key", "app-secret")?
        .build_reqwest()?;

    let token = client.issue_token().await?;
    let request = InquirePriceRequest::new(
        MarketDivision::Krx,
        StockCode::new("005930")?,
    );
    let response = client
        .domestic_stock()
        .quotations()
        .inquire_price(&token.access_token, request)
        .await?;

    let quote = response.typed()?;

    println!("current price: {}", quote.current_price);

    Ok(())
}
```

환경 변수로 실행:

```bash
export KIS_APP_KEY="..."
export KIS_APP_SECRET="..."
export KIS_USE_VIRTUAL=true
export KIS_STOCK_CODE=005930

cargo run --example quote
```

## Pagination

연속조회가 있는 API는 단일 페이지 응답의 `continuation`을 그대로 노출한다. 필요한 경우
`*_pages` helper로 여러 페이지를 수집할 수 있다.

```rust
use kis_rs_client::rest::{PageLimit, PageStopReason};

# async fn run(client: kis_rs_client::Client<kis_rs_client::ReqwestHttpClient>, token: kis_rs_client::AccessToken) -> kis_rs_client::Result<()> {
let request = kis_rs_client::rest::domestic_stock::ranking::AfterHourBalanceRequest::new();
let pages = client
    .domestic_stock()
    .ranking()
    .after_hour_balance_pages(&token, request, PageLimit::Max(3))
    .await?;

match pages.stop_reason {
    PageStopReason::Exhausted => {}
    PageStopReason::PageLimitReached => {
        let next = pages.next;
        println!("resume continuation: {next:?}");
    }
    PageStopReason::RateLimited { error } => {
        let next = pages.next;
        println!("rate limited: {error}; resume continuation: {next:?}");
    }
}
# Ok(())
# }
```

`PageLimit::Max(n)`은 첫 페이지를 포함한 최대 호출 횟수다. `PageLimit::All`은
`continuation`이 끝날 때까지 수집한다. HTTP 429를 만나면 에러로 중단하지 않고 지금까지
수집한 page와 재개용 continuation을 반환한다. KIS 유량제어 오류 코드 `EGW00201`도 같은 방식으로
처리한다.

## Order Safety

모의투자 주문은 기본 허용된다. 실전 주문/정정/취소는 `.real()`만으로는 실행되지 않고 별도 opt-in이 필요하다.

```rust
let client = Client::builder()
    .real()
    .enable_real_ordering()
    .credentials("app-key", "app-secret")?
    .account("12345678", "01")?
    .build_reqwest()?;
```

`enable_real_ordering()`이 없으면 실전 주문/정정/취소 API는 `Error::Config`를 반환한다. 조회 API는 이 opt-in 없이 사용할 수 있다.

## WebSocket Sketch

```rust
use kis_rs_client::Client;
use kis_rs_client::websocket::{
    DomesticRealtimePrice, DomesticRealtimePriceMarket, IncomingFrame,
    SubscriptionAction, SubscriptionBook, WebSocketClient, domestic,
};

# async fn run() -> kis_rs_client::Result<()> {
let client = Client::builder()
    .virtual_trading()
    .credentials("app-key", "app-secret")?
    .build_reqwest()?;
let approval = client.issue_approval_key().await?;

let mut book = SubscriptionBook::new();
book.add(domestic::realtime_price_subscription(
    SubscriptionAction::Subscribe,
    DomesticRealtimePriceMarket::Krx,
    "005930",
)?);

let ws = WebSocketClient::new(client.websocket_base_url());
let mut session = ws
    .connect_with_subscriptions(&approval.approval_key, &book)
    .await?;

while let Some(frame) = session.next_frame().await? {
    match frame {
        IncomingFrame::Data(data) => {
            if let Some(price) = DomesticRealtimePrice::from_frame(&data)? {
                println!("{} {:?}", price.stock_code, price.current_price);
            }
        }
        IncomingFrame::System(message) => {
            println!("{message:?}");
        }
    }
}
# Ok(())
# }
```

## Live Smoke Test

실제 KIS API 호출은 기본 테스트에서 제외되어 있다. 필요할 때만 명시적으로 실행한다.

```bash
KIS_APP_KEY="..." \
KIS_APP_SECRET="..." \
KIS_USE_VIRTUAL=true \
cargo test --test live_smoke live_smoke_readonly_domestic_quote_and_today_minute -- --ignored
```

스모크 테스트는 기본적으로 조회만 호출한다. 모의 주문 smoke는 별도 환경 변수로 명시적으로 켤 때만
모의 계좌에 주문을 생성한다. 실전 주문과 체결통보 WebSocket 수신 테스트는 자동화하지 않는다.

```bash
# 이전일자 분봉 조회
KIS_APP_KEY="..." \
KIS_APP_SECRET="..." \
KIS_SMOKE_DATE=20260508 \
cargo test --test live_smoke live_smoke_readonly_domestic_previous_day_minute -- --ignored

# 국내/해외 체결여부 조회
KIS_APP_KEY="..." \
KIS_APP_SECRET="..." \
KIS_ACCOUNT_NO="..." \
KIS_ACCOUNT_PRODUCT_CODE="..." \
KIS_SMOKE_START_DATE=20260508 \
KIS_SMOKE_END_DATE=20260508 \
cargo test --test live_smoke live_smoke_readonly_order_conclusions -- --ignored

# 모의 국내 매수 주문 및 가능한 경우 취소
KIS_APP_KEY="..." \
KIS_APP_SECRET="..." \
KIS_ACCOUNT_NO="..." \
KIS_ACCOUNT_PRODUCT_CODE="..." \
KIS_ENABLE_VIRTUAL_ORDER_SMOKE=true \
KIS_VIRTUAL_DOMESTIC_ORDER_PRICE=50000 \
cargo test --test live_smoke live_smoke_virtual_domestic_buy_order_and_best_effort_cancel -- --ignored

# 모의 해외 매수 주문 및 취소
KIS_APP_KEY="..." \
KIS_APP_SECRET="..." \
KIS_ACCOUNT_NO="..." \
KIS_ACCOUNT_PRODUCT_CODE="..." \
KIS_ENABLE_VIRTUAL_ORDER_SMOKE=true \
KIS_VIRTUAL_OVERSEAS_ORDER_PRICE=100.00 \
cargo test --test live_smoke live_smoke_virtual_overseas_buy_order_and_cancel -- --ignored

# 국내 실시간 가격 WebSocket 1프레임 수신
KIS_APP_KEY="..." \
KIS_APP_SECRET="..." \
KIS_ENABLE_WS_SMOKE=true \
cargo test --features websocket-client --test live_smoke live_smoke_websocket_domestic_price_first_frame -- --ignored
```

## Development

```bash
cargo fmt --all
cargo test --all-features
cargo build --all-features
cargo build --examples --all-features
cargo clippy --all-targets --all-features -- -D warnings
```
