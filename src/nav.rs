//! Motion for the screen stack: push and pop transitions, and the predictive back
//! preview that follows a back swipe on Android 14+.
//!
//! Everything is one number, the top page's horizontal offset as a fraction of the
//! content width: 0 in place, 1 fully off to the right, -1 fully off to the left.
//! The page under it follows with a parallax and a scrim, so `app.rs` only places
//! two layers.

use std::time::{Duration, Instant};

/// How far the page underneath sits to the side while covered (fraction of width).
const PARALLAX: f32 = 0.25;
/// How far a back swipe drags the top page before release (fraction of width).
/// The system's progress is already eased, so the page tracks it linearly.
pub const PREVIEW_TRAVEL: f32 = 0.3;
/// Scrim over the page underneath while it is fully covered.
pub const SCRIM: f32 = 0.3;

const PUSH: Duration = Duration::from_millis(300);
const POP: Duration = Duration::from_millis(250);
const SETTLE: Duration = Duration::from_millis(200);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Motion {
    /// A new page slides in over the current one.
    Push,
    /// The top page slides out; it is dropped once the motion ends.
    Pop,
    /// A cancelled back preview returns to rest.
    Settle,
}

#[derive(Clone, Copy, Debug)]
pub struct Transition {
    pub motion: Motion,
    start: Instant,
    from: f32,
    to: f32,
    duration: Duration,
}

impl Transition {
    pub fn push(now: Instant) -> Self {
        Self::new(Motion::Push, now, 1.0, 0.0, PUSH)
    }

    /// Pop from `from` (0 at rest, or where a back preview left the page) towards
    /// the side it is already moving to.
    pub fn pop(now: Instant, from: f32) -> Self {
        let to = if from < 0.0 { -1.0 } else { 1.0 };
        let duration = POP.mul_f32((1.0 - from.abs()).clamp(0.4, 1.0));
        Self::new(Motion::Pop, now, from, to, duration)
    }

    pub fn settle(now: Instant, from: f32) -> Self {
        Self::new(Motion::Settle, now, from, 0.0, SETTLE)
    }

    fn new(motion: Motion, start: Instant, from: f32, to: f32, duration: Duration) -> Self {
        Self {
            motion,
            start,
            from,
            to,
            duration,
        }
    }

    /// The top page's offset at `now`, and whether the motion has finished.
    pub fn sample(&self, now: Instant) -> (f32, bool) {
        let elapsed = now.saturating_duration_since(self.start).as_secs_f32();
        let t = (elapsed / self.duration.as_secs_f32()).clamp(0.0, 1.0);
        (
            self.from + (self.to - self.from) * ease_out_cubic(t),
            t >= 1.0,
        )
    }
}

/// A back swipe in progress, before Android commits or cancels it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Preview {
    pub progress: f32,
    /// The swipe came from the right edge: the page moves left.
    pub from_right: bool,
}

impl Preview {
    pub fn offset(&self) -> f32 {
        let offset = self.progress.clamp(0.0, 1.0) * PREVIEW_TRAVEL;
        if self.from_right { -offset } else { offset }
    }
}

/// Offset of the page underneath, given the top page's.
pub fn under_offset(top: f32) -> f32 {
    let direction = if top < 0.0 { 1.0 } else { -1.0 };
    direction * PARALLAX * (1.0 - top.abs().min(1.0))
}

/// Scrim opacity over the page underneath, given the top page's offset.
pub fn scrim(top: f32) -> f32 {
    SCRIM * (1.0 - top.abs().min(1.0))
}

/// Where a deep link (`gpuilab://…`) or a launch intent sends the app.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Route {
    /// `gpuilab://catalog`: back to the catalog.
    Catalog,
    /// `gpuilab://screen/Dialogs`, or `gpuilab://screen/Navigation/Buttons` for a
    /// back stack: each title is pushed on top of the catalog, in order. Titles
    /// are percent-decoded (`Text%20Input`).
    Screens(Vec<String>),
}

impl Route {
    pub fn parse(url: &str) -> Option<Self> {
        let rest = url.strip_prefix("gpuilab:")?.trim_start_matches('/');
        let rest = rest.split(['?', '#']).next().unwrap_or_default();
        let mut parts = rest.split('/').filter(|part| !part.is_empty());
        match parts.next()? {
            "catalog" => Some(Self::Catalog),
            "screen" => {
                let titles: Vec<String> = parts.map(percent_decode).collect();
                (!titles.is_empty()).then_some(Self::Screens(titles))
            }
            _ => None,
        }
    }
}

fn percent_decode(text: &str) -> String {
    let bytes = text.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        let hex = |b: u8| (b as char).to_digit(16);
        match (bytes[i], bytes.get(i + 1), bytes.get(i + 2)) {
            (b'%', Some(&hi), Some(&lo)) if hex(hi).is_some() && hex(lo).is_some() => {
                out.push((hex(hi).unwrap() * 16 + hex(lo).unwrap()) as u8);
                i += 3;
            }
            (b'+', _, _) => {
                out.push(b' ');
                i += 1;
            }
            (b, _, _) => {
                out.push(b);
                i += 1;
            }
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn ease_out_cubic(t: f32) -> f32 {
    1.0 - (1.0 - t).powi(3)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn push_slides_in_from_the_right() {
        let now = Instant::now();
        let push = Transition::push(now);
        assert_eq!(push.sample(now), (1.0, false));
        assert_eq!(push.sample(now + PUSH), (0.0, true));
    }

    #[test]
    fn pop_continues_a_preview_in_its_direction() {
        let now = Instant::now();
        let left = Preview {
            progress: 1.0,
            from_right: true,
        };
        let pop = Transition::pop(now, left.offset());
        let (end, done) = pop.sample(now + POP);
        assert!(done);
        assert_eq!(end, -1.0);
        let (mid, _) = pop.sample(now + POP / 4);
        assert!(mid < left.offset() && mid > -1.0);
    }

    #[test]
    fn links_name_a_stack_of_screens() {
        assert_eq!(Route::parse("gpuilab://catalog"), Some(Route::Catalog));
        assert_eq!(
            Route::parse("gpuilab://screen/Navigation/Text%20Input?x=1"),
            Some(Route::Screens(vec![
                "Navigation".into(),
                "Text Input".into()
            ]))
        );
        assert_eq!(Route::parse("gpuilab://screen"), None);
        assert_eq!(Route::parse("https://screen/Buttons"), None);
    }

    #[test]
    fn page_underneath_is_uncovered_as_the_top_leaves() {
        assert_eq!(under_offset(0.0), -PARALLAX);
        assert_eq!(under_offset(1.0), 0.0);
        assert_eq!(under_offset(-1.0), 0.0);
        assert_eq!(scrim(0.0), SCRIM);
        assert_eq!(scrim(1.0), 0.0);
    }
}
