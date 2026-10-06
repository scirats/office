use execution::Clock;
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;

#[derive(Debug, Clone, Copy, Default)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn now_rfc3339(&self) -> String {
        OffsetDateTime::now_utc()
            .format(&Rfc3339)
            .expect("RFC 3339 formatting is infallible")
    }
}
