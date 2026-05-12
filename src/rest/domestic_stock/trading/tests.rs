use super::common::{
    ACNT_PRDT_CD, CANO, CCLD_DVSN, CTX_AREA_FK100, CTX_AREA_NK100, EXCG_ID_DVSN_CD, INQR_DVSN,
    INQR_DVSN_1, INQR_DVSN_2, INQR_DVSN_3, INQR_END_DT, INQR_STRT_DT, KRX_FWDG_ORD_ORGNO, ODNO,
    ORD_DVSN, ORD_GNO_BRNO, ORD_QTY, ORD_UNPR, ORGN_ODNO, PDNO, QTY_ALL_ORD_YN, RVSE_CNCL_DVSN_CD,
    SLL_BUY_DVSN_CD,
};
use super::*;
use crate::Client;
use crate::auth::AccessToken;
use crate::config::{Account, AccountNumber, Config, Credentials, Environment, ProductCode};
use crate::error::{Error, Result};
use crate::http::{Header, HttpClient, Method, Request, Response};
use crate::rest::domestic_stock::{Continuation, StockCode};
use async_trait::async_trait;
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
async fn order_cash_sends_virtual_buy_request() {
    let http_client = MockHttpClient::new(ok_output_response());
    let client = mock_client(&http_client);
    let access_token = AccessToken::new("access-token-value");
    let request =
        OrderCashRequest::buy(StockCode::new("005930").unwrap(), "00", "1", "70000", "KRX")
            .unwrap();

    let response = client
        .domestic_stock()
        .trading()
        .order_cash(&access_token, request)
        .await
        .unwrap();

    assert_eq!(
        response.output.get("ODNO").and_then(Value::as_str),
        Some("0000000001")
    );

    let request = only_request(&http_client);
    assert_eq!(request.method(), Method::Post);
    assert_eq!(
        request.url(),
        "https://openapivts.koreainvestment.com:29443/uapi/domestic-stock/v1/trading/order-cash"
    );
    assert_header(&request, "tr_id", ORDER_CASH_VIRTUAL_BUY_TR_ID);
    let body = request_body(&request);
    assert_eq!(body[CANO], "12345678");
    assert_eq!(body[ACNT_PRDT_CD], "01");
    assert_eq!(body[PDNO], "005930");
    assert_eq!(body[ORD_DVSN], "00");
    assert_eq!(body[ORD_QTY], "1");
    assert_eq!(body[ORD_UNPR], "70000");
    assert_eq!(body[EXCG_ID_DVSN_CD], "KRX");
}

#[tokio::test]
async fn real_order_cash_requires_explicit_opt_in() {
    let http_client = MockHttpClient::new(ok_output_response());
    let client = real_client(&http_client, false);
    let access_token = AccessToken::new("access-token-value");
    let request =
        OrderCashRequest::buy(StockCode::new("005930").unwrap(), "00", "1", "70000", "KRX")
            .unwrap();

    let error = client
        .domestic_stock()
        .trading()
        .order_cash(&access_token, request)
        .await
        .unwrap_err();

    assert_eq!(
        error,
        Error::config("real ordering requires explicit opt-in")
    );
    assert!(http_client.requests().is_empty());
}

#[tokio::test]
async fn order_cash_requires_account() {
    let http_client = MockHttpClient::new(ok_output_response());
    let credentials = Credentials::new("app-key", "app-secret").unwrap();
    let client = Client::new(Config::new(Environment::Virtual, credentials), &http_client);
    let access_token = AccessToken::new("access-token-value");
    let request =
        OrderCashRequest::buy(StockCode::new("005930").unwrap(), "00", "1", "70000", "KRX")
            .unwrap();

    let error = client
        .domestic_stock()
        .trading()
        .order_cash(&access_token, request)
        .await
        .unwrap_err();

    assert_eq!(error, Error::config("account is required"));
    assert!(http_client.requests().is_empty());
}

