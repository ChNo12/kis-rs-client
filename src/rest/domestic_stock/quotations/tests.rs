use super::common::{
    FID_ETC_CLS_CODE, FID_FAKE_TICK_INCU_YN, FID_INPUT_DATE_1, FID_INPUT_DATE_2, FID_INPUT_HOUR_1,
    FID_ORG_ADJ_PRC, FID_PERIOD_DIV_CODE, FID_PW_DATA_INCU_YN,
};
use super::*;
use crate::Client;
use crate::auth::AccessToken;
use crate::config::{Config, Credentials, Environment};
use crate::error::{Error, Result};
use crate::http::{HttpClient, Method, Request, Response};
use crate::rest::domestic_stock::common::{
    FID_COND_MRKT_DIV_CODE, FID_INPUT_ISCD, MarketDivision, StockCode,
};
use async_trait::async_trait;
use rust_decimal::Decimal;
use serde_json::Value;
use std::sync::Mutex;

#[derive(Debug)]
struct MockHttpClient {
    response: Response,
    requests: Mutex<Vec<Request>>,
}

impl MockHttpClient {
    fn new(response: Response) -> Self {
        Self {
            response,
            requests: Mutex::new(Vec::new()),
        }
    }

    fn requests(&self) -> Vec<Request> {
        self.requests.lock().unwrap().clone()
    }
}

#[async_trait]
impl HttpClient for &MockHttpClient {
    async fn send(&self, request: Request) -> Result<Response> {
        self.requests.lock().unwrap().push(request);
        Ok(self.response.clone())
    }
}

#[tokio::test]
async fn inquire_price_sends_kis_request() {
    let http_client = MockHttpClient::new(Response::new(
        200,
        r#"{
            "rt_cd": "0",
            "msg_cd": "MCA00000",
            "msg1": "ok",
            "output": {
                "stck_prpr": "70500.5",
                "prdy_vrss": "100",
                "prdy_vrss_sign": "2",
                "prdy_ctrt": "0.14",
                "acml_vol": "123456",
                "acml_tr_pbmn": "8700000000"
            }
        }"#,
    ));
    let client = mock_client(&http_client);
    let access_token = AccessToken::new("access-token-value");
    let request = InquirePriceRequest::new(MarketDivision::Krx, StockCode::new("005930").unwrap());

    let response = client
        .domestic_stock()
        .quotations()
        .inquire_price(&access_token, request)
        .await
        .unwrap();

    assert_eq!(response.current_price(), Some("70500.5"));
    let typed = response.typed().unwrap();
    assert_eq!(typed.current_price, Decimal::new(705005, 1));
    assert_eq!(typed.previous_day_rate, Decimal::new(14, 2));

    let request = only_request(&http_client);
    assert_eq!(request.method(), Method::Get);
    assert_eq!(
        request.url(),
        "https://openapivts.koreainvestment.com:29443/uapi/domestic-stock/v1/quotations/inquire-price"
    );
    assert_eq!(
        request.query_params(),
        &[
            (FID_COND_MRKT_DIV_CODE.to_string(), "J".to_string()),
            (FID_INPUT_ISCD.to_string(), "005930".to_string())
        ]
    );
    assert_header(&request, "tr_id", INQUIRE_PRICE_TR_ID);
    assert_header(&request, "tr_cont", "");
}

#[tokio::test]
async fn inquire_asking_price_exp_ccn_parses_output_pair() {
    let http_client = MockHttpClient::new(Response::new(
        200,
        r#"{
            "rt_cd": "0",
            "msg_cd": "MCA00000",
            "msg1": "ok",
            "output1": {
                "askp1": "70600",
                "bidp1": "70400",
                "askp_rsqn1": "1200",
                "bidp_rsqn1": "900",
                "total_askp_rsqn": "10000",
                "total_bidp_rsqn": "9000"
            },
            "output2": {
                "antc_cnpr": "70500",
                "antc_cntg_vrss": "100",
                "antc_cntg_vol": "500"
            }
        }"#,
    ));
    let client = mock_client(&http_client);
    let access_token = AccessToken::new("access-token-value");
    let request = InquireAskingPriceExpCcnRequest::new(
        MarketDivision::Unified,
        StockCode::new("005930").unwrap(),
    );

    let response = client
        .domestic_stock()
        .quotations()
        .inquire_asking_price_exp_ccn(&access_token, request)
        .await
        .unwrap();

    assert_eq!(
        response.output1.get("askp1").and_then(Value::as_str),
        Some("70600")
    );
    assert_eq!(
        response.output2.get("antc_cnpr").and_then(Value::as_str),
        Some("70500")
    );
    let typed = response.typed().unwrap();
    assert_eq!(typed.asking_price.ask_price1, decimal(70600));
    assert_eq!(
        typed.expected_conclusion.expected_conclusion_price,
        decimal(70500)
    );

    let request = only_request(&http_client);
    assert_eq!(
        request.url(),
        "https://openapivts.koreainvestment.com:29443/uapi/domestic-stock/v1/quotations/inquire-asking-price-exp-ccn"
    );
    assert_eq!(
        request.query_params(),
        &[
            (FID_COND_MRKT_DIV_CODE.to_string(), "UN".to_string()),
            (FID_INPUT_ISCD.to_string(), "005930".to_string())
        ]
    );
    assert_header(&request, "tr_id", INQUIRE_ASKING_PRICE_EXP_CCN_TR_ID);
}

