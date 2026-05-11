# REST API Catalog

Updated: 2026-05-11

이 문서는 KIS REST API 구현 상태를 추적한다. WebSocket 항목은
[`WEBSOCKET_CATALOG.md`](WEBSOCKET_CATALOG.md)에서 별도로 추적한다.

기준:

- KIS 공식 샘플 목록: <https://github.com/koreainvestment/open-trading-api>
- 국내주식 샘플 목록: <https://github.com/koreainvestment/open-trading-api/tree/main/examples_llm/domestic_stock>
- 해외주식 샘플 목록: <https://github.com/koreainvestment/open-trading-api/tree/main/examples_llm/overseas_stock>
- 이 문서의 분류는 Rust crate 구현 편의를 위한 초안이다. 공식 카테고리와 다를 수 있다.
- 미구현 API의 path, TR ID, 요청/응답 필드는 구현 시점에 공식 샘플과 공식 문서를 다시 확인한다.
- typed view의 숫자형 응답 필드는 기본적으로 `rust_decimal::Decimal`을 사용한다.
- 자동 테스트에서 실제 KIS API는 호출하지 않는다. mock fixture로 path, TR ID, query/body, output parsing을 검증한다.

## Summary

| Status | Count |
|---|---:|
| Implemented domestic REST | 15 |
| Implemented overseas REST | 3 |
| Remaining/deferred domestic catalog entries | 85 |
| Total official domestic_stock sample folders | 100 |

## Implemented Domestic REST

| Sample Folder | Public API | Module | Path | TR ID |
|---|---|---|---|---|
| `after_hour_balance` | `after_hour_balance` | `domestic_stock().ranking()` | `/uapi/domestic-stock/v1/ranking/after-hour-balance` | `FHPST01760000` |
| `bulk_trans_num` | `bulk_trans_num` | `domestic_stock().ranking()` | `/uapi/domestic-stock/v1/ranking/bulk-trans-num` | `FHKST190900C0` |
| `capture_uplowprice` | `capture_up_low_price` | `domestic_stock().market_analysis()` | `/uapi/domestic-stock/v1/quotations/capture-uplowprice` | `FHKST130000C0` |
| `fluctuation` | `fluctuation` | `domestic_stock().ranking()` | `/uapi/domestic-stock/v1/ranking/fluctuation` | `FHPST01700000` |
| `inquire_asking_price_exp_ccn` | `inquire_asking_price_exp_ccn` | `domestic_stock().quotations()` | `/uapi/domestic-stock/v1/quotations/inquire-asking-price-exp-ccn` | `FHKST01010200` |
| `inquire_ccnl` | `inquire_ccnl` | `domestic_stock().quotations()` | `/uapi/domestic-stock/v1/quotations/inquire-ccnl` | `FHKST01010300` |
| `inquire_daily_itemchartprice` | `inquire_daily_item_chart_price` | `domestic_stock().quotations()` | `/uapi/domestic-stock/v1/quotations/inquire-daily-itemchartprice` | `FHKST03010100` |
| `inquire_daily_price` | `inquire_daily_price` | `domestic_stock().quotations()` | `/uapi/domestic-stock/v1/quotations/inquire-daily-price` | `FHKST01010400` |
| `inquire_price` | `inquire_price` | `domestic_stock().quotations()` | `/uapi/domestic-stock/v1/quotations/inquire-price` | `FHKST01010100` |
| `inquire_time_dailychartprice` | `inquire_time_daily_chart_price` | `domestic_stock().quotations()` | `/uapi/domestic-stock/v1/quotations/inquire-time-dailychartprice` | `FHKST03010230` |
| `inquire_time_itemchartprice` | `inquire_time_item_chart_price` | `domestic_stock().quotations()` | `/uapi/domestic-stock/v1/quotations/inquire-time-itemchartprice` | `FHKST03010200` |
| `inquire_daily_ccld` | `inquire_daily_ccld` | `domestic_stock().trading()` | `/uapi/domestic-stock/v1/trading/inquire-daily-ccld` | `TTTC0081R`/`VTTC0081R`, `CTSC9215R`/`VTSC9215R` |
| `inquire_psbl_rvsecncl` | `inquire_psbl_rvsecncl` | `domestic_stock().trading()` | `/uapi/domestic-stock/v1/trading/inquire-psbl-rvsecncl` | `TTTC0084R`/`VTTC0084R` |
| `order_cash` | `order_cash` | `domestic_stock().trading()` | `/uapi/domestic-stock/v1/trading/order-cash` | `TTTC0011U`/`TTTC0012U`, `VTTC0011U`/`VTTC0012U` |
| `order_rvsecncl` | `order_rvsecncl` | `domestic_stock().trading()` | `/uapi/domestic-stock/v1/trading/order-rvsecncl` | `TTTC0013U`/`VTTC0013U` |

