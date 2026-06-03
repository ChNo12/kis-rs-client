use super::*;
use crate::Client;
use crate::auth::AccessToken;
use crate::config::{Account, AccountNumber, Config, Credentials, Environment, ProductCode};
use crate::error::{Error, Result};
use crate::http::{Header, HttpClient, Method, Request, Response};
use crate::rest::overseas_stock::common::{ACNT_PRDT_CD, CANO};
use crate::rest::overseas_stock::{Continuation, OrderSide, OverseasExchange, OverseasStockCode};
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
async fn order_sends_us_virtual_buy_request() {
    let http_client = MockHttpClient::new(ok_output_response());
    let client = mock_client(&http_client);
    let access_token = AccessToken::new("access-token-value");
    let request = OrderRequest::buy(
        OverseasExchange::Nasdaq,
        OverseasStockCode::new("AAPL").unwrap(),
        "1",
        "145.00",
        "00",
    )
    .unwrap();

    let response = client
        .overseas_stock()
        .trading()
        .order(&access_token, request)
        .await
        .unwrap();

    assert_eq!(
        response.output.get("ODNO").and_then(Value::as_str),
        Some("3000000001")
    );

    let request = only_request(&http_client);
    assert_eq!(request.method(), Method::Post);
    assert_eq!(
        request.url(),
        "https://openapivts.koreainvestment.com:29443/uapi/overseas-stock/v1/trading/order"
    );
    assert_header(&request, "tr_id", ORDER_VIRTUAL_BUY_TR_ID);
    let body = request_body(&request);
    assert_eq!(body[CANO], "12345678");
    assert_eq!(body[ACNT_PRDT_CD], "01");
    assert_eq!(body["OVRS_EXCG_CD"], "NASD");
    assert_eq!(body["PDNO"], "AAPL");
    assert_eq!(body["ORD_QTY"], "1");
    assert_eq!(body["OVRS_ORD_UNPR"], "145.00");
    assert_eq!(body["ORD_SVR_DVSN_CD"], "0");
    assert_eq!(body["ORD_DVSN"], "00");
}

#[tokio::test]
async fn order_tr_id_maps_us_side_and_environment() {
    assert_eq!(
        order_tr_id(Environment::Real, OrderSide::Buy),
        ORDER_REAL_BUY_TR_ID
    );
    assert_eq!(
        order_tr_id(Environment::Virtual, OrderSide::Sell),
        ORDER_VIRTUAL_SELL_TR_ID
    );
}

#[tokio::test]
async fn real_order_requires_explicit_opt_in() {
    let http_client = MockHttpClient::new(ok_output_response());
    let client = real_client(&http_client, false);
    let access_token = AccessToken::new("access-token-value");
    let request = OrderRequest::buy(
        OverseasExchange::Nasdaq,
        OverseasStockCode::new("AAPL").unwrap(),
        "1",
        "145.00",
        "00",
    )
    .unwrap();

    let error = client
        .overseas_stock()
        .trading()
        .order(&access_token, request)
        .await
        .unwrap_err();

    assert_eq!(
        error,
        Error::config("real ordering requires explicit opt-in")
    );
    assert!(http_client.requests().is_empty());
}

#[tokio::test]
async fn order_rvsecncl_sends_cancel_request() {
    let http_client = MockHttpClient::new(ok_output_response());
    let client = mock_client(&http_client);
    let access_token = AccessToken::new("access-token-value");
    let request = OrderRvsecnclRequest::cancel(
        OverseasExchange::Nyse,
        OverseasStockCode::new("BA").unwrap(),
        "3000000001",
        "1",
        "0",
    )
    .unwrap();

    client
        .overseas_stock()
        .trading()
        .order_rvsecncl(&access_token, request)
        .await
        .unwrap();

    let request = only_request(&http_client);
    assert_eq!(request.method(), Method::Post);
    assert_eq!(
        request.url(),
        "https://openapivts.koreainvestment.com:29443/uapi/overseas-stock/v1/trading/order-rvsecncl"
    );
    assert_header(&request, "tr_id", ORDER_RVSECNCL_VIRTUAL_TR_ID);
    let body = request_body(&request);
    assert_eq!(body["OVRS_EXCG_CD"], "NYSE");
    assert_eq!(body["PDNO"], "BA");
    assert_eq!(body["ORGN_ODNO"], "3000000001");
    assert_eq!(body["RVSE_CNCL_DVSN_CD"], "02");
    assert_eq!(body["ORD_QTY"], "1");
    assert_eq!(body["OVRS_ORD_UNPR"], "0");
}