#[tokio::test]
async fn order_rvsecncl_sends_cancel_request() {
    let http_client = MockHttpClient::new(ok_output_response());
    let client = mock_client(&http_client);
    let access_token = AccessToken::new("access-token-value");
    let request = OrderRvsecnclRequest::cancel(
        "06010",
        "0000000001",
        "00",
        "1",
        "0",
        AllQuantityOrder::All,
        "KRX",
    )
    .unwrap();

    client
        .domestic_stock()
        .trading()
        .order_rvsecncl(&access_token, request)
        .await
        .unwrap();

    let request = only_request(&http_client);
    assert_eq!(request.method(), Method::Post);
    assert_eq!(
        request.url(),
        "https://openapivts.koreainvestment.com:29443/uapi/domestic-stock/v1/trading/order-rvsecncl"
    );
    assert_header(&request, "tr_id", ORDER_RVSECNCL_VIRTUAL_TR_ID);
    let body = request_body(&request);
    assert_eq!(body[KRX_FWDG_ORD_ORGNO], "06010");
    assert_eq!(body[ORGN_ODNO], "0000000001");
    assert_eq!(body[RVSE_CNCL_DVSN_CD], "02");
    assert_eq!(body[QTY_ALL_ORD_YN], "Y");
}

#[tokio::test]
async fn inquire_psbl_rvsecncl_sends_query_and_reads_continuation() {
    let http_client = MockHttpClient::new(
        Response::new(
            200,
            r#"{
                "rt_cd": "0",
                "msg_cd": "MCA00000",
                "msg1": "ok",
                "ctx_area_fk100": "fk",
                "ctx_area_nk100": "nk",
                "output": [{"odno": "0000000001", "psbl_qty": "1"}]
            }"#,
        )
        .with_headers([Header::new("tr_cont", "M")]),
    );
    let client = mock_client(&http_client);
    let access_token = AccessToken::new("access-token-value");
    let request = InquirePsblRvsecnclRequest::new("1", "0").unwrap();

    let response = client
        .domestic_stock()
        .trading()
        .inquire_psbl_rvsecncl(&access_token, request)
        .await
        .unwrap();

    assert!(response.continuation.has_next());
    assert_eq!(response.continuation.ctx_area_fk.as_deref(), Some("fk"));
    assert_eq!(response.continuation.ctx_area_nk.as_deref(), Some("nk"));

    let request = only_request(&http_client);
    assert_eq!(request.method(), Method::Get);
    assert_eq!(
        request.query_params(),
        &[
            (CANO.to_string(), "12345678".to_string()),
            (ACNT_PRDT_CD.to_string(), "01".to_string()),
            (INQR_DVSN_1.to_string(), "1".to_string()),
            (INQR_DVSN_2.to_string(), "0".to_string()),
            (CTX_AREA_FK100.to_string(), "".to_string()),
            (CTX_AREA_NK100.to_string(), "".to_string())
        ]
    );
    assert_header(&request, "tr_id", INQUIRE_PSBL_RVSECNCL_VIRTUAL_TR_ID);
}

#[tokio::test]
async fn inquire_psbl_rvsecncl_sends_next_page_marker() {
    let http_client = MockHttpClient::new(Response::new(
        200,
        r#"{
            "rt_cd": "0",
            "msg_cd": "MCA00000",
            "msg1": "ok",
            "output": []
        }"#,
    ));
    let client = mock_client(&http_client);
    let access_token = AccessToken::new("access-token-value");
    let continuation = Continuation {
        tr_cont: Some("M".to_string()),
        ctx_area_fk: Some("fk".to_string()),
        ctx_area_nk: Some("nk".to_string()),
    }
    .next_request()
    .unwrap();
    let request = InquirePsblRvsecnclRequest::new("1", "0")
        .unwrap()
        .with_continuation(continuation);

    client
        .domestic_stock()
        .trading()
        .inquire_psbl_rvsecncl(&access_token, request)
        .await
        .unwrap();

    let request = only_request(&http_client);
    assert_header(&request, "tr_cont", "N");
    assert!(
        request
            .query_params()
            .contains(&(CTX_AREA_FK100.to_string(), "fk".to_string()))
    );
    assert!(
        request
            .query_params()
            .contains(&(CTX_AREA_NK100.to_string(), "nk".to_string()))
    );
}