#[tokio::test]
async fn inquire_ccnl_parses_typed_items() {
    let http_client = MockHttpClient::new(Response::new(
        200,
        r#"{
            "rt_cd": "0",
            "msg_cd": "MCA00000",
            "msg1": "ok",
            "output": [{
                "stck_cntg_hour": "093000",
                "stck_prpr": "70500",
                "prdy_vrss": "100",
                "prdy_vrss_sign": "2",
                "cntg_vol": "10"
            }]
        }"#,
    ));
    let client = mock_client(&http_client);
    let access_token = AccessToken::new("access-token-value");
    let request = InquireCcnlRequest::new(MarketDivision::Krx, StockCode::new("005930").unwrap());

    let response = client
        .domestic_stock()
        .quotations()
        .inquire_ccnl(&access_token, request)
        .await
        .unwrap();

    let typed = response.typed().unwrap();
    assert_eq!(typed[0].conclusion_time, "093000");
    assert_eq!(typed[0].current_price, decimal(70500));

    let request = only_request(&http_client);
    assert_eq!(
        request.url(),
        "https://openapivts.koreainvestment.com:29443/uapi/domestic-stock/v1/quotations/inquire-ccnl"
    );
    assert_header(&request, "tr_id", INQUIRE_CCNL_TR_ID);
}

#[tokio::test]
async fn inquire_daily_price_sends_period_params() {
    let http_client = MockHttpClient::new(Response::new(
        200,
        r#"{
            "rt_cd": "0",
            "msg_cd": "MCA00000",
            "msg1": "ok",
            "output": [{
                "stck_bsop_date": "20260511",
                "stck_oprc": "70000",
                "stck_hgpr": "70700",
                "stck_lwpr": "69900",
                "stck_clpr": "70500",
                "acml_vol": "123456",
                "acml_tr_pbmn": "8700000000"
            }]
        }"#,
    ));
    let client = mock_client(&http_client);
    let access_token = AccessToken::new("access-token-value");
    let request =
        InquireDailyPriceRequest::new(MarketDivision::Krx, StockCode::new("005930").unwrap())
            .with_period_division_code("D")
            .with_adjusted_price_code("1");

    let response = client
        .domestic_stock()
        .quotations()
        .inquire_daily_price(&access_token, request)
        .await
        .unwrap();

    assert!(response.output.is_array());
    let typed = response.typed().unwrap();
    assert_eq!(typed[0].business_date, "20260511");
    assert_eq!(typed[0].close_price, decimal(70500));

    let request = only_request(&http_client);
    assert_eq!(
        request.query_params(),
        &[
            (FID_COND_MRKT_DIV_CODE.to_string(), "J".to_string()),
            (FID_INPUT_ISCD.to_string(), "005930".to_string()),
            (FID_PERIOD_DIV_CODE.to_string(), "D".to_string()),
            (FID_ORG_ADJ_PRC.to_string(), "1".to_string())
        ]
    );
    assert_header(&request, "tr_id", INQUIRE_DAILY_PRICE_TR_ID);
}

