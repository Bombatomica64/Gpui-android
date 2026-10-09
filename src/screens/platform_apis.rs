//! Exercises gpui-mobile's platform packages on the device, the way an app would
//! call them, and logs what each call returned and how long it took.

use std::fmt::Debug;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use gpui::{Context, IntoElement, ParentElement, Render, SharedString, Styled, Task, Window, div, px};
use gpui_kit::component::{button::Button, h_flex, v_flex};
use gpui_mobile::packages::{
    audio, calendar, contacts, deeplink, file_selector, image_picker, local_auth, location,
    maps_launcher, media_session, microphone, notifications, permission_handler as perms,
    url_launcher,
};

use crate::ui::{self, EventLog, prelude::*};

/// Links delivered to the deep-link handler, newest last.
static DEEP_LINKS: Mutex<Vec<String>> = Mutex::new(Vec::new());
/// Media-session actions delivered to the handler, newest last.
static MEDIA_ACTIONS: Mutex<Vec<String>> = Mutex::new(Vec::new());

/// Size on disk and in pixels of a picked file, to check it is readable and scaled.
fn describe(path: &str) -> String {
    let bytes = std::fs::metadata(path).map(|m| m.len());
    let pixels = image::image_dimensions(path);
    format!("{bytes:?} bytes, {pixels:?} px")
}

const AUDIO_URL: &str = "https://www.soundhelix.com/examples/mp3/SoundHelix-Song-1.mp3";

pub struct PlatformApisScreen {
    /// Run calls inside the click handler (as most apps would) instead of on a
    /// background thread.
    on_gpui_thread: bool,
    log: EventLog,
    /// Longest gap between two heartbeat ticks of the GPUI thread.
    worst_stall: Duration,
    _heartbeat: Task<()>,
    player: Option<Arc<audio::AudioPlayer>>,
}

impl PlatformApisScreen {
    pub fn new(_: &mut Window, cx: &mut Context<Self>) -> Self {
        // A tick every 100 ms on the GPUI thread: a longer gap means something blocked it.
        let heartbeat = cx.spawn(async move |this, cx| {
            let mut last = Instant::now();
            loop {
                cx.background_executor().timer(Duration::from_millis(100)).await;
                let gap = last.elapsed();
                last = Instant::now();
                let alive = this.update(cx, |this, cx| {
                    if gap > Duration::from_millis(400) {
                        this.log.push(format!("GPUI thread stalled {} ms", gap.as_millis()));
                        this.worst_stall = this.worst_stall.max(gap);
                        cx.notify();
                    }
                });
                if alive.is_err() {
                    break;
                }
            }
        });
        deeplink::set_deep_link_handler(|url| {
            log::info!("event: deep link handler <- {url}");
            DEEP_LINKS.lock().unwrap().push(url.to_string());
        });
        Self {
            on_gpui_thread: true,
            log: EventLog::default(),
            worst_stall: Duration::ZERO,
            _heartbeat: heartbeat,
            player: None,
        }
    }

    /// Run `call`, on the GPUI thread or a background one, and log its result.
    fn run<T: Debug + Send + 'static>(
        &mut self,
        name: &'static str,
        call: impl FnOnce() -> T + Send + 'static,
        cx: &mut Context<Self>,
    ) {
        if self.on_gpui_thread {
            let start = Instant::now();
            let result = call();
            self.record(name, "gpui", &result, start.elapsed(), cx);
        } else {
            cx.spawn(async move |this, cx| {
                let start = Instant::now();
                let result = cx.background_executor().spawn(async move { call() }).await;
                let elapsed = start.elapsed();
                let _ = this.update(cx, |this, cx| this.record(name, "bg", &result, elapsed, cx));
            })
            .detach();
        }
    }

    fn record(&mut self, name: &str, thread: &str, result: &dyn Debug, elapsed: Duration, cx: &mut Context<Self>) {
        let mut text = format!("{result:?}");
        if text.len() > 600 {
            let mut end = 600;
            while !text.is_char_boundary(end) {
                end -= 1;
            }
            text.truncate(end);
            text.push('…');
        }
        self.log.push(format!("{name} [{thread} {} ms] -> {text}", elapsed.as_millis()));
        cx.notify();
    }

    /// Calls that show no UI, one after another.
    fn run_read_only_checks(&mut self, cx: &mut Context<Self>) {
        self.run("get_calendars", calendar::get_calendars, cx);
        self.run("get_contacts", || contacts::get_contacts().map(|list| list.len()), cx);
        self.run("is_location_service_enabled", location::is_location_service_enabled, cx);
        self.run("get_last_known_position", location::get_last_known_position, cx);
        self.run("get_current_position", || location::get_current_position(&Default::default()), cx);
        self.run("can_launch_url", || {
            (
                url_launcher::can_launch_url("https://example.com"),
                url_launcher::can_launch_url("mailto:a@example.com"),
                maps_launcher::is_available(),
            )
        }, cx);
        self.run("local_auth info", || {
            (
                local_auth::is_device_supported(),
                local_auth::can_authenticate(),
                local_auth::get_available_biometrics(),
            )
        }, cx);
        self.run("deeplink", || {
            (deeplink::get_initial_link(), deeplink::get_latest_link(), DEEP_LINKS.lock().unwrap().clone())
        }, cx);
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

    fn row(buttons: impl IntoIterator<Item = Button>) -> impl IntoElement {
        h_flex().flex_wrap().gap_1().children(buttons)
    }
}

