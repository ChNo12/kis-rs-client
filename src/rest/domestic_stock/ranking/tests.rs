use super::*;
use crate::Client;
use crate::auth::AccessToken;
use crate::config::{Config, Credentials, Environment};
use crate::error::{Error, Result};
use crate::http::{Header, HttpClient, Method, Request, Response};
use crate::rest::domestic_stock::Continuation;
use crate::rest::{PageLimit, PageStopReason};
use async_trait::async_trait;
use rust_decimal::Decimal;
use std::{collections::VecDeque, sync::Mutex};

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

#[derive(Debug)]
struct SequenceMockHttpClient {
    responses: Mutex<VecDeque<Response>>,
    requests: Mutex<Vec<Request>>,
}

impl SequenceMockHttpClient {
    fn new(responses: impl IntoIterator<Item = Response>) -> Self {
        Self {
            responses: Mutex::new(responses.into_iter().collect()),
            requests: Mutex::new(Vec::new()),
        }
    }

    fn requests(&self) -> Vec<Request> {
        self.requests.lock().unwrap().clone()
    }
}

#[async_trait]
impl HttpClient for &SequenceMockHttpClient {
    async fn send(&self, request: Request) -> Result<Response> {
        self.requests.lock().unwrap().push(request);

        Ok(self.responses.lock().unwrap().pop_front().unwrap())
    }
}

#[tokio::test]
async fn after_hour_balance_sends_request_and_parses_typed_output() {
    let http_client = MockHttpClient::new(
        Response::new(
            200,
            r#"{
                "rt_cd": "0",
                "msg_cd": "MCA00000",
                "msg1": "ok",
                "output": [{
                    "stck_shrn_iscd": "005930",
                    "data_rank": "1",
                    "hts_kor_isnm": "삼성전자",
                    "stck_prpr": "70500.5",
                    "prdy_vrss": "100",
                    "prdy_vrss_sign": "2",
                    "prdy_ctrt": "0.14",
                    "ovtm_total_askp_rsqn": "1000",
                    "ovtm_total_bidp_rsqn": "900",
                    "mkob_otcp_vol": "200",
                    "mkfa_otcp_vol": "300"
                }]
            }"#,
        )
        .with_headers([Header::new("tr_cont", "M")]),
    );
    let client = mock_client(&http_client);
    let access_token = AccessToken::new("access-token-value");
    let request = AfterHourBalanceRequest::new()
        .with_rank_sort_class_code("3")
        .with_price_range("0", "1000000")
        .with_volume_count("100");

    let response = client
        .domestic_stock()
        .ranking()
        .after_hour_balance(&access_token, request)
        .await
        .unwrap();

    assert!(response.continuation.has_next());
    let typed = response.typed().unwrap();
    assert_eq!(typed[0].stock_code, "005930");
    assert_eq!(typed[0].current_price, Decimal::new(705005, 1));
    assert_eq!(typed[0].overtime_total_ask_quantity, Decimal::new(1000, 0));

    let request = only_request(&http_client);
    assert_eq!(request.method(), Method::Get);
    assert_eq!(
        request.url(),
        "https://openapivts.koreainvestment.com:29443/uapi/domestic-stock/v1/ranking/after-hour-balance"
    );
    assert_eq!(
        request.query_params(),
        &[
            ("fid_input_price_1".to_string(), "0".to_string()),
            ("fid_cond_mrkt_div_code".to_string(), "J".to_string()),
            ("fid_cond_scr_div_code".to_string(), "20176".to_string()),
            ("fid_rank_sort_cls_code".to_string(), "3".to_string()),
            ("fid_div_cls_code".to_string(), "0".to_string()),
            ("fid_input_iscd".to_string(), "0000".to_string()),
            ("fid_trgt_exls_cls_code".to_string(), "0".to_string()),
            ("fid_trgt_cls_code".to_string(), "0".to_string()),
            ("fid_vol_cnt".to_string(), "100".to_string()),
            ("fid_input_price_2".to_string(), "1000000".to_string())
        ]
    );
    assert_header(&request, "tr_id", AFTER_HOUR_BALANCE_TR_ID);
    assert_header(&request, "tr_cont", "");
}