#[tokio::test]
async fn inquire_daily_item_chart_price_sends_date_params() {
    let http_client = MockHttpClient::new(Response::new(
        200,
        r#"{
            "rt_cd": "0",
            "msg_cd": "MCA00000",
            "msg1": "ok",
            "output1": {
                "stck_prpr": "70500",
                "prdy_vrss": "100",
                "prdy_vrss_sign": "2",
                "prdy_ctrt": "0.14"
            },
            "output2": [{
                "stck_bsop_date": "20260511",
                "stck_oprc": "70000",
                "stck_hgpr": "70700",
                "stck_lwpr": "69900",
                "stck_clpr": "70500",
                "acml_vol": "123456",
                "acml_tr_pbmn": "8700000000"
            }]
        }"#,
    ));
    let client = mock_client(&http_client);
    let access_token = AccessToken::new("access-token-value");
    let request = InquireDailyItemChartPriceRequest::new(
        MarketDivision::Krx,
        StockCode::new("005930").unwrap(),
        "20260501",
        "20260511",
    )
    .with_period_division_code("D")
    .with_adjusted_price_code("1");

    let response = client
        .domestic_stock()
        .quotations()
        .inquire_daily_item_chart_price(&access_token, request)
        .await
        .unwrap();

    let typed = response.typed().unwrap();
    assert_eq!(typed.summary.previous_day_difference, decimal(100));
    assert_eq!(typed.summary.previous_day_rate, Decimal::new(14, 2));
    assert_eq!(typed.prices[0].business_date, "20260511");

    let request = only_request(&http_client);
    assert_eq!(
        request.query_params(),
        &[
            (FID_COND_MRKT_DIV_CODE.to_string(), "J".to_string()),
            (FID_INPUT_ISCD.to_string(), "005930".to_string()),
            (FID_INPUT_DATE_1.to_string(), "20260501".to_string()),
            (FID_INPUT_DATE_2.to_string(), "20260511".to_string()),
            (FID_PERIOD_DIV_CODE.to_string(), "D".to_string()),
            (FID_ORG_ADJ_PRC.to_string(), "1".to_string())
        ]
    );
    assert_header(&request, "tr_id", INQUIRE_DAILY_ITEM_CHART_PRICE_TR_ID);
}

#[tokio::test]
async fn inquire_time_item_chart_price_sends_hour_params() {
    let http_client = MockHttpClient::new(Response::new(
        200,
        r#"{
            "rt_cd": "0",
            "msg_cd": "MCA00000",
            "msg1": "ok",
            "output1": {
                "stck_prpr": "70500",
                "prdy_vrss": "100",
                "prdy_vrss_sign": "2",
                "prdy_ctrt": "0.14"
            },
            "output2": [{
                "stck_cntg_hour": "093000",
                "stck_prpr": "70500",
                "stck_oprc": "70000",
                "stck_hgpr": "70700",
                "stck_lwpr": "69900",
                "cntg_vol": "10",
                "acml_vol": "123456"
            }]
        }"#,
    ));
    let client = mock_client(&http_client);
    let access_token = AccessToken::new("access-token-value");
    let request = InquireTimeItemChartPriceRequest::new(
        MarketDivision::Krx,
        StockCode::new("005930").unwrap(),
        "093000",
    )
    .include_past_data(true);

    let response = client
        .domestic_stock()
        .quotations()
        .inquire_time_item_chart_price(&access_token, request)
        .await
        .unwrap();

    let typed = response.typed().unwrap();
    assert_eq!(typed.summary.current_price, decimal(70500));
    assert_eq!(typed.summary.previous_day_rate, Decimal::new(14, 2));
    assert_eq!(typed.items[0].conclusion_time, "093000");

    let request = only_request(&http_client);
    assert_eq!(
        request.query_params(),
        &[
            (FID_COND_MRKT_DIV_CODE.to_string(), "J".to_string()),
            (FID_INPUT_ISCD.to_string(), "005930".to_string()),
            (FID_INPUT_HOUR_1.to_string(), "093000".to_string()),
            (FID_PW_DATA_INCU_YN.to_string(), "Y".to_string()),
            (FID_ETC_CLS_CODE.to_string(), "".to_string())
        ]
    );
    assert_header(&request, "tr_id", INQUIRE_TIME_ITEM_CHART_PRICE_TR_ID);
}

#[tokio::test]
async fn inquire_time_daily_chart_price_sends_date_and_hour_params() {
    let http_client = MockHttpClient::new(Response::new(
        200,
        r#"{
            "rt_cd": "0",
            "msg_cd": "MCA00000",
            "msg1": "ok",
            "output1": {
                "stck_prpr": "70500",
                "prdy_vrss": "100",
                "prdy_vrss_sign": "2",
                "prdy_ctrt": "0.14"
            },
            "output2": [{
                "stck_cntg_hour": "130000",
                "stck_prpr": "70500",
                "stck_oprc": "70000",
                "stck_hgpr": "70700",
                "stck_lwpr": "69900",
                "cntg_vol": "10",
                "acml_vol": "123456"
            }]
        }"#,
    ));
    let client = mock_client(&http_client);
    let access_token = AccessToken::new("access-token-value");
    let request = InquireTimeDailyChartPriceRequest::new(
        MarketDivision::Krx,
        StockCode::new("005930").unwrap(),
        "130000",
        "20241023",
    )
    .include_past_data(true)
    .include_fake_tick(true);

    let response = client
        .domestic_stock()
        .quotations()
        .inquire_time_daily_chart_price(&access_token, request)
        .await
        .unwrap();

    let typed = response.typed().unwrap();
    assert_eq!(typed.summary.current_price, decimal(70500));
    assert_eq!(typed.items[0].conclusion_time, "130000");

    let request = only_request(&http_client);
    assert_eq!(
        request.query_params(),
        &[
            (FID_COND_MRKT_DIV_CODE.to_string(), "J".to_string()),
            (FID_INPUT_ISCD.to_string(), "005930".to_string()),
            (FID_INPUT_HOUR_1.to_string(), "130000".to_string()),
            (FID_INPUT_DATE_1.to_string(), "20241023".to_string()),
            (FID_PW_DATA_INCU_YN.to_string(), "Y".to_string()),
            (FID_FAKE_TICK_INCU_YN.to_string(), "Y".to_string())
        ]
    );
    assert_header(&request, "tr_id", INQUIRE_TIME_DAILY_CHART_PRICE_TR_ID);
}

