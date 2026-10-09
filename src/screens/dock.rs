use gpui::{
    App, AppContext as _, Context, Entity, EventEmitter, FocusHandle, Focusable, IntoElement,
    ParentElement, Styled, Window, div, prelude::*, px,
};
use gpui_kit::component::{
    dock::{
        BasePanel, DockArea, DockLayout, DockPlacement, DockSkin, Panel, PanelEvent, panel_handle,
    },
    v_flex,
};

use crate::ui::{self, prelude::*};

struct DemoPanel {
    name: &'static str,
    title: &'static str,
    focus: FocusHandle,
    taps: usize,
}

impl DemoPanel {
    fn new(name: &'static str, title: &'static str, cx: &mut App) -> Entity<Self> {
        cx.new(|cx| Self {
            name,
            title,
            focus: cx.focus_handle(),
            taps: 0,
        })
    }
}

impl EventEmitter<PanelEvent> for DemoPanel {}

impl Focusable for DemoPanel {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus.clone()
    }
}

impl BasePanel for DemoPanel {
    fn panel_name(&self) -> &'static str {
        self.name
    }
}

impl Panel for DemoPanel {
    fn title(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        self.title
    }
}

crate::hot_render!(DemoPanel);

impl DemoPanel {
    fn render_view(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .id(self.name)
            .size_full()
            .p_2()
            .gap_1()
            .text_sm()
            .on_click(cx.listener(|this, _, _, cx| {
                this.taps += 1;
                log::info!("dock panel {} tapped ({})", this.name, this.taps);
                cx.notify();
            }))
            .child(div().font_semibold().child(self.title))
            .child(format!("Taps: {}", self.taps))
            .child("Tab drag/split is a pointer-drag interaction.")
    }
}

pub struct DockScreen {
    area: Entity<DockArea>,
}

impl DockScreen {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let explorer = DemoPanel::new("explorer", "Explorer", cx);
        let search = DemoPanel::new("search", "Search", cx);
        let editor = DemoPanel::new("editor", "Editor", cx);
        let terminal = DemoPanel::new("terminal", "Terminal", cx);
        let (area, skin) = DockSkin::dock_area("lab-dock", Some(1), window, cx);
        area.update(cx, |area, cx| {
            area.set_center(
                DockLayout::h_split()
                    .child(
                        DockLayout::tabs()
                            .panel_view(panel_handle(explorer), cx)
                            .panel_view(panel_handle(search), cx),
                        Some(px(150.)),
                    )
                    .child(
                        DockLayout::tabs().panel_view(panel_handle(editor), cx),
                        None,
                    ),
                window,
                cx,
            );
            area.set_dock(
                DockPlacement::Bottom,
                DockLayout::tabs().panel_view(panel_handle(terminal), cx),
                window,
                cx,
            );
            area.set_dock_size(DockPlacement::Bottom, px(140.), window, cx);
            area.set_dock_collapsible(DockPlacement::Bottom, true, window, cx);
        });
        skin.set_toggle_button_visible(true, cx);
        Self { area }
    }
}

crate::hot_render!(DockScreen);

impl DockScreen {
    fn render_view(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .size_full()
            .p_2()
            .gap_2()
            .child(ui::hint(
                "Desktop docking squeezed to phone width. Taps and tab switches should work; \
                 dragging tabs to split panels relies on mouse drag.",
                cx,
            ))
            .child(
                div()
                    .flex_1()
                    .min_h_0()
                    .border_1()
                    .border_color(cx.theme().border)
                    .child(self.area.clone()),
            )
    }
}