#[tokio::test]
async fn inquire_ccnl_sends_query_and_reads_continuation() {
    let http_client = MockHttpClient::new(
        Response::new(
            200,
            r#"{
                "rt_cd": "0",
                "msg_cd": "MCA00000",
                "msg1": "ok",
                "ctx_area_fk200": "fk",
                "ctx_area_nk200": "nk",
                "output": [{"odno": "3000000001"}]
            }"#,
        )
        .with_headers([Header::new("tr_cont", "M")]),
    );
    let client = mock_client(&http_client);
    let access_token = AccessToken::new("access-token-value");
    let request = InquireCcnlRequest::new("20260601", "20260630", "00", "00", "DS")
        .unwrap()
        .with_exchange(OverseasExchange::Nasdaq);

    let response = client
        .overseas_stock()
        .trading()
        .inquire_ccnl(&access_token, request)
        .await
        .unwrap();

    assert!(response.continuation.has_next());
    assert_eq!(response.continuation.ctx_area_fk.as_deref(), Some("fk"));
    assert_eq!(response.continuation.ctx_area_nk.as_deref(), Some("nk"));

    let request = only_request(&http_client);
    assert_eq!(request.method(), Method::Get);
    assert_eq!(
        request.url(),
        "https://openapivts.koreainvestment.com:29443/uapi/overseas-stock/v1/trading/inquire-ccnl"
    );
    assert_header(&request, "tr_id", INQUIRE_CCNL_VIRTUAL_TR_ID);
    assert_eq!(
        request.query_params(),
        &[
            (CANO.to_string(), "12345678".to_string()),
            (ACNT_PRDT_CD.to_string(), "01".to_string()),
            ("PDNO".to_string(), "".to_string()),
            ("ORD_STRT_DT".to_string(), "20260601".to_string()),
            ("ORD_END_DT".to_string(), "20260630".to_string()),
            ("SLL_BUY_DVSN".to_string(), "00".to_string()),
            ("CCLD_NCCS_DVSN".to_string(), "00".to_string()),
            ("OVRS_EXCG_CD".to_string(), "NASD".to_string()),
            ("SORT_SQN".to_string(), "DS".to_string()),
            ("ORD_DT".to_string(), "".to_string()),
            ("ORD_GNO_BRNO".to_string(), "".to_string()),
            ("ODNO".to_string(), "".to_string()),
            ("CTX_AREA_NK200".to_string(), "".to_string()),
            ("CTX_AREA_FK200".to_string(), "".to_string())
        ]
    );
}

#[tokio::test]
async fn inquire_ccnl_sends_next_page_marker() {
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
    let request = InquireCcnlRequest::new("20260601", "20260630", "00", "00", "DS")
        .unwrap()
        .with_continuation(continuation);

    client
        .overseas_stock()
        .trading()
        .inquire_ccnl(&access_token, request)
        .await
        .unwrap();

    let request = only_request(&http_client);
    assert_header(&request, "tr_cont", "N");
    assert!(
        request
            .query_params()
            .contains(&("CTX_AREA_FK200".to_string(), "fk".to_string()))
    );
    assert!(
        request
            .query_params()
            .contains(&("CTX_AREA_NK200".to_string(), "nk".to_string()))
    );
}

