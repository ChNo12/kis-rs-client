# AGENTS.md

**kis-rs-client** — 한국투자증권(KIS) REST API와 WebSocket API를 Rust에서 안전하게 다루기 위한 클라이언트 라이브러리.

현재는 초기 crate 골격 상태다. 사소한 변경은 판단껏 진행하되, 인증/주문/실거래 가능성이 있는 변경은 신중함을 우선한다.

## 1. Think Before Coding

**가정하지 마라. 혼란을 숨기지 마라. 트레이드오프를 드러내라.**

- 가정은 명시한다. 불확실하면 묻는다.
- 해석이 여러 개면 제시한다. 조용히 하나를 고르지 않는다.
- 더 단순한 접근이 있으면 말한다.
- 진행할 수 없을 만큼 막히면 멈춘다. 무엇이 불분명한지 이름 붙이고 묻는다.
- KIS API 경로, TR ID, 요청/응답 필드, 인증 방식은 바뀔 수 있다. 추가/수정 시 공식 문서 확인이 필요하면 먼저 밝힌다.

## 2. Simplicity First

**문제를 푸는 최소 코드. 추측성 코드 금지.**

- 요청되지 않은 기능을 추가하지 않는다.
- 일회성 코드에 추상화를 만들지 않는다.
- "나중에 필요할 유연성"을 임의로 넣지 않는다.
- 요청되지 않은 방어 코드는 넣지 않는다. 단, 외부 입력, 네트워크 응답, 인증, 주문, 금액/수량 같은 금융 불변식 검증은 유지한다.
- 시니어가 보고 과설계라고 판단할 구조라면 단순화한다.

## 3. Surgical Changes

**필요한 곳만 건드린다. 자기가 만든 흔적만 치운다.**

- 인접 코드, 주석, 포맷을 이유 없이 개선하지 않는다.
- 망가지지 않은 코드를 리팩터하지 않는다.
- 기존 스타일을 따른다.
- Rust 경로는 특별한 이유가 없으면 본문에서 `crate::foo::bar::Type`처럼 길게 쓰지 말고 `use`로 import한 뒤 짧은 이름을 사용한다. 이름 충돌이나 의미 명확성이 필요하면 `use ... as ...` alias를 사용한다.
- 무관한 dead code를 발견하면 언급만 하고 지우지 않는다.
- 본인 변경으로 고아가 된 import, 변수, 함수만 제거한다.
- 모든 변경 줄은 사용자 요청까지 직접 추적될 수 있어야 한다.

## 4. Goal-Driven Execution

**성공 기준을 정의하고, 검증될 때까지 루프한다.**

- 검증 추가: 잘못된 입력 테스트를 쓰고 통과시킨다.
- 버그 수정: 가능하면 재현 테스트를 먼저 쓰고 통과시킨다.
- 리팩터: 전후 모두 관련 테스트가 통과해야 한다.
- 다단계 작업은 짧은 계획으로 진행한다.

```text
1. [단계] -> 검증: [확인]
2. [단계] -> 검증: [확인]
```

## 5. Comment Policy

**최소, 명확, 일관. 코드가 자명하면 주석/문서는 쓰지 않는다. 공개 API라도 예외가 아니다.**

- `///` 또는 `//`는 WHY가 비자명할 때만 쓴다. 이유, 제약, 불변 조건, 외부 API 특이사항, 도메인 규칙을 남긴다.
- 코드와 의미가 중복되는 주석은 쓰지 않는다.
- 코드 수정 시 인접 주석도 같은 변경 안에서 갱신한다.
- 주석과 rustdoc 본문은 한국어를 우선한다.
- `thiserror`의 `#[error(...)]`, `tracing` 로그 본문, 외부 API 키/식별자 인용은 필요하면 영어를 유지한다.

## 6. Testing

동작이 변경될 때 테스트를 추가하거나 업데이트한다.

- 단위 테스트는 같은 파일 하단의 `#[cfg(test)] mod tests`에 둔다.
- 통합 테스트는 `tests/`에 둔다.
- 비동기는 `#[tokio::test]`, 동기는 `#[test]`를 사용한다.
- 순수 로직, 구문 분석, 유효성 검사, 매핑, 서식 지정, 예외 상황은 단위 테스트로 검증한다.
- API 클라이언트 동작, 직렬화, WebSocket 메시지 흐름은 통합 테스트로 검증한다.
- 버그 수정에는 가능하면 회귀 테스트를 추가한다.
- 광범위하거나 취약한 테스트보다 특정 목표에 집중한 결정론적 테스트를 우선한다.
- 사소한 구현 세부 사항은 테스트하지 않는다.
- 자동화된 테스트에서 실제 KIS API나 외부 서비스를 호출하지 않는다. mock, fixture, 로컬 테스트 서버를 사용한다.
- 실제 KIS API 호출 테스트는 `#[ignore]`, 명시적 feature, 환경 변수 중 하나로 보호한다.
- 실거래 주문 테스트는 기본 금지다. 꼭 필요하면 사용자 명시 승인, opt-in feature, 모의투자 우선, 주문 금액/수량 제한을 둔다.
- 작업 완료 전 가장 관련성 높은 테스트를 실행하고 실행한 내용을 보고한다.