#[tokio::test]
async fn rejects_kis_error_body() {
    let http_client = MockHttpClient::new(Response::new(
        200,
        r#"{
            "rt_cd": "1",
            "msg_cd": "MCA12345",
            "msg1": "invalid stock code"
        }"#,
    ));
    let client = mock_client(&http_client);
    let access_token = AccessToken::new("access-token-value");
    let request = InquirePriceRequest::new(MarketDivision::Krx, StockCode::new("005930").unwrap());

    let error = client
        .domestic_stock()
        .quotations()
        .inquire_price(&access_token, request)
        .await
        .unwrap_err();

    assert_eq!(
        error,
        Error::api(Some("MCA12345".to_string()), "invalid stock code")
    );
}

#[tokio::test]
async fn rejects_missing_output() {
    let http_client = MockHttpClient::new(Response::new(
        200,
        r#"{
            "rt_cd": "0",
            "msg_cd": "MCA00000",
            "msg1": "ok"
        }"#,
    ));
    let client = mock_client(&http_client);
    let access_token = AccessToken::new("access-token-value");
    let request = InquirePriceRequest::new(MarketDivision::Krx, StockCode::new("005930").unwrap());

    let error = client
        .domestic_stock()
        .quotations()
        .inquire_price(&access_token, request)
        .await
        .unwrap_err();

    assert!(matches!(error, Error::Parse { .. }));
}

#[tokio::test]
async fn typed_output_rejects_missing_required_field() {
    let http_client = MockHttpClient::new(Response::new(
        200,
        r#"{
            "rt_cd": "0",
            "msg_cd": "MCA00000",
            "msg1": "ok",
            "output": {
                "prdy_vrss": "100",
                "prdy_vrss_sign": "2",
                "prdy_ctrt": "0.14",
                "acml_vol": "123456",
                "acml_tr_pbmn": "8700000000"
            }
        }"#,
    ));
    let client = mock_client(&http_client);
    let access_token = AccessToken::new("access-token-value");
    let request = InquirePriceRequest::new(MarketDivision::Krx, StockCode::new("005930").unwrap());

    let response = client
        .domestic_stock()
        .quotations()
        .inquire_price(&access_token, request)
        .await
        .unwrap();

    assert!(matches!(response.typed(), Err(Error::Parse { .. })));
}

#[tokio::test]
async fn typed_output_rejects_invalid_numeric_field() {
    let http_client = MockHttpClient::new(Response::new(
        200,
        r#"{
            "rt_cd": "0",
            "msg_cd": "MCA00000",
            "msg1": "ok",
            "output": {
                "stck_prpr": "not-a-number",
                "prdy_vrss": "100",
                "prdy_vrss_sign": "2",
                "prdy_ctrt": "0.14",
                "acml_vol": "123456",
                "acml_tr_pbmn": "8700000000"
            }
        }"#,
    ));
    let client = mock_client(&http_client);
    let access_token = AccessToken::new("access-token-value");
    let request = InquirePriceRequest::new(MarketDivision::Krx, StockCode::new("005930").unwrap());

    let response = client
        .domestic_stock()
        .quotations()
        .inquire_price(&access_token, request)
        .await
        .unwrap();

    assert!(matches!(response.typed(), Err(Error::Parse { .. })));
}

fn mock_client(http_client: &MockHttpClient) -> Client<&MockHttpClient> {
    let credentials = Credentials::new("app-key", "app-secret").unwrap();
    let config = Config::new(Environment::Virtual, credentials);

    Client::new(config, http_client)
}

fn only_request(http_client: &MockHttpClient) -> Request {
    let requests = http_client.requests();
    assert_eq!(requests.len(), 1);
    requests[0].clone()
}

fn assert_header(request: &Request, name: &str, value: &str) {
    assert!(
        request
            .headers()
            .iter()
            .any(|header| header.name() == name && header.value() == value),
        "missing header {name}: {value}"
    );
}

fn decimal(value: i64) -> Decimal {
    Decimal::new(value, 0)
}