fn permission_buttons(cx: &mut Context<PlatformApisScreen>) -> Vec<Button> {
    let list: [(&'static str, &'static str, perms::Permission); 6] = [
        ("perm-camera", "Camera", perms::Permission::Camera),
        ("perm-contacts", "Contacts", perms::Permission::Contacts),
        ("perm-calendar", "Calendar", perms::Permission::Calendar),
        ("perm-location", "Location", perms::Permission::LocationWhenInUse),
        ("perm-notif", "Notifications", perms::Permission::Notification),
        ("perm-mic", "Microphone", perms::Permission::Microphone),
    ];
    list.into_iter()
        .map(|(id, label, permission)| {
            PlatformApisScreen::button(id, label, cx, move |this, cx| {
                this.run("request_permission", move || {
                    let before = perms::check_permission(permission);
                    let after = perms::request_permission(permission);
                    format!("{permission:?}: check {before:?}, request {after:?}")
                }, cx)
            })
        })
        .collect()
}

impl Render for PlatformApisScreen {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let thread_label: SharedString = if self.on_gpui_thread {
            "Calls run on: GPUI thread (click handler)".into()
        } else {
            "Calls run on: background thread".into()
        };
        v_flex()
            .gap_3()
            .child(
                ui::section("How calls run", cx)
                    .child(Self::row([Self::button("thread", "Toggle thread", cx, |this, cx| {
                        this.on_gpui_thread = !this.on_gpui_thread;
                        cx.notify();
                    })
                    .selected(!self.on_gpui_thread)]))
                    .child(ui::value_row("Mode", thread_label, cx))
                    .child(ui::value_row(
                        "Worst GPUI stall",
                        format!("{} ms", self.worst_stall.as_millis()),
                        cx,
                    ))
                    .child(Self::row([Self::button("read-only", "Run read-only checks", cx, |this, cx| {
                        this.run_read_only_checks(cx)
                    })]))
                    // Fixed height, so the buttons below stay put as results come in.
                    .child(div().h(px(230.)).overflow_hidden().child(self.log.render(cx)))
                    .child(ui::hint(
                        "Every result is also logged as `event:` in logcat (tag GPUI_MOBILE_LAB). \
                         A heartbeat on the GPUI thread logs any stall over 400 ms.",
                        cx,
                    )),
            )
            .child(
                ui::section("Files and images", cx)
                    .child(Self::row([
                        Self::button("open-file", "Open file", cx, |this, cx| {
                            this.run("open_file", || {
                                file_selector::open_file(&Default::default())
                                    .map(|file| file.map(|file| (describe(&file.path), file)))
                            }, cx)
                        }),
                        Self::button("open-files", "Open files", cx, |this, cx| {
                            this.run("open_files", || file_selector::open_files(&Default::default()), cx)
                        }),
                        Self::button("save-path", "Save path", cx, |this, cx| {
                            this.run("get_save_path", || {
                                file_selector::get_save_path(&file_selector::SaveFileOptions {
                                    suggested_name: Some("lab.txt".into()),
                                    ..Default::default()
                                })
                            }, cx)
                        }),
                        Self::button("dir", "Directory", cx, |this, cx| {
                            this.run("get_directory_path", || file_selector::get_directory_path(None), cx)
                        }),
                    ]))
                    .child(Self::row([
                        Self::button("pick-image", "Gallery image", cx, |this, cx| {
                            this.run("pick_image(gallery)", || {
                                image_picker::pick_image(&Default::default())
                                    .map(|file| file.map(|file| (describe(&file.path), file)))
                            }, cx)
                        }),
                        Self::button("pick-resized", "Image ≤256 px", cx, |this, cx| {
                            this.run("pick_image(max 256)", || {
                                image_picker::pick_image(&image_picker::ImagePickerOptions {
                                    max_width: Some(256.),
                                    max_height: Some(256.),
                                    image_quality: Some(50),
                                    ..Default::default()
                                })
                                .map(|file| file.map(|file| (describe(&file.path), file)))
                            }, cx)
                        }),
                        Self::button("take-photo", "Take photo", cx, |this, cx| {
                            this.run("pick_image(camera)", || {
                                image_picker::pick_image(&image_picker::ImagePickerOptions {
                                    source: image_picker::ImageSource::Camera,
                                    ..Default::default()
                                })
                                .map(|file| file.map(|file| (describe(&file.path), file)))
                            }, cx)
                        }),
                        Self::button("pick-multi", "Several images", cx, |this, cx| {
                            this.run("pick_multi_image", || image_picker::pick_multi_image(None, None, None), cx)
                        }),
                        Self::button("pick-video", "Gallery video", cx, |this, cx| {
                            this.run("pick_video", || {
                                image_picker::pick_video(image_picker::ImageSource::Gallery, Default::default())
                            }, cx)
                        }),
                    ])),
            )
            .child(
                ui::section("Biometrics", cx).child(Self::row([
                    Self::button("auth-info", "Capabilities", cx, |this, cx| {
                        this.run("local_auth info", || {
                            (
                                local_auth::is_device_supported(),
                                local_auth::can_authenticate(),
                                local_auth::get_available_biometrics(),
                            )
                        }, cx)
                    }),
                    Self::button("auth", "Authenticate", cx, |this, cx| {
                        this.run("authenticate", || local_auth::authenticate("Lab test"), cx)
                    }),
                ])),
            )
            .child(
                ui::section("Permissions (check, then request)", cx)
                    .child(Self::row(permission_buttons(cx)))
                    .child(Self::row([Self::button("settings", "App settings", cx, |this, cx| {
                        this.run("open_app_settings", perms::open_app_settings, cx)
                    })])),
            )
            .child(
                ui::section("Calendar, contacts, location", cx)
                    .child(Self::row([
                        Self::button("calendars", "Calendars", cx, |this, cx| {
                            this.run("get_calendars", calendar::get_calendars, cx)
                        }),
                        Self::button("events", "Events ±30 d", cx, |this, cx| {
                            this.run("get_events", || {
                                let now = chrono::Utc::now().timestamp_millis();
                                let month = 30 * 24 * 3600 * 1000;
                                calendar::get_calendars().map(|calendars| {
                                    calendars
                                        .iter()
                                        .map(|c| (c.name.clone(), calendar::get_events(&c.id, now - month, now + month)))
                                        .collect::<Vec<_>>()
                                })
                            }, cx)
                        }),
                        Self::button("create-event", "Create | event", cx, |this, cx| {
                            this.run("create_event", || {
                                let calendars = calendar::get_calendars()?;
                                let target = calendars
                                    .iter()
                                    .find(|c| !c.is_read_only)
                                    .ok_or("no writable calendar")?;
                                let now = chrono::Utc::now().timestamp_millis();
                                let event = calendar::CalendarEvent {
                                    id: String::new(),
                                    title: "Lab | pipe".into(),
                                    description: "line one\nline two | with pipe".into(),
                                    location: "Room 1|2".into(),
                                    start_ms: now + 3_600_000,
                                    end_ms: now + 7_200_000,
                                    all_day: false,
                                    calendar_id: target.id.clone(),
                                };
                                let id = calendar::create_event(&event)?;
                                let read_back = calendar::get_events(&target.id, now, now + 10_800_000)?;
                                let read_back: Vec<_> = read_back.into_iter().filter(|e| e.id == id).collect();
                                let deleted = calendar::delete_event(&id);
                                Ok::<_, String>((id, read_back, deleted))
                            }, cx)
                        }),
                        Self::button("contacts", "Contacts", cx, |this, cx| {
                            this.run("get_contacts", || {
                                contacts::get_contacts().map(|list| {
                                    let count = list.len();
                                    (count, list.into_iter().take(3).collect::<Vec<_>>())
                                })
                            }, cx)
                        }),
                    ]))
                    .child(Self::row([
                        Self::button("loc-service", "Location on?", cx, |this, cx| {
                            this.run("is_location_service_enabled", location::is_location_service_enabled, cx)
                        }),
                        Self::button("loc-last", "Last position", cx, |this, cx| {
                            this.run("get_last_known_position", location::get_last_known_position, cx)
                        }),
                        Self::button("loc-current", "Current position", cx, |this, cx| {
                            this.run("get_current_position", || location::get_current_position(&Default::default()), cx)
                        }),
                    ])),
            )
            .child(
                ui::section("Links and notifications", cx)
                    .child(Self::row([
                        Self::button("can-launch", "can_launch https", cx, |this, cx| {
                            this.run("can_launch_url", || {
                                (
                                    url_launcher::can_launch_url("https://example.com"),
                                    url_launcher::can_launch_url("mailto:a@example.com"),
                                    maps_launcher::is_available(),
                                )
                            }, cx)
                        }),
                        Self::button("launch", "Open example.com", cx, |this, cx| {
                            this.run("launch_url", || url_launcher::launch_url("https://example.com"), cx)
                        }),
                        Self::button("maps", "Open map", cx, |this, cx| {
                            this.run("open_coordinates", || maps_launcher::open_coordinates(45.4642, 9.19, Some("Milano")), cx)
                        }),
                    ]))
                    .child(Self::row([
                        Self::button("links", "Deep links", cx, |this, cx| {
                            this.run("deeplink", || {
                                (
                                    deeplink::get_initial_link(),
                                    deeplink::get_latest_link(),
                                    DEEP_LINKS.lock().unwrap().clone(),
                                )
                            }, cx)
                        }),
                        Self::button("payload", "Launch payload", cx, |this, cx| {
                            this.run("take_launch_payload", notifications::take_launch_payload, cx)
                        }),
                        Self::button("notify", "Notify", cx, |this, cx| {
                            this.run("notifications::show", || {
                                notifications::initialize()?;
                                notifications::show(&notifications::Notification {
                                    id: 7,
                                    title: "GPUI Mobile Lab".into(),
                                    body: "Tap to open the lab".into(),
                                    channel: Default::default(),
                                    payload: Some("lab-payload".into()),
                                })
                            }, cx)
                        }),
                    ])),
            )
            .child(
                ui::section("Media", cx)
                    .child(Self::row([
                        Self::button("audio-load", "Load stream", cx, |this, cx| {
                            let player = match audio::AudioPlayer::new() {
                                Ok(player) => Arc::new(player),
                                Err(err) => {
                                    this.log.push(format!("AudioPlayer::new -> {err}"));
                                    return;
                                }
                            };
                            this.player = Some(player.clone());
                            this.run("audio set_url", move || player.set_url(AUDIO_URL), cx)
                        }),
                        Self::button("audio-play", "Play", cx, |this, cx| {
                            if let Some(player) = this.player.clone() {
                                this.run("audio play", move || player.play(), cx)
                            }
                        }),
                        Self::button("audio-pause", "Pause", cx, |this, cx| {
                            if let Some(player) = this.player.clone() {
                                this.run("audio pause", move || player.pause(), cx)
                            }
                        }),
                        Self::button("audio-loop", "Loop One", cx, |this, cx| {
                            if let Some(player) = this.player.clone() {
                                this.run("audio loop One", move || player.set_loop_mode(audio::LoopMode::One), cx)
                            }
                        }),
                        Self::button("audio-state", "State", cx, |this, cx| {
                            if let Some(player) = this.player.clone() {
                                this.run("audio state", move || (player.state(), player.position(), player.duration()), cx)
                            }
                        }),
                    ]))
                    .child(Self::row([
                        Self::button("session", "Media session", cx, |this, cx| {
                            media_session::set_action_handler(|action| {
                                log::info!("event: media action <- {action:?}");
                                MEDIA_ACTIONS.lock().unwrap().push(format!("{action:?}"));
                            });
                            this.run("media_session init", || {
                                media_session::init()?;
                                media_session::set_metadata("Lab song", "GPUI", 180_000)?;
                                media_session::set_playback_state(true, 0, 1.0)
                            }, cx)
                        }),
                        Self::button("session-actions", "Actions", cx, |this, cx| {
                            this.run("media actions", || MEDIA_ACTIONS.lock().unwrap().clone(), cx)
                        }),
                        Self::button("mic-start", "Record", cx, |this, cx| {
                            this.run("start_recording", || microphone::start_recording(&Default::default()), cx)
                        }),
                        Self::button("mic-stop", "Stop", cx, |this, cx| {
                            this.run("stop_recording", microphone::stop_recording, cx)
                        }),
                    ])),
            )
    }
}
