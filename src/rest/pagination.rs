use crate::error::{Error, Result};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PageLimit {
    All,
    Max(usize),
}

impl PageLimit {
    pub(crate) fn max_pages(self) -> Result<Option<usize>> {
        match self {
            Self::All => Ok(None),
            Self::Max(0) => Err(Error::config("page limit must be at least 1")),
            Self::Max(value) => Ok(Some(value)),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PageStopReason {
    Exhausted,
    PageLimitReached,
    RateLimited { error: Error },
}

#[derive(Clone, Debug, PartialEq)]
pub struct PageCollection<T, C> {
    pub pages: Vec<T>,
    pub next: Option<C>,
    pub stop_reason: PageStopReason,
}