#[tokio::test]
async fn after_hour_balance_pages_stops_at_page_limit() {
    let http_client = SequenceMockHttpClient::new([
        ranking_page_response("005930").with_headers([Header::new("tr_cont", "M")]),
        ranking_page_response("000660").with_headers([Header::new("tr_cont", "M")]),
    ]);
    let client = sequence_mock_client(&http_client);
    let access_token = AccessToken::new("access-token-value");

    let collection = client
        .domestic_stock()
        .ranking()
        .after_hour_balance_pages(
            &access_token,
            AfterHourBalanceRequest::new(),
            PageLimit::Max(2),
        )
        .await
        .unwrap();

    assert_eq!(collection.pages.len(), 2);
    assert_eq!(collection.stop_reason, PageStopReason::PageLimitReached);
    assert_eq!(
        collection
            .next
            .as_ref()
            .and_then(|next| next.tr_cont.as_deref()),
        Some("N")
    );

    let requests = http_client.requests();
    assert_eq!(requests.len(), 2);
    assert_header(&requests[0], "tr_cont", "");
    assert_header(&requests[1], "tr_cont", "N");
}

#[tokio::test]
async fn after_hour_balance_pages_returns_partial_result_on_http_429() {
    let http_client = SequenceMockHttpClient::new([
        ranking_page_response("005930").with_headers([Header::new("tr_cont", "M")]),
        Response::new(
            429,
            r#"{"rt_cd":"1","msg_cd":"RATE","msg1":"too many requests"}"#,
        ),
    ]);
    let client = sequence_mock_client(&http_client);
    let access_token = AccessToken::new("access-token-value");

    let collection = client
        .domestic_stock()
        .ranking()
        .after_hour_balance_pages(
            &access_token,
            AfterHourBalanceRequest::new(),
            PageLimit::All,
        )
        .await
        .unwrap();

    assert_eq!(collection.pages.len(), 1);
    assert_eq!(
        collection
            .next
            .as_ref()
            .and_then(|next| next.tr_cont.as_deref()),
        Some("N")
    );
    assert!(matches!(
        collection.stop_reason,
        PageStopReason::RateLimited { ref error } if error.is_rate_limited()
    ));

    let requests = http_client.requests();
    assert_eq!(requests.len(), 2);
    assert_header(&requests[1], "tr_cont", "N");
}

#[tokio::test]
async fn after_hour_balance_pages_returns_partial_result_on_kis_rate_limit_code() {
    let http_client = SequenceMockHttpClient::new([
        ranking_page_response("005930").with_headers([Header::new("tr_cont", "M")]),
        Response::new(
            200,
            r#"{
                "rt_cd": "1",
                "msg_cd": "EGW00201",
                "msg1": "초당 거래건수를 초과하였습니다."
            }"#,
        ),
    ]);
    let client = sequence_mock_client(&http_client);
    let access_token = AccessToken::new("access-token-value");

    let collection = client
        .domestic_stock()
        .ranking()
        .after_hour_balance_pages(
            &access_token,
            AfterHourBalanceRequest::new(),
            PageLimit::All,
        )
        .await
        .unwrap();

    assert_eq!(collection.pages.len(), 1);
    assert_eq!(
        collection
            .next
            .as_ref()
            .and_then(|next| next.tr_cont.as_deref()),
        Some("N")
    );
    assert!(matches!(
        collection.stop_reason,
        PageStopReason::RateLimited { ref error } if error.is_rate_limited()
    ));
}

#[tokio::test]
async fn after_hour_balance_sends_next_page_header() {
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
        ctx_area_fk: None,
        ctx_area_nk: None,
    }
    .next_request()
    .unwrap();
    let request = AfterHourBalanceRequest::new().with_continuation(continuation);

    client
        .domestic_stock()
        .ranking()
        .after_hour_balance(&access_token, request)
        .await
        .unwrap();

    let request = only_request(&http_client);
    assert_header(&request, "tr_cont", "N");
}