#[tokio::test]
async fn inquire_daily_ccld_sends_query() {
    let http_client = MockHttpClient::new(Response::new(
        200,
        r#"{
            "rt_cd": "0",
            "msg_cd": "MCA00000",
            "msg1": "ok",
            "output1": [{"odno": "0000000001"}],
            "output2": {"tot_ord_qty": "1"}
        }"#,
    ));
    let client = mock_client(&http_client);
    let access_token = AccessToken::new("access-token-value");
    let request = InquireDailyCcldRequest::new(
        InquireDailyCcldPeriod::Inner3Months,
        "20260511",
        "20260511",
        "00",
        "00",
        "00",
        "00",
    )
    .unwrap()
    .with_stock_code("005930");

    let response = client
        .domestic_stock()
        .trading()
        .inquire_daily_ccld(&access_token, request)
        .await
        .unwrap();

    assert!(response.output1.is_array());
    assert!(response.output2.is_object());

    let request = only_request(&http_client);
    assert_eq!(request.method(), Method::Get);
    assert_header(&request, "tr_id", INQUIRE_DAILY_CCLD_VIRTUAL_INNER_TR_ID);
    assert_eq!(
        request.query_params(),
        &[
            (CANO.to_string(), "12345678".to_string()),
            (ACNT_PRDT_CD.to_string(), "01".to_string()),
            (INQR_STRT_DT.to_string(), "20260511".to_string()),
            (INQR_END_DT.to_string(), "20260511".to_string()),
            (SLL_BUY_DVSN_CD.to_string(), "00".to_string()),
            (PDNO.to_string(), "005930".to_string()),
            (CCLD_DVSN.to_string(), "00".to_string()),
            (INQR_DVSN.to_string(), "00".to_string()),
            (INQR_DVSN_3.to_string(), "00".to_string()),
            (ORD_GNO_BRNO.to_string(), "".to_string()),
            (ODNO.to_string(), "".to_string()),
            (INQR_DVSN_1.to_string(), "".to_string()),
            (CTX_AREA_FK100.to_string(), "".to_string()),
            (CTX_AREA_NK100.to_string(), "".to_string()),
            (EXCG_ID_DVSN_CD.to_string(), "KRX".to_string())
        ]
    );
}

fn ok_output_response() -> Response {
    Response::new(
        200,
        r#"{
            "rt_cd": "0",
            "msg_cd": "MCA00000",
            "msg1": "ok",
            "output": {
                "ODNO": "0000000001"
            }
        }"#,
    )
}

fn mock_client(http_client: &MockHttpClient) -> Client<&MockHttpClient> {
    let credentials = Credentials::new("app-key", "app-secret").unwrap();
    let account = Account::new(
        AccountNumber::new("12345678").unwrap(),
        ProductCode::new("01").unwrap(),
    );
    let config = Config::new(Environment::Virtual, credentials).with_account(account);

    Client::new(config, http_client)
}

fn real_client(
    http_client: &MockHttpClient,
    real_ordering_enabled: bool,
) -> Client<&MockHttpClient> {
    let credentials = Credentials::new("app-key", "app-secret").unwrap();
    let account = Account::new(
        AccountNumber::new("12345678").unwrap(),
        ProductCode::new("01").unwrap(),
    );
    let config = Config::new(Environment::Real, credentials)
        .with_account(account)
        .with_real_ordering_enabled(real_ordering_enabled);

    Client::new(config, http_client)
}

fn only_request(http_client: &MockHttpClient) -> Request {
    let requests = http_client.requests();
    assert_eq!(requests.len(), 1);
    requests[0].clone()
}

fn request_body(request: &Request) -> Value {
    serde_json::from_slice(request.body()).unwrap()
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
