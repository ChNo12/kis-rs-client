# WebSocket API Catalog

Updated: 2026-05-11

이 문서는 KIS WebSocket 구현 상태를 추적한다. REST 항목은
[`REST_CATALOG.md`](REST_CATALOG.md)에서 별도로 추적한다.

기준:

- KIS 공식 샘플 저장소: <https://github.com/koreainvestment/open-trading-api>
- 국내주식 샘플 목록: <https://github.com/koreainvestment/open-trading-api/tree/main/examples_llm/domestic_stock>
- 해외주식 샘플 목록: <https://github.com/koreainvestment/open-trading-api/tree/main/examples_llm/overseas_stock>
- 미구현 WebSocket feed의 TR ID, payload 컬럼, 암호화 여부는 구현 시점에 공식 샘플과 공식 문서를 다시 확인한다.
- 자동 테스트에서 실제 KIS WebSocket 서버는 호출하지 않는다. live smoke는 `#[ignore]`와 환경 변수로 보호한다.

## Summary

| Status | Count |
|---|---:|
| Implemented domestic WebSocket feeds | 2 |
| Implemented overseas WebSocket feeds | 1 |

## Implemented

| Area | Feed | Public API | TR ID | Notes |
|---|---|---|---|---|
| Domestic | 실시간 가격 | `DomesticRealtimePrice` | `H0STCNT0`, `H0NXCNT0`, `H0UNCNT0` | KRX/NXT/통합 market constructor 제공 |
| Domestic | 실시간 체결통보 | `DomesticExecutionNotice` | `H0STCNI0`, `H0STCNI9` | AES-256-CBC/base64 복호화 지원 |
| Overseas | 실시간 체결통보 | `OverseasExecutionNotice` | `H0GSCNI0`, `H0GSCNI9` | AES-256-CBC/base64 복호화 지원 |

## Public Constructor Surface

도메인별 constructor가 `Subscription` protocol DTO를 생성한다.

| Constructor | TR ID Policy | `tr_key` |
|---|---|---|
| `websocket::domestic::realtime_price_subscription` | `DomesticRealtimePriceMarket`에서 선택 | 종목코드 |
| `websocket::domestic::execution_notice_subscription` | `Environment::Real`/`Virtual`에서 선택 | HTS ID |
| `websocket::overseas::execution_notice_subscription` | `Environment::Real`/`Virtual`에서 선택 | HTS ID |

## Deferred

| Area | Status | Notes |
|---|---|---|
| Domestic realtime asking price | Deferred | 실시간 호가는 focused v1 범위 밖이다. |
| Domestic expected conclusion | Deferred | 예상체결 feed는 focused v1 범위 밖이다. |
| Domestic index feeds | Deferred | 지수/업종 WebSocket feed는 focused v1 범위 밖이다. |
| Overseas quote feeds | Deferred | 해외 시세 WebSocket feed는 focused v1 범위 밖이다. |

## Implementation Checklist

새 WebSocket typed event를 추가할 때는 아래 순서로 진행한다.

1. 공식 샘플에서 TR ID, `tr_key`, payload 컬럼, 암호화 여부를 확인한다.
2. 도메인 모듈에 constructor를 추가하고 공통 `Subscription`은 protocol DTO로 유지한다.
3. raw `RealtimeDataFrame`을 lossless하게 보존한다.
4. typed view는 확인된 컬럼만 `Decimal`/`String` 등으로 파싱한다.
5. caret payload field count mismatch와 숫자 파싱 실패 테스트를 추가한다.
6. 암호화 feed는 key/iv 길이, base64, decrypt 실패 테스트를 추가한다.
7. 실제 WebSocket 호출 없이 fixture로 parser와 subscription JSON을 검증한다.