#[tokio::test]
async fn bulk_trans_num_sends_request_and_parses_typed_output() {
    let http_client = MockHttpClient::new(
        Response::new(
            200,
            r#"{
                "rt_cd": "0",
                "msg_cd": "MCA00000",
                "msg1": "ok",
                "output": [{
                    "mksc_shrn_iscd": "005930",
                    "data_rank": "1",
                    "hts_kor_isnm": "삼성전자",
                    "stck_prpr": "70500.5",
                    "prdy_vrss_sign": "2",
                    "prdy_vrss": "100",
                    "prdy_ctrt": "0.14",
                    "acml_vol": "123456",
                    "shnu_cntg_csnu": "42",
                    "seln_cntg_csnu": "30",
                    "ntby_cnqn": "1200"
                }]
            }"#,
        )
        .with_headers([Header::new("tr_cont", "M")]),
    );
    let client = mock_client(&http_client);
    let access_token = AccessToken::new("access-token-value");
    let request = BulkTransNumRequest::new()
        .with_input_price1("50000")
        .with_applied_price_range("200000", "100000")
        .with_volume_count("1000");

    let response = client
        .domestic_stock()
        .ranking()
        .bulk_trans_num(&access_token, request)
        .await
        .unwrap();

    assert!(response.continuation.has_next());
    let typed = response.typed().unwrap();
    assert_eq!(typed[0].stock_code, "005930");
    assert_eq!(typed[0].current_price, Decimal::new(705005, 1));
    assert_eq!(typed[0].buy_conclusion_count, Decimal::new(42, 0));

    let request = only_request(&http_client);
    assert_eq!(request.method(), Method::Get);
    assert_eq!(
        request.url(),
        "https://openapivts.koreainvestment.com:29443/uapi/domestic-stock/v1/ranking/bulk-trans-num"
    );
    assert_eq!(
        request.query_params(),
        &[
            ("fid_aply_rang_prc_2".to_string(), "100000".to_string()),
            ("fid_cond_mrkt_div_code".to_string(), "J".to_string()),
            ("fid_cond_scr_div_code".to_string(), "11909".to_string()),
            ("fid_input_iscd".to_string(), "0000".to_string()),
            ("fid_rank_sort_cls_code".to_string(), "0".to_string()),
            ("fid_div_cls_code".to_string(), "0".to_string()),
            ("fid_input_price_1".to_string(), "50000".to_string()),
            ("fid_aply_rang_prc_1".to_string(), "200000".to_string()),
            ("fid_input_iscd_2".to_string(), "".to_string()),
            ("fid_trgt_exls_cls_code".to_string(), "0".to_string()),
            ("fid_trgt_cls_code".to_string(), "0".to_string()),
            ("fid_vol_cnt".to_string(), "1000".to_string())
        ]
    );
    assert_header(&request, "tr_id", BULK_TRANS_NUM_TR_ID);
    assert_header(&request, "tr_cont", "");
}

#[tokio::test]
async fn bulk_trans_num_sends_next_page_header() {
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
        ctx_area_fk: None,
        ctx_area_nk: None,
    }
    .next_request()
    .unwrap();
    let request = BulkTransNumRequest::new().with_continuation(continuation);

    client
        .domestic_stock()
        .ranking()
        .bulk_trans_num(&access_token, request)
        .await
        .unwrap();

    let request = only_request(&http_client);
    assert_header(&request, "tr_cont", "N");
}

#[tokio::test]
async fn fluctuation_sends_request_and_parses_typed_output() {
    let http_client = MockHttpClient::new(
        Response::new(
            200,
            r#"{
                "rt_cd": "0",
                "msg_cd": "MCA00000",
                "msg1": "ok",
                "output": [{
                    "stck_shrn_iscd": "005930",
                    "data_rank": "1",
                    "hts_kor_isnm": "삼성전자",
                    "stck_prpr": "70500.5",
                    "prdy_vrss": "100",
                    "prdy_vrss_sign": "2",
                    "prdy_ctrt": "0.14",
                    "acml_vol": "123456",
                    "stck_hgpr": "70700",
                    "hgpr_hour": "093000",
                    "acml_hgpr_date": "20260511",
                    "stck_lwpr": "69900",
                    "lwpr_hour": "091500",
                    "acml_lwpr_date": "20260511",
                    "lwpr_vrss_prpr_rate": "0.86",
                    "dsgt_date_clpr_vrss_prpr_rate": "1.20",
                    "cnnt_ascn_dynu": "2",
                    "hgpr_vrss_prpr_rate": "-0.28",
                    "cnnt_down_dynu": "0",
                    "oprc_vrss_prpr_sign": "2",
                    "oprc_vrss_prpr": "500",
                    "oprc_vrss_prpr_rate": "0.71",
                    "prd_rsfl": "1500",
                    "prd_rsfl_rate": "2.17"
                }]
            }"#,
        )
        .with_headers([Header::new("tr_cont", "M")]),
    );
    let client = mock_client(&http_client);
    let access_token = AccessToken::new("access-token-value");
    let request = FluctuationRequest::new()
        .with_input_count("10")
        .with_price_range("0", "1000000")
        .with_volume_count("100000")
        .with_fluctuation_rate_range("0", "10");

    let response = client
        .domestic_stock()
        .ranking()
        .fluctuation(&access_token, request)
        .await
        .unwrap();

    assert!(response.continuation.has_next());
    let typed = response.typed().unwrap();
    assert_eq!(typed[0].stock_code, "005930");
    assert_eq!(typed[0].current_price, Decimal::new(705005, 1));
    assert_eq!(typed[0].previous_day_rate, Decimal::new(14, 2));

    let request = only_request(&http_client);
    assert_eq!(request.method(), Method::Get);
    assert_eq!(
        request.url(),
        "https://openapivts.koreainvestment.com:29443/uapi/domestic-stock/v1/ranking/fluctuation"
    );
    assert_eq!(
        request.query_params(),
        &[
            ("fid_rsfl_rate2".to_string(), "10".to_string()),
            ("fid_cond_mrkt_div_code".to_string(), "J".to_string()),
            ("fid_cond_scr_div_code".to_string(), "20170".to_string()),
            ("fid_input_iscd".to_string(), "0000".to_string()),
            ("fid_rank_sort_cls_code".to_string(), "0".to_string()),
            ("fid_input_cnt_1".to_string(), "10".to_string()),
            ("fid_prc_cls_code".to_string(), "0".to_string()),
            ("fid_input_price_1".to_string(), "0".to_string()),
            ("fid_input_price_2".to_string(), "1000000".to_string()),
            ("fid_vol_cnt".to_string(), "100000".to_string()),
            ("fid_trgt_cls_code".to_string(), "0".to_string()),
            ("fid_trgt_exls_cls_code".to_string(), "0".to_string()),
            ("fid_div_cls_code".to_string(), "0".to_string()),
            ("fid_rsfl_rate1".to_string(), "0".to_string())
        ]
    );
    assert_header(&request, "tr_id", FLUCTUATION_TR_ID);
    assert_header(&request, "tr_cont", "");
}

