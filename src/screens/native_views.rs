//! Native Android views embedded in GPUI (#26): a tile map, a WebView and a
//! camera preview, each placed by `platform_view_element`, which moves the
//! native view to the element's bounds, clips it to the scroll area and hides
//! it when the element is no longer painted. "Recreate Activity" checks that
//! all three move to the new Activity (#87).

use std::time::Duration;

use gpui::{
    App, AppContext as _, Context, Entity, IntoElement, ParentElement, Render, Styled, Subscription, Task,
    TextInputAction, Window, div, px,
};
use gpui_kit::component::{
    button::Button,
    h_flex,
    input::{Input, InputContentType, InputEvent, InputState},
    v_flex,
};
use gpui_mobile::components::platform_view_element::platform_view_element;
use gpui_mobile::packages::camera::{self, CameraHandle, CameraLensDirection, ResolutionPreset};
use gpui_mobile::packages::maps::{LatLng, MapMarker, MapSettings, MapView};
use gpui_mobile::packages::permission_handler as perms;
use gpui_mobile::packages::webview::{self, WebViewHandle, WebViewSettings, WebViewState};
use gpui_mobile::platform_view::PlatformViewHandle;
use std::sync::Arc;

use crate::ui::{self, EventLog, prelude::*};

const MAP_HEIGHT: f32 = 220.;
const WEB_HEIGHT: f32 = 280.;
const CAMERA_HEIGHT: f32 = 240.;

const ROME: LatLng = LatLng {
    latitude: 41.8902,
    longitude: 12.4922,
};