#[tokio::test]
async fn inquire_balance_sends_query_and_reads_continuation() {
    let http_client = MockHttpClient::new(
        Response::new(
            200,
            r#"{
                "rt_cd": "0",
                "msg_cd": "MCA00000",
                "msg1": "ok",
                "ctx_area_fk200": "fk",
                "ctx_area_nk200": "nk",
                "output1": {"frcr_pchs_amt1": "100.00"},
                "output2": [{"ovrs_pdno": "AAPL", "ovrs_cblc_qty": "1"}]
            }"#,
        )
        .with_headers([Header::new("tr_cont", "M")]),
    );
    let client = mock_client(&http_client);
    let access_token = AccessToken::new("access-token-value");
    let request = InquireBalanceRequest::new("NASD", "USD").unwrap();

    let response = client
        .overseas_stock()
        .trading()
        .inquire_balance(&access_token, request)
        .await
        .unwrap();

    assert!(response.continuation.has_next());
    assert_eq!(response.continuation.ctx_area_fk.as_deref(), Some("fk"));
    assert_eq!(response.continuation.ctx_area_nk.as_deref(), Some("nk"));
    assert_eq!(
        response
            .output2
            .get(0)
            .and_then(|output| output.get("ovrs_pdno"))
            .and_then(Value::as_str),
        Some("AAPL")
    );

    let request = only_request(&http_client);
    assert_eq!(request.method(), Method::Get);
    assert_eq!(
        request.url(),
        "https://openapivts.koreainvestment.com:29443/uapi/overseas-stock/v1/trading/inquire-balance"
    );
    assert_header(&request, "tr_id", INQUIRE_BALANCE_VIRTUAL_TR_ID);
    assert_eq!(
        request.query_params(),
        &[
            (CANO.to_string(), "12345678".to_string()),
            (ACNT_PRDT_CD.to_string(), "01".to_string()),
            ("OVRS_EXCG_CD".to_string(), "NASD".to_string()),
            ("TR_CRCY_CD".to_string(), "USD".to_string()),
            ("CTX_AREA_FK200".to_string(), "".to_string()),
            ("CTX_AREA_NK200".to_string(), "".to_string())
        ]
    );
}

#[tokio::test]
async fn inquire_balance_sends_next_page_marker() {
    let http_client = MockHttpClient::new(Response::new(
        200,
        r#"{
            "rt_cd": "0",
            "msg_cd": "MCA00000",
            "msg1": "ok",
            "output1": {},
            "output2": []
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
    let request = InquireBalanceRequest::new("NASD", "USD")
        .unwrap()
        .with_continuation(continuation);

    client
        .overseas_stock()
        .trading()
        .inquire_balance(&access_token, request)
        .await
        .unwrap();

    let request = only_request(&http_client);
    assert_header(&request, "tr_cont", "N");
    assert!(
        request
            .query_params()
            .contains(&("CTX_AREA_FK200".to_string(), "fk".to_string()))
    );
    assert!(
        request
            .query_params()
            .contains(&("CTX_AREA_NK200".to_string(), "nk".to_string()))
    );
}

#[tokio::test]
async fn inquire_present_balance_sends_query() {
    let http_client = MockHttpClient::new(Response::new(
        200,
        r#"{
            "rt_cd": "0",
            "msg_cd": "MCA00000",
            "msg1": "ok",
            "output1": [{"ovrs_pdno": "AAPL"}],
            "output2": [{"crcy_cd": "USD"}],
            "output3": {"tot_asst_amt": "1000.00"}
        }"#,
    ));
    let client = mock_client(&http_client);
    let access_token = AccessToken::new("access-token-value");
    let request = InquirePresentBalanceRequest::new("01", "000", "00", "00").unwrap();

    let response = client
        .overseas_stock()
        .trading()
        .inquire_present_balance(&access_token, request)
        .await
        .unwrap();

    assert_eq!(
        response.output3.get("tot_asst_amt").and_then(Value::as_str),
        Some("1000.00")
    );

    let request = only_request(&http_client);
    assert_eq!(request.method(), Method::Get);
    assert_eq!(
        request.url(),
        "https://openapivts.koreainvestment.com:29443/uapi/overseas-stock/v1/trading/inquire-present-balance"
    );
    assert_header(&request, "tr_id", INQUIRE_PRESENT_BALANCE_VIRTUAL_TR_ID);
    assert_eq!(
        request.query_params(),
        &[
            (CANO.to_string(), "12345678".to_string()),
            (ACNT_PRDT_CD.to_string(), "01".to_string()),
            ("WCRC_FRCR_DVSN_CD".to_string(), "01".to_string()),
            ("NATN_CD".to_string(), "000".to_string()),
            ("TR_MKET_CD".to_string(), "00".to_string()),
            ("INQR_DVSN_CD".to_string(), "00".to_string())
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
                "ODNO": "3000000001"
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