#[tokio::test]
async fn fluctuation_sends_next_page_header() {
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
        ctx_area_fk: None,
        ctx_area_nk: None,
    }
    .next_request()
    .unwrap();
    let request = FluctuationRequest::new().with_continuation(continuation);

    client
        .domestic_stock()
        .ranking()
        .fluctuation(&access_token, request)
        .await
        .unwrap();

    let request = only_request(&http_client);
    assert_header(&request, "tr_cont", "N");
}

#[tokio::test]
async fn fluctuation_rejects_invalid_typed_number() {
    let http_client = MockHttpClient::new(Response::new(
        200,
        r#"{
            "rt_cd": "0",
            "msg_cd": "MCA00000",
            "msg1": "ok",
            "output": [{
                "stck_shrn_iscd": "005930",
                "data_rank": "not-a-number",
                "hts_kor_isnm": "삼성전자",
                "stck_prpr": "70500",
                "prdy_vrss": "100",
                "prdy_vrss_sign": "2",
                "prdy_ctrt": "0.14",
                "acml_vol": "123456",
                "stck_hgpr": "70700",
                "hgpr_hour": "093000",
                "acml_hgpr_date": "20260511",
                "stck_lwpr": "69900",
                "lwpr_hour": "091500",
                "acml_lwpr_date": "20260511",
                "lwpr_vrss_prpr_rate": "0.86",
                "dsgt_date_clpr_vrss_prpr_rate": "1.20",
                "cnnt_ascn_dynu": "2",
                "hgpr_vrss_prpr_rate": "-0.28",
                "cnnt_down_dynu": "0",
                "oprc_vrss_prpr_sign": "2",
                "oprc_vrss_prpr": "500",
                "oprc_vrss_prpr_rate": "0.71",
                "prd_rsfl": "1500",
                "prd_rsfl_rate": "2.17"
            }]
        }"#,
    ));
    let client = mock_client(&http_client);
    let access_token = AccessToken::new("access-token-value");

    let response = client
        .domestic_stock()
        .ranking()
        .fluctuation(&access_token, FluctuationRequest::new())
        .await
        .unwrap();

    assert!(matches!(response.typed(), Err(Error::Parse { .. })));
}

fn mock_client(http_client: &MockHttpClient) -> Client<&MockHttpClient> {
    let credentials = Credentials::new("app-key", "app-secret").unwrap();
    let config = Config::new(Environment::Virtual, credentials);

    Client::new(config, http_client)
}

fn sequence_mock_client(http_client: &SequenceMockHttpClient) -> Client<&SequenceMockHttpClient> {
    let credentials = Credentials::new("app-key", "app-secret").unwrap();
    let config = Config::new(Environment::Virtual, credentials);

    Client::new(config, http_client)
}

fn ranking_page_response(stock_code: &str) -> Response {
    Response::new(
        200,
        format!(
            r#"{{
                "rt_cd": "0",
                "msg_cd": "MCA00000",
                "msg1": "ok",
                "output": [{{
                    "stck_shrn_iscd": "{stock_code}",
                    "data_rank": "1",
                    "hts_kor_isnm": "삼성전자",
                    "stck_prpr": "70500.5",
                    "prdy_vrss": "100",
                    "prdy_vrss_sign": "2",
                    "prdy_ctrt": "0.14",
                    "ovtm_total_askp_rsqn": "1000",
                    "ovtm_total_bidp_rsqn": "900",
                    "mkob_otcp_vol": "200",
                    "mkfa_otcp_vol": "300"
                }}]
            }}"#
        ),
    )
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