## 프로젝트 특화 규칙

### KIS API 안전성

- 실전투자와 모의투자 base URL, TR ID, 인증 흐름을 명확히 구분한다.
- 실전/모의 차이를 호출 지점마다 `if mock { ... }`로 흩뿌리지 않는다. 설정, endpoint 정책, trait 구현체 등 한 경계로 모은다.
- 주문 API는 조회 API보다 더 보수적으로 다룬다. 기본 예제와 테스트는 조회 또는 모의투자 중심으로 작성한다.
- access token, app key, app secret, 계좌번호 전체값은 `Debug`, 로그, 에러 메시지에 그대로 노출하지 않는다.
- `.env`, 인증키, 토큰, 계좌번호가 들어간 파일은 커밋하지 않는다.

### REST API

- 요청 생성, 인증 헤더 삽입, 응답 파싱, KIS 오류 코드 처리를 분리한다.
- API별 raw response 구조와 사용자가 다루는 domain type을 가능하면 분리한다.
- KIS 원문 필드명, TR ID, API 경로, 문서 용어는 serde rename, 상수, 주석 등으로 원형을 보존한다.
- retry 가능 오류와 재시도하면 안 되는 주문 오류를 구분한다.
- rate limit, token expiration, clock skew 가능성을 명시적으로 고려한다.

### WebSocket API

- 접속, 인증, 구독 요청, 수신 루프, 재연결 정책을 분리한다.
- 구독 상태는 재연결 후 복구할 수 있게 보관한다.
- text/binary frame, heartbeat, ping/pong, 서버 오류 메시지를 명시적으로 처리한다.
- 메시지 파싱은 lossless raw representation을 먼저 두고, 그 위에 typed event를 제공하는 방식을 우선한다.
- 수신 루프가 사용자 콜백 또는 채널 backpressure 때문에 영구 정지하지 않게 설계한다.

### 타입과 정밀도

- 공개 API는 Rust 관용에 맞춰 영어 식별자를 사용한다.
- 금액, 가격, 수량, 환율 같은 숫자형 KIS 응답 필드는 typed view에서 기본적으로 `rust_decimal::Decimal`로 표현한다. `f64` 또는 `i64`는 명확한 도메인 이유가 있을 때만 사용한다.
- 계좌번호, 종목코드, TR ID처럼 의미 있는 문자열은 필요하면 newtype을 사용한다.
- DTO나 snapshot 성격의 타입은 public field를 허용한다. trivial getter를 만들지 않는다.
- 모듈 안에서 `KisClient`처럼 crate명을 반복하는 C-STUTTER 네이밍을 피한다. `client::Client`처럼 둔다.
- `unwrap`, `expect`, `panic!`은 테스트가 아닌 라이브러리 코드에서 피한다.

### 의존성과 경계

- 현재는 단일 crate다. crate가 늘어나면 순수 도메인 타입과 오류는 인프라 의존성(reqwest, WebSocket client, async runtime)에 의존하지 않게 둔다.
- HTTP, WebSocket, clock처럼 테스트 구현체가 필요한 경계는 trait으로 역전한다. 구현체가 1개여도 외부 효과가 있으면 경계를 검토한다.
- 새 의존성은 실제 필요가 생겼을 때 추가한다. 비동기 런타임, HTTP/WebSocket client, decimal 타입은 프로젝트 방향과 MSRV/edition 영향을 확인한다.

## 권장 모듈 구조

구현이 커질 때는 다음 구조를 우선 검토한다. 확정 규칙은 아니며, 기존 코드가 다른 방향으로 정리되면 그 패턴을 우선한다.

```text
src/
  lib.rs
  client.rs
  config.rs
  error.rs
  auth.rs
  rest/
    mod.rs
    domestic.rs
    overseas.rs
    order.rs
  websocket/
    mod.rs
    client.rs
    message.rs
    subscription.rs
  models/
    mod.rs
```

권장 환경 변수 이름:

```text
KIS_APP_KEY
KIS_APP_SECRET
KIS_ACCOUNT_NO
KIS_ACCOUNT_PRODUCT_CODE
KIS_USE_MOCK
```

## 빌드 / 테스트

작업 후 가능한 범위에서 아래 명령을 실행한다.

```bash
cargo fmt --all
cargo build
cargo test
cargo clippy --all-targets --all-features -- -D warnings
```

의존성 다운로드나 네트워크가 필요한 명령은 사용자 승인 없이 진행하지 않는다.

## 에이전트 작업 규칙

- 작업 전 `git status --short`로 사용자 변경사항을 확인한다.
- 사용자가 만든 변경을 되돌리지 않는다.
- 파일 검색은 우선 `rg` 또는 `rg --files`를 사용한다.
- 변경 범위는 요청된 기능과 직접 관련된 파일로 제한한다.
- 파일은 마지막에 LF 한 개로 끝낸다.
- 네트워크 접근, 의존성 다운로드, 실제 API 호출, 실거래 주문 가능 작업은 사용자 승인 없이 진행하지 않는다.
- 본 문서와 향후 추가될 설계 문서가 충돌하면 더 구체적인 프로젝트 설계 문서를 우선한다.
