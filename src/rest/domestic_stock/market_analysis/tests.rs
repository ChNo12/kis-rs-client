use super::*;
use crate::Client;
use crate::auth::AccessToken;
use crate::config::{Config, Credentials, Environment};
use crate::error::{Error, Result};
use crate::http::{HttpClient, Method, Request, Response};
use async_trait::async_trait;
use rust_decimal::Decimal;
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
async fn capture_up_low_price_sends_request_and_parses_typed_output() {
    let http_client = MockHttpClient::new(Response::new(
        200,
        r#"{
            "rt_cd": "0",
            "msg_cd": "MCA00000",
            "msg1": "ok",
            "output": [{
                "mksc_shrn_iscd": "005930",
                "hts_kor_isnm": "삼성전자",
                "stck_prpr": "70500.5",
                "prdy_vrss_sign": "2",
                "prdy_vrss": "100",
                "prdy_ctrt": "0.14",
                "acml_vol": "123456",
                "total_askp_rsqn": "1000",
                "total_bidp_rsqn": "900",
                "askp_rsqn1": "100",
                "bidp_rsqn1": "90",
                "prdy_vol": "111111",
                "seln_cnqn": "200",
                "shnu_cnqn": "300",
                "stck_llam": "49350",
                "stck_mxpr": "91650",
                "prdy_vrss_vol_rate": "1.12"
            }]
        }"#,
    ));
    let client = mock_client(&http_client);
    let access_token = AccessToken::new("access-token-value");
    let request = CaptureUpLowPriceRequest::new()
        .with_price_class_code("0")
        .with_division_class_code("6")
        .with_input_code("0001")
        .with_target_class_code("1")
        .with_target_exclusion_class_code("2")
        .with_price_range("1000", "100000")
        .with_volume_count("10000");

    let response = client
        .domestic_stock()
        .market_analysis()
        .capture_up_low_price(&access_token, request)
        .await
        .unwrap();

    let typed = response.typed().unwrap();
    assert_eq!(typed[0].stock_code, "005930");
    assert_eq!(typed[0].current_price, Decimal::new(705005, 1));
    assert_eq!(typed[0].upper_limit_price, Decimal::new(91650, 0));

    let request = only_request(&http_client);
    assert_eq!(request.method(), Method::Get);
    assert_eq!(
        request.url(),
        "https://openapivts.koreainvestment.com:29443/uapi/domestic-stock/v1/quotations/capture-uplowprice"
    );
    assert_eq!(
        request.query_params(),
        &[
            ("FID_COND_MRKT_DIV_CODE".to_string(), "J".to_string()),
            ("FID_COND_SCR_DIV_CODE".to_string(), "11300".to_string()),
            ("FID_PRC_CLS_CODE".to_string(), "0".to_string()),
            ("FID_DIV_CLS_CODE".to_string(), "6".to_string()),
            ("FID_INPUT_ISCD".to_string(), "0001".to_string()),
            ("FID_TRGT_CLS_CODE".to_string(), "1".to_string()),
            ("FID_TRGT_EXLS_CLS_CODE".to_string(), "2".to_string()),
            ("FID_INPUT_PRICE_1".to_string(), "1000".to_string()),
            ("FID_INPUT_PRICE_2".to_string(), "100000".to_string()),
            ("FID_VOL_CNT".to_string(), "10000".to_string())
        ]
    );
    assert_header(&request, "tr_id", CAPTURE_UP_LOW_PRICE_TR_ID);
    assert_header(&request, "tr_cont", "");
}

#[tokio::test]
async fn capture_up_low_price_rejects_invalid_typed_number() {
    let http_client = MockHttpClient::new(Response::new(
        200,
        r#"{
            "rt_cd": "0",
            "msg_cd": "MCA00000",
            "msg1": "ok",
            "output": [{
                "mksc_shrn_iscd": "005930",
                "hts_kor_isnm": "삼성전자",
                "stck_prpr": "not-a-number",
                "prdy_vrss_sign": "2",
                "prdy_vrss": "100",
                "prdy_ctrt": "0.14",
                "acml_vol": "123456",
                "total_askp_rsqn": "1000",
                "total_bidp_rsqn": "900",
                "askp_rsqn1": "100",
                "bidp_rsqn1": "90",
                "prdy_vol": "111111",
                "seln_cnqn": "200",
                "shnu_cnqn": "300",
                "stck_llam": "49350",
                "stck_mxpr": "91650",
                "prdy_vrss_vol_rate": "1.12"
            }]
        }"#,
    ));
    let client = mock_client(&http_client);
    let access_token = AccessToken::new("access-token-value");

    let response = client
        .domestic_stock()
        .market_analysis()
        .capture_up_low_price(&access_token, CaptureUpLowPriceRequest::new())
        .await
        .unwrap();

    assert!(matches!(response.typed(), Err(Error::Parse { .. })));
}

fn mock_client(http_client: &MockHttpClient) -> Client<&MockHttpClient> {
    let credentials = Credentials::new("app-key", "app-secret").unwrap();
    let config = Config::new(Environment::Mock, credentials);

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