/// A page that needs no network: a button and a text field inside the WebView
/// check that touches and the keyboard reach it, and `clicks` is read back by
/// "Run JS".
const LOCAL_PAGE: &str = r#"<!doctype html>
<html><head><meta name="viewport" content="width=device-width, initial-scale=1">
<title>GPUI lab page</title>
<style>
 body { font-family: sans-serif; margin: 12px; background: #f6f3ee; color: #222; }
 button, input { font-size: 16px; padding: 8px; margin: 4px 0; }
 a { display: block; margin-top: 8px; }
</style></head>
<body>
 <h3>Inside the WebView</h3>
 <button onclick="window.clicks = (window.clicks || 0) + 1;
   document.getElementById('n').textContent = window.clicks">Tap me</button>
 taps: <span id="n">0</span>
 <input id="t" placeholder="Type here (WebView keyboard)">
 <a href="https://example.org/">Go to example.org</a>
</body></html>"#;

pub struct NativeViewsScreen {
    log: EventLog,
    map: Option<MapView>,
    map_status: String,
    markers: u32,
    web: Option<WebViewHandle>,
    web_state: Option<WebViewState>,
    js_runs: u32,
    url: Entity<InputState>,
    camera: Option<CameraHandle>,
    camera_busy: bool,
    hidden: bool,
    _ticker: Task<()>,
    _subscriptions: Vec<Subscription>,
}

impl NativeViewsScreen {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let mut log = EventLog::default();

        let map = MapView::new(MapSettings {
            center: ROME,
            zoom: 14.,
            ..Default::default()
        });
        let map = match map {
            Ok(map) => {
                let colosseum = MapMarker {
                    id: "colosseum".into(),
                    position: ROME,
                    title: Some("Colosseum".into()),
                    snippet: Some("Tap a pin to open its bubble".into()),
                };
                log.push(format!(
                    "MapView::new -> Ok, add_marker -> {:?}",
                    map.add_marker(&colosseum)
                ));
                Some(map)
            }
            Err(err) => {
                log.push(format!("MapView::new -> {err}"));
                None
            }
        };

        let web = match webview::load_html(LOCAL_PAGE, &WebViewSettings::default()) {
            Ok(web) => Some(web),
            Err(err) => {
                log.push(format!("webview::load_html -> {err}"));
                None
            }
        };

        let url = cx.new(|cx| {
            InputState::new(window, cx)
                .default_value("https://example.org/")
                .input_action(TextInputAction::Go)
        });
        let subscriptions = vec![
            cx.subscribe_in(&url, window, |this, state, event: &InputEvent, _, cx| {
                if let InputEvent::PressEnter { .. } = event {
                    let url = state.read(cx).value().to_string();
                    this.web_call("navigate", cx, |web| webview::navigate(web, &url));
                }
            }),
        ];

        // Read back what the user does inside the native views.
        let ticker = cx.spawn(async move |this, cx| {
            loop {
                cx.background_executor().timer(Duration::from_millis(250)).await;
                if this.update(cx, |this, cx| this.poll(cx)).is_err() {
                    break;
                }
            }
        });

        Self {
            log,
            map,
            map_status: String::new(),
            markers: 0,
            web,
            web_state: None,
            js_runs: 0,
            url,
            camera: None,
            camera_busy: false,
            hidden: false,
            _ticker: ticker,
            _subscriptions: subscriptions,
        }
    }

    fn poll(&mut self, cx: &mut Context<Self>) {
        let mut changed = false;
        if let Some(map) = &self.map {
            let status = match (map.camera(), map.selected_marker()) {
                (Ok(camera), Ok(selected)) => format!(
                    "{:.4}, {:.4} at zoom {:.1}{}",
                    camera.center.latitude,
                    camera.center.longitude,
                    camera.zoom,
                    selected.map(|id| format!(", selected {id}")).unwrap_or_default()
                ),
                (Err(err), _) | (_, Err(err)) => err,
            };
            if status != self.map_status {
                self.map_status = status;
                changed = true;
            }
        }
        if let Some(web) = &self.web {
            let state = webview::state(web).ok();
            if state != self.web_state {
                if let Some(state) = &state {
                    let was = self.web_state.as_ref();
                    if was.map(|s| &s.url) != Some(&state.url) && !state.url.is_empty() {
                        self.log.push(format!("web: {}", state.url));
                    }
                    if let Some(error) = state
                        .error
                        .as_ref()
                        .filter(|e| was.and_then(|s| s.error.as_ref()) != Some(*e))
                    {
                        self.log.push(format!("web error: {error}"));
                    }
                }
                self.web_state = state;
                changed = true;
            }
        }
        if changed {
            cx.notify();
        }
    }

    fn map_call(
        &mut self,
        name: &str,
        cx: &mut Context<Self>,
        call: impl FnOnce(&mut MapView) -> Result<(), String>,
    ) {
        let result = match &mut self.map {
            Some(map) => call(map),
            None => Err("no map".into()),
        };
        self.log.push(format!("map {name} -> {result:?}"));
        cx.notify();
    }

    fn web_call(
        &mut self,
        name: &str,
        cx: &mut Context<Self>,
        call: impl FnOnce(&WebViewHandle) -> Result<(), String>,
    ) {
        let result = match &self.web {
            Some(web) => call(web),
            None => Err("no web view".into()),
        };
        self.log.push(format!("web {name} -> {result:?}"));
        cx.notify();
    }

    fn run_js(&mut self, cx: &mut Context<Self>) {
        let Some(web) = &self.web else { return };
        self.js_runs += 1;
        let script = format!(
            "document.body.style.background = '{}'; \
             JSON.stringify({{title: document.title, taps: window.clicks || 0, run: {}}})",
            if self.js_runs % 2 == 0 {
                "#f6f3ee"
            } else {
                "#dcefe1"
            },
            self.js_runs
        );
        let result = webview::evaluate_javascript_for_result(web, &script);
        cx.spawn(async move |this, cx| {
            let result = result.await;
            let _ = this.update(cx, |this, cx| {
                this.log.push(format!("JS result -> {result:?}"));
                cx.notify();
            });
        })
        .detach();
    }

    fn start_camera(&mut self, cx: &mut Context<Self>) {
        if self.camera.is_some() || self.camera_busy {
            return;
        }
        self.camera_busy = true;
        self.log.push("camera: asking for permission…");
        cx.notify();
        // The permission dialog and opening the camera wait: off GPUI's thread.
        let task = cx.background_spawn(async move {
            let permission = perms::request_permission(perms::Permission::Camera)?;
            if !permission.is_granted() {
                return Err(format!("camera permission {permission:?}"));
            }
            let cameras = camera::available_cameras()?;
            let description = cameras
                .iter()
                .find(|c| c.lens_direction == CameraLensDirection::Back)
                .or(cameras.first())
                .ok_or("no camera")?
                .clone();
            let mut handle = camera::create_camera(&description, ResolutionPreset::High, false)?;
            camera::start_preview(&mut handle)?;
            Ok::<_, String>((handle, description.name))
        });
        cx.spawn(async move |this, cx| {
            let result = task.await;
            let _ = this.update(cx, |this, cx| {
                this.camera_busy = false;
                match result {
                    Ok((handle, name)) => {
                        this.log.push(format!("camera {name}: preview started"));
                        if this.hidden {
                            if let Some(view) = camera::preview_platform_view_handle(&handle) {
                                view.set_visible(false);
                            }
                        }
                        this.camera = Some(handle);
                    }
                    Err(err) => this.log.push(format!("camera -> {err}")),
                }
                cx.notify();
            });
        })
        .detach();
    }

    fn take_photo(&mut self, cx: &mut Context<Self>) {
        let Some(handle) = &self.camera else {
            self.log.push("photo: start the camera first");
            cx.notify();
            return;
        };
        let id = handle.id;
        let task = cx.background_spawn(async move { camera::take_picture(&CameraHandle::from_id(id)) });
        cx.spawn(async move |this, cx| {
            let result = task.await;
            let _ = this.update(cx, |this, cx| {
                this.log.push(match result {
                    Ok(photo) => format!("photo {}x{} -> {}", photo.width, photo.height, photo.path),
                    Err(err) => format!("photo -> {err}"),
                });
                cx.notify();
            });
        })
        .detach();
    }

    fn stop_camera(&mut self, cx: &mut Context<Self>) {
        if let Some(handle) = self.camera.take() {
            self.log
                .push(format!("camera dispose -> {:?}", camera::dispose(handle)));
            cx.notify();
        }
    }

    fn views(&self) -> Vec<Arc<PlatformViewHandle>> {
        [
            self.map.as_ref().and_then(|m| m.platform_view_handle()),
            self.web.as_ref().and_then(|w| w.platform_view_handle()),
            self.camera
                .as_ref()
                .and_then(camera::preview_platform_view_handle),
        ]
        .into_iter()
        .flatten()
        .collect()
    }

    fn toggle_hidden(&mut self, cx: &mut Context<Self>) {
        self.hidden = !self.hidden;
        for view in self.views() {
            view.set_visible(!self.hidden);
        }
        self.log.push(if self.hidden {
            "native views hidden"
        } else {
            "native views shown"
        });
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

    fn view_box(
        handle: Option<Arc<PlatformViewHandle>>,
        height: f32,
        empty: &'static str,
        cx: &App,
    ) -> impl IntoElement {
        let frame = div()
            .w_full()
            .h(px(height))
            .flex_none()
            .rounded(cx.theme().radius)
            .bg(cx.theme().muted)
            .overflow_hidden();
        match handle {
            Some(handle) => frame.child(platform_view_element(handle).size_full()),
            None => frame
                .flex()
                .items_center()
                .justify_center()
                .child(ui::hint(empty, cx)),
        }
    }
}

impl Drop for NativeViewsScreen {
    fn drop(&mut self) {
        if let Some(handle) = self.camera.take() {
            let _ = camera::dispose(handle);
        }
    }
}

#[gpui_hot::hot]
impl Render for NativeViewsScreen {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let web_status = match &self.web_state {
            Some(state) => format!(
                "{}{} · {}% · back {} · forward {}",
                if state.loading { "loading · " } else { "" },
                if state.title.is_empty() {
                    &state.url
                } else {
                    &state.title
                },
                state.progress,
                state.can_go_back,
                state.can_go_forward
            ),
            None => "creating…".into(),
        };
        let camera_hint = if self.camera_busy {
            "Starting…"
        } else {
            "Camera off"
        };

        v_flex()
            .w_full()
            .p_3()
            .gap_3()
            .child(
                ui::section("Lifecycle", cx)
                    .child(ui::hint(
                        "Native views draw above GPUI. Recreate the Activity: the map, page \
                         and camera should come back in place with their state. Hide them as \
                         an app would under a GPUI dialog.",
                        cx,
                    ))
                    .child(
                        h_flex()
                            .flex_wrap()
                            .gap_1()
                            .child(Self::button(
                                "nv-recreate",
                                "Recreate Activity",
                                cx,
                                |this, cx| {
                                    #[cfg(target_os = "android")]
                                    let result = crate::host::recreate_activity();
                                    #[cfg(not(target_os = "android"))]
                                    let result: Result<(), String> = Err("Android only".into());
                                    this.log.push(format!("recreate -> {result:?}"));
                                    cx.notify();
                                },
                            ))
                            .child(Self::button(
                                "nv-hide",
                                if self.hidden {
                                    "Show native views"
                                } else {
                                    "Hide native views"
                                },
                                cx,
                                |this, cx| this.toggle_hidden(cx),
                            )),
                    ),
            )
            .child(
                ui::section("Map (OpenStreetMap tiles)", cx)
                    .child(Self::view_box(
                        self.map.as_ref().and_then(|m| m.platform_view_handle()),
                        MAP_HEIGHT,
                        "No map",
                        cx,
                    ))
                    .child(ui::value_row("Camera", self.map_status.clone(), cx))
                    .child(
                        h_flex()
                            .flex_wrap()
                            .gap_1()
                            .child(Self::button("nv-rome", "Rome", cx, |this, cx| {
                                this.map_call("set_center", cx, |m| m.set_center(ROME))
                            }))
                            .child(Self::button("nv-zoom-in", "Zoom +", cx, |this, cx| {
                                this.map_call("zoom in", cx, |m| {
                                    let zoom = m.camera()?.zoom;
                                    m.set_zoom(zoom + 1.)
                                })
                            }))
                            .child(Self::button("nv-zoom-out", "Zoom −", cx, |this, cx| {
                                this.map_call("zoom out", cx, |m| {
                                    let zoom = m.camera()?.zoom;
                                    m.set_zoom(zoom - 1.)
                                })
                            }))
                            .child(Self::button("nv-marker", "Pin center", cx, |this, cx| {
                                this.markers += 1;
                                let n = this.markers;
                                this.map_call("add_marker", cx, |m| {
                                    m.add_marker(&MapMarker {
                                        id: format!("pin-{n}"),
                                        position: m.camera()?.center,
                                        title: Some(format!("Pin {n}")),
                                        snippet: None,
                                    })
                                })
                            }))
                            .child(Self::button("nv-clear", "Clear pins", cx, |this, cx| {
                                this.map_call("clear_markers", cx, |m| m.clear_markers())
                            })),
                    ),
            )
            .child(
                ui::section("Web view", cx)
                    .child(
                        Input::new(&self.url)
                            .content_type(InputContentType::Url)
                            .cleanable(true),
                    )
                    .child(Self::view_box(
                        self.web.as_ref().and_then(|w| w.platform_view_handle()),
                        WEB_HEIGHT,
                        "No web view",
                        cx,
                    ))
                    .child(ui::value_row("Page", web_status, cx))
                    .child(
                        h_flex()
                            .flex_wrap()
                            .gap_1()
                            .child(Self::button("nv-go", "Go", cx, |this, cx| {
                                let url = this.url.read(cx).value().to_string();
                                this.web_call("navigate", cx, |w| webview::navigate(w, &url))
                            }))
                            .child(Self::button("nv-local", "Local page", cx, |this, cx| {
                                this.web_call("navigate_to_html", cx, |w| {
                                    webview::navigate_to_html(w, LOCAL_PAGE)
                                })
                            }))
                            .child(Self::button("nv-back", "Back", cx, |this, cx| {
                                this.web_call("go_back", cx, webview::go_back)
                            }))
                            .child(Self::button("nv-forward", "Forward", cx, |this, cx| {
                                this.web_call("go_forward", cx, webview::go_forward)
                            }))
                            .child(Self::button("nv-reload", "Reload", cx, |this, cx| {
                                this.web_call("reload", cx, webview::reload)
                            }))
                            .child(Self::button("nv-js", "Run JS", cx, |this, cx| this.run_js(cx))),
                    ),
            )
            .child(
                ui::section("Camera preview", cx)
                    .child(Self::view_box(
                        self.camera
                            .as_ref()
                            .and_then(camera::preview_platform_view_handle),
                        CAMERA_HEIGHT,
                        camera_hint,
                        cx,
                    ))
                    .child(
                        h_flex()
                            .flex_wrap()
                            .gap_1()
                            .child(Self::button("nv-cam", "Start camera", cx, |this, cx| {
                                this.start_camera(cx)
                            }))
                            .child(Self::button("nv-photo", "Take photo", cx, |this, cx| {
                                this.take_photo(cx)
                            }))
                            .child(Self::button("nv-cam-stop", "Stop camera", cx, |this, cx| {
                                this.stop_camera(cx)
                            })),
                    ),
            )
            .child(self.log.render(cx))
    }
}