## Implemented Overseas REST

v1은 미국 `NASD`, `NYSE`, `AMEX`만 지원한다.

| Public API | Module | Path | TR ID |
|---|---|---|---|
| `order` | `overseas_stock().trading()` | `/uapi/overseas-stock/v1/trading/order` | `TTTT1002U`/`VTTT1002U`, `TTTT1006U`/`VTTT1006U` |
| `order_rvsecncl` | `overseas_stock().trading()` | `/uapi/overseas-stock/v1/trading/order-rvsecncl` | `TTTT1004U`/`VTTT1004U` |
| `inquire_ccnl` | `overseas_stock().trading()` | `/uapi/overseas-stock/v1/trading/inquire-ccnl` | `TTTS3035R`/`VTTS3035R` |

## Remaining Domestic By Suggested Module

### `quotations`

| Status | Sample Folder | Notes |
|---|---|---|
| Planned | `asking_price_krx` | KRX 호가 |
| Planned | `asking_price_nxt` | NXT 호가 |
| Planned | `asking_price_total` | 통합 호가 |
| Planned | `chk_holiday` | 휴장일 조회 |
| Planned | `exp_ccnl_krx` | KRX 예상체결 |
| Planned | `exp_ccnl_nxt` | NXT 예상체결 |
| Planned | `exp_ccnl_total` | 통합 예상체결 |
| Planned | `exp_closing_price` | 예상종가 |
| Planned | `exp_index_trend` | 예상 지수 추이 |
| Planned | `exp_price_trend` | 예상 가격 추이 |
| Planned | `exp_total_index` | 예상 종합지수 |
| Planned | `exp_trans_updown` | 예상 체결 상하락 |
| Planned | `index_ccnl` | 지수 체결 |
| Planned | `index_exp_ccnl` | 지수 예상체결 |
| Planned | `index_program_trade` | 지수 프로그램매매 |
| Planned | `inquire_daily_indexchartprice` | 지수 일봉 차트 |
| Planned | `inquire_daily_overtimeprice` | 일별 시간외 시세 |
| Planned | `inquire_daily_trade_volume` | 일별 거래량 |
| Planned | `inquire_elw_price` | ELW 현재가 |
| Planned | `inquire_index_category_price` | 지수 업종 가격 |
| Planned | `inquire_index_daily_price` | 지수 일별 가격 |
| Planned | `inquire_index_price` | 지수 현재가 |
| Planned | `inquire_index_tickprice` | 지수 틱 |
| Planned | `inquire_index_timeprice` | 지수 시간대별 |
| Planned | `inquire_overtime_asking_price` | 시간외 호가 |
| Planned | `inquire_overtime_price` | 시간외 현재가 |
| Planned | `inquire_price_2` | 현재가 확장/대체 API 확인 필요 |
| Planned | `inquire_time_indexchartprice` | 지수 분봉 차트 |
| Planned | `inquire_time_itemconclusion` | 종목 시간대별 체결 |
| Planned | `inquire_time_overtimeconclusion` | 시간외 체결 |
| Planned | `inquire_vi_status` | VI 현황 |

### `ranking`

이미 `fluctuation`, `after_hour_balance`, `bulk_trans_num`은 구현됐다.

| Status | Sample Folder | Notes |
|---|---|---|
| Planned | `disparity` | 이격도 순위/분석 |
| Planned | `dividend_rate` | 배당률 |
| Planned | `hts_top_view` | HTS 조회 상위 |

### `market_analysis`

이미 `capture_uplowprice`는 구현됐다.

