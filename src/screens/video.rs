//! Exercises gpui-mobile's video player: a stream loaded from the GPUI thread
//! must not block it, report `Loading` until it is ready, and start if Play was
//! pressed meanwhile. The video draws in a native SurfaceView (a platform view)
//! placed over the box at the top of the screen.

use std::cell::Cell;
use std::rc::Rc;
use std::time::{Duration, Instant};

use gpui::{
    Bounds, Context, IntoElement, ParentElement, Pixels, Render, Styled, Task, Window, canvas, div, px,
};
use gpui_kit::component::{button::Button, h_flex, v_flex};
use gpui_mobile::packages::video_player::{VideoPlayer, VideoPlayerState};
use gpui_mobile::platform_view::PlatformViewBounds;

use crate::ui::{self, EventLog, prelude::*};

/// A public-domain test clip (Big Buck Bunny, ~10 s at 360p) served over https.
const VIDEO_URL: &str = "https://test-videos.co.uk/vids/bigbuckbunny/mp4/h264/360/Big_Buck_Bunny_360_10s_1MB.mp4";
const VIDEO_HEIGHT: f32 = 200.;

pub struct VideoScreen {
    log: EventLog,
    player: Option<VideoPlayer>,
    /// Where the video box was laid out last frame, in window coordinates.
    video_bounds: Rc<Cell<Option<Bounds<Pixels>>>>,
    /// When the current source was set, to time the state changes after it.
    loaded_at: Option<Instant>,
    last_state: Option<Result<VideoPlayerState, String>>,
    /// Longest gap between two ticks of the GPUI thread since the last load.
    worst_stall: Duration,
    _ticker: Task<()>,
}

impl VideoScreen {
    pub fn new(_: &mut Window, cx: &mut Context<Self>) -> Self {
        // Every 50 ms on the GPUI thread: log state changes, and any stall.
        let ticker = cx.spawn(async move |this, cx| {
            let mut last = Instant::now();
            loop {
                cx.background_executor().timer(Duration::from_millis(50)).await;
                let gap = last.elapsed();
                last = Instant::now();
                let alive = this.update(cx, |this, cx| this.tick(gap, cx));
                if alive.is_err() {
                    break;
                }
            }
        });
        Self {
            log: EventLog::default(),
            player: None,
            video_bounds: Rc::default(),
            loaded_at: None,
            last_state: None,
            worst_stall: Duration::ZERO,
            _ticker: ticker,
        }
    }

    fn tick(&mut self, gap: Duration, cx: &mut Context<Self>) {
        if gap > Duration::from_millis(250) {
            self.log.push(format!("GPUI thread stalled {} ms", gap.as_millis()));
            self.worst_stall = self.worst_stall.max(gap);
            cx.notify();
        }
        let Some(player) = &self.player else { return };
        let state = player.state();
        if self.last_state.as_ref() != Some(&state) {
            let since = self.loaded_at.map(|t| t.elapsed().as_millis()).unwrap_or_default();
            let detail = match state {
                Ok(VideoPlayerState::Loading) | Err(_) => String::new(),
                Ok(_) => format!(
                    ", duration {:?}, size {:?}, position {:?}",
                    player.duration(),
                    player.video_size(),
                    player.position()
                ),
            };
            self.log.push(format!("state {state:?} at +{since} ms{detail}"));
            self.last_state = Some(state);
            cx.notify();
        }
    }

    fn load(&mut self, cx: &mut Context<Self>) {
        if self.player.is_none() {
            match VideoPlayer::new() {
                Ok(player) => self.player = Some(player),
                Err(err) => {
                    self.log.push(format!("VideoPlayer::new -> {err}"));
                    cx.notify();
                    return;
                }
            }
        }
        let player = self.player.as_mut().unwrap();
        self.worst_stall = Duration::ZERO;
        self.last_state = None;
        let start = Instant::now();
        let result = player.set_url(VIDEO_URL);
        let elapsed = start.elapsed();
        self.loaded_at = Some(start);
        self.log.push(format!("set_url [{} ms] -> {result:?}", elapsed.as_millis()));
        // Show the surface right away, so it attaches while the stream is preparing.
        if let Some(bounds) = self.video_bounds.get() {
            let result = player.show_surface(
                bounds.origin.x.into(),
                bounds.origin.y.into(),
                bounds.size.width.into(),
                bounds.size.height.into(),
            );
            self.log.push(format!("show_surface -> {result:?}"));
        }
        cx.notify();
    }

    fn call(&mut self, name: &str, cx: &mut Context<Self>, call: impl FnOnce(&VideoPlayer) -> Result<(), String>) {
        let Some(player) = &self.player else {
            self.log.push(format!("{name}: load a stream first"));
            cx.notify();
            return;
        };
        let start = Instant::now();
        let result = call(player);
        let since = self.loaded_at.map(|t| t.elapsed().as_millis()).unwrap_or_default();
        self.log.push(format!(
            "{name} [{} ms] at +{since} ms (state {:?}) -> {result:?}",
            start.elapsed().as_millis(),
            player.state()
        ));
        cx.notify();
    }

    fn button(
        id: &'static str,
        label: &'static str,
        cx: &mut Context<Self>,
        on_click: impl Fn(&mut Self, &mut Context<Self>) + 'static,
    ) -> Button {
        Button::new(id)
            .small()
            .outline()
            .label(label)
            .on_click(cx.listener(move |this, _, _, cx| on_click(this, cx)))
    }
}

#[gpui_hot::hot]
impl Render for VideoScreen {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let video_bounds = self.video_bounds.clone();
        // The native view does not follow GPUI's layout by itself: move it to the box.
        let handle = self.player.as_ref().and_then(|p| p.platform_view_handle());
        let video_box = div()
            .w_full()
            .h(px(VIDEO_HEIGHT))
            .flex_none()
            .rounded(cx.theme().radius)
            .bg(cx.theme().muted)
            .child(
                canvas(
                    move |bounds, _, _| {
                        if video_bounds.get() != Some(bounds) {
                            video_bounds.set(Some(bounds));
                            if let Some(handle) = &handle {
                                handle.set_bounds(PlatformViewBounds {
                                    x: bounds.origin.x.into(),
                                    y: bounds.origin.y.into(),
                                    width: bounds.size.width.into(),
                                    height: bounds.size.height.into(),
                                });
                            }
                        }
                    },
                    |_, _, _, _| {},
                )
                .size_full(),
            );
        v_flex()
            .size_full()
            .p_3()
            .gap_3()
            .child(video_box)
            .child(
                h_flex()
                    .flex_wrap()
                    .gap_1()
                    .child(Self::button("video-load", "Load stream", cx, |this, cx| this.load(cx)))
                    .child(Self::button("video-play", "Play", cx, |this, cx| {
                        this.call("play", cx, |p| p.play())
                    }))
                    .child(Self::button("video-pause", "Pause", cx, |this, cx| {
                        this.call("pause", cx, |p| p.pause())
                    }))
                    .child(Self::button("video-seek", "Seek 5 s", cx, |this, cx| {
                        this.call("seek 5000", cx, |p| p.seek(5000))
                    })),
            )
            .child(ui::value_row(
                "Worst GPUI stall since load",
                format!("{} ms", self.worst_stall.as_millis()),
                cx,
            ))
            .child(self.log.render(cx))
            .child(ui::hint(
                "All calls run on the GPUI thread. set_url should return at once with None, \
                 the state go Loading -> Ready, and a Play pressed while loading start the \
                 video once it is ready. Events are logged as `event:` in logcat.",
                cx,
            ))
    }
}