| Status | Sample Folder | Notes |
|---|---|---|
| Planned | `comp_interest` | 관심종목 비교 |
| Planned | `comp_program_trade_daily` | 프로그램매매 일별 비교 |
| Planned | `comp_program_trade_today` | 프로그램매매 당일 비교 |
| Planned | `credit_balance` | 신용잔고 |
| Planned | `credit_by_company` | 증권사별 신용 |
| Planned | `daily_credit_balance` | 일별 신용잔고 |
| Planned | `daily_loan_trans` | 일별 대주거래 |
| Planned | `daily_short_sale` | 일별 공매도 |
| Planned | `estimate_perform` | 실적 추정 |
| Planned | `foreign_institution_total` | 외국인/기관 종합 |
| Planned | `frgnmem_pchs_trend` | 외국계 매수 추이 |
| Planned | `frgnmem_trade_estimate` | 외국계 매매 추정 |
| Planned | `frgnmem_trade_trend` | 외국계 매매 추이 |
| Planned | `inquire_investor` | 투자자 조회 |
| Planned | `inquire_investor_daily_by_market` | 시장별 투자자 일별 |
| Planned | `inquire_investor_time_by_market` | 시장별 투자자 시간대별 |
| Planned | `inquire_member` | 회원사 |
| Planned | `inquire_member_daily` | 회원사 일별 |
| Planned | `invest_opbysec` | 업종별 투자자 |
| Planned | `invest_opinion` | 투자의견 |
| Planned | `investor_program_trade_today` | 투자자 프로그램매매 당일 |
| Planned | `investor_trade_by_stock_daily` | 종목별 투자자 일별 |
| Planned | `investor_trend_estimate` | 투자자 추정 추이 |

### `stock_info`

| Status | Sample Folder | Notes |
|---|---|---|
| Planned | `finance_balance_sheet` | 재무 대차대조표 |
| Planned | `finance_financial_ratio` | 재무비율 |
| Planned | `finance_growth_ratio` | 성장성비율 |
| Planned | `finance_income_statement` | 손익계산서 |
| Planned | `finance_other_major_ratios` | 기타 주요비율 |
| Planned | `finance_profit_ratio` | 수익성비율 |
| Planned | `finance_ratio` | 재무비율 종합 |
| Planned | `finance_stability_ratio` | 안정성비율 |
| Planned | `intstock_grouplist` | 관심그룹 목록 |
| Planned | `intstock_multprice` | 관심종목 복수시세 |
| Planned | `intstock_stocklist_by_group` | 관심그룹별 종목 |
| Planned | `ksdinfo_bonus_issue` | KSD 무상증자 |
| Planned | `ksdinfo_cap_dcrs` | KSD 감자 |
| Planned | `ksdinfo_dividend` | KSD 배당 |
| Planned | `ksdinfo_forfeit` | KSD 실권주 |
| Planned | `ksdinfo_list_info` | KSD 상장정보 |
| Planned | `ksdinfo_mand_deposit` | KSD 의무예탁 |

### `account`

계좌번호/상품코드가 필요할 수 있으므로 masking과 config 검증을 먼저 적용한다.

| Status | Sample Folder | Notes |
|---|---|---|
| Planned-sensitive | `inquire_account_balance` | 계좌 잔고성 조회 |
| Planned-sensitive | `inquire_balance` | 잔고 조회 |
| Planned-sensitive | `inquire_balance_rlz_pl` | 실현손익/잔고성 조회 |
| Planned-sensitive | `inquire_credit_psamount` | 신용 가능금액 |
| Planned-sensitive | `inquire_period_profit` | 기간 손익 |
| Planned-sensitive | `inquire_period_trade_profit` | 기간 매매손익 |
| Planned-sensitive | `intgr_margin` | 통합증거금 |

### `order_support`

주문 실행은 아니지만 주문 가능 수량/정정취소 가능 여부와 가까운 API 후보다.

| Status | Sample Folder | Notes |
|---|---|---|
| Planned-sensitive | `inquire_psbl_order` | 주문가능 조회 |
| Planned-sensitive | `inquire_psbl_sell` | 매도가능 조회 |

## Deferred

| Area | Status | Notes |
|---|---|---|
| Broad domestic REST expansion | Deferred | 순위/시세분석/종목정보/계좌성 전체 확장은 focused v1 이후 검토한다. |
| Overseas non-US expansion | Deferred | v1은 미국 `NASD`, `NYSE`, `AMEX`만 지원한다. |

## Implementation Checklist

새 REST API를 추가할 때는 아래 순서로 진행한다.

1. 공식 샘플에서 path, TR ID, query/body field, response field를 확인한다.
2. 요청 struct는 공식 key를 보존하고, 기본값은 샘플의 안전한 조회값을 따른다.
3. 응답 struct는 raw `serde_json::Value`를 보존한다.
4. 안정적으로 확인한 필드는 `typed()` view를 추가한다.
5. 숫자형 typed field는 `Decimal`로 파싱한다.
6. 필수 필드 누락/숫자 파싱 실패 fixture를 추가한다.
7. 연속조회 API면 `Continuation` fixture를 추가한다.
8. 실제 KIS API 호출 없이 mock transport로 path, TR ID, query/body, parsing을 검증한다.
