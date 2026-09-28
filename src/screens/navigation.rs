use gpui::{
    AppContext as _, Context, Entity, IntoElement, ParentElement, Render, SharedString, Styled,
    Subscription, Window, div, prelude::*, px,
};
use gpui_kit::component::{
    IconName,
    breadcrumb::{Breadcrumb, BreadcrumbItem},
    button::Button,
    carousel::{
        Carousel, CarouselContent, CarouselEvent, CarouselItem, CarouselNext, CarouselPrevious,
        CarouselState,
    },
    h_flex,
    pagination::Pagination,
    sidebar::{Sidebar, SidebarGroup, SidebarHeader, SidebarMenu, SidebarMenuItem},
    tab::{Tab, TabBar},
    v_flex,
};

use crate::ui::{self, EventLog, prelude::*};

const PATH: &[&str] = &["Home", "Projects", "gpui-kit", "crates", "component", "src"];

pub struct NavigationScreen {
    tab: usize,
    tab_pill: usize,
    tab_segmented: usize,
    tab_underline: usize,
    many_tabs: usize,
    depth: usize,
    page: usize,
    carousel: Entity<CarouselState>,
    slide: usize,
    sidebar_item: SharedString,
    sidebar_collapsed: bool,
    log: EventLog,
    _subscriptions: Vec<Subscription>,
}

impl NavigationScreen {
    pub fn new(_: &mut Window, cx: &mut Context<Self>) -> Self {
        let carousel = cx.new(|_| CarouselState::new(5).with_looping(true));
        let subscription = cx.subscribe(&carousel, |this, _, event: &CarouselEvent, cx| {
            let CarouselEvent::Change(index) = event;
            this.slide = *index;
            this.log.push(format!("carousel -> slide {index}"));
            cx.notify();
        });
        Self {
            tab: 0,
            tab_pill: 1,
            tab_segmented: 0,
            tab_underline: 2,
            many_tabs: 0,
            depth: PATH.len(),
            page: 3,
            carousel,
            slide: 0,
            sidebar_item: "Inbox".into(),
            sidebar_collapsed: false,
            log: EventLog::default(),
            _subscriptions: vec![subscription],
        }
    }

    fn tab_bar(
        &self,
        id: &'static str,
        selected: usize,
        variant: usize,
        cx: &mut Context<Self>,
        set: fn(&mut Self, usize),
    ) -> TabBar {
        let bar = TabBar::new(id)
            .selected_index(selected)
            .child(Tab::new().label("Account"))
            .child(Tab::new().label("Password"))
            .child(Tab::new().label("Billing"))
            .on_click(cx.listener(move |this, ix: &usize, _, cx| {
                set(this, *ix);
                this.log.push(format!("{id} -> {ix}"));
                cx.notify();
            }));
        match variant {
            1 => bar.pill(),
            2 => bar.segmented(),
            3 => bar.underline(),
            _ => bar,
        }
    }
}

impl Render for NavigationScreen {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let crumbs = PATH[..self.depth].iter().enumerate().map(|(ix, name)| {
            BreadcrumbItem::new(*name).on_click(cx.listener(move |this, _, _, cx| {
                this.depth = ix + 1;
                this.log.push(format!("breadcrumb -> {name}"));
                cx.notify();
            }))
        });
        let crumbs: Vec<_> = crumbs.collect();

        let sidebar_items = [
            ("Inbox", IconName::Inbox),
            ("Calendar", IconName::Calendar),
            ("Search", IconName::Search),
            ("Settings", IconName::Settings),
        ]
        .into_iter()
        .map(|(label, icon)| {
            SidebarMenuItem::new(label)
                .icon(icon)
                .active(self.sidebar_item.as_ref() == label)
                .on_click(cx.listener(move |this, _, _, cx| {
                    this.sidebar_item = label.into();
                    this.log.push(format!("sidebar -> {label}"));
                    cx.notify();
                }))
        });
        let sidebar_items: Vec<_> = sidebar_items.collect();

        let slides = ["Sunrise", "Forest", "Ocean", "Desert", "Night"];
        let slide_colors = [0xf59e0b, 0x16a34a, 0x2563eb, 0xd97706, 0x4338ca];

        v_flex()
            .gap_3()
            .child(
                ui::section("Tabs", cx)
                    .child(self.tab_bar("tabs-default", self.tab, 0, cx, |this, ix| this.tab = ix))
                    .child(self.tab_bar("tabs-pill", self.tab_pill, 1, cx, |this, ix| this.tab_pill = ix))
                    .child(self.tab_bar("tabs-segmented", self.tab_segmented, 2, cx, |this, ix| this.tab_segmented = ix))
                    .child(self.tab_bar("tabs-underline", self.tab_underline, 3, cx, |this, ix| this.tab_underline = ix))
                    .child(
                        TabBar::new("many-tabs")
                            .underline()
                            .selected_index(self.many_tabs)
                            .children((1..=12).map(|i| Tab::new().label(format!("Tab {i}"))))
                            .on_click(cx.listener(|this, ix: &usize, _, cx| {
                                this.many_tabs = *ix;
                                this.log.push(format!("many tabs -> {ix}"));
                                cx.notify();
                            })),
                    )
                    .child(ui::value_row(
                        "Selected",
                        format!(
                            "{} / {} / {} / {} / many {}",
                            self.tab, self.tab_pill, self.tab_segmented, self.tab_underline, self.many_tabs
                        ),
                        cx,
                    ))
                    .child(ui::hint("12 tabs overflow a phone width: swipe the bar horizontally.", cx)),
            )
            .child(
                ui::section("Breadcrumb", cx)
                    .child(Breadcrumb::new().children(crumbs))
                    .child(
                        Button::new("reset-crumbs").small().outline().label("Reset path").on_click(cx.listener(
                            |this, _, _, cx| {
                                this.depth = PATH.len();
                                this.log.push("breadcrumb reset");
                                cx.notify();
                            },
                        )),
                    )
                    .child(ui::hint("Six levels wrap on a narrow screen. Tap a level to go up.", cx)),
            )
            .child(
                ui::section("Pagination", cx)
                    .child(
                        Pagination::new("pages")
                            .current_page(self.page)
                            .total_pages(20)
                            .on_click(cx.listener(|this, page: &usize, _, cx| {
                                this.page = *page;
                                this.log.push(format!("page -> {page}"));
                                cx.notify();
                            })),
                    )
                    .child(
                        Pagination::new("pages-compact")
                            .compact()
                            .current_page(self.page)
                            .total_pages(20)
                            .on_click(cx.listener(|this, page: &usize, _, cx| {
                                this.page = *page;
                                this.log.push(format!("compact page -> {page}"));
                                cx.notify();
                            })),
                    )
                    .child(ui::value_row("Page", format!("{} / 20", self.page), cx)),
            )
            .child(
                ui::section("Carousel", cx)
                    .child(
                        Carousel::new("carousel", &self.carousel)
                            .w_full()
                            .child(
                                CarouselContent::new(&self.carousel).children(slides.iter().enumerate().map(
                                    |(ix, name)| {
                                        CarouselItem::new(("slide", ix), ix, &self.carousel).child(
                                            div()
                                                .h(px(140.))
                                                .flex()
                                                .items_center()
                                                .justify_center()
                                                .rounded(cx.theme().radius_lg)
                                                .bg(gpui::rgb(slide_colors[ix]))
                                                .text_color(gpui::white())
                                                .text_xl()
                                                .child(*name),
                                        )
                                    },
                                )),
                            )
                            .child(h_flex().justify_between().child(CarouselPrevious::new(&self.carousel)).child(CarouselNext::new(&self.carousel))),
                    )
                    .child(ui::value_row("Slide", format!("{} ({})", self.slide, slides[self.slide.min(4)]), cx))
                    .child(ui::hint("Swipe horizontally; a vertical swipe must still scroll the page.", cx)),
            )
            .child(
                ui::section("Sidebar", cx)
                    .child(
                        h_flex()
                            .h(px(260.))
                            .w_full()
                            .border_1()
                            .border_color(cx.theme().border)
                            .rounded(cx.theme().radius)
                            .overflow_hidden()
                            .child(
                                Sidebar::new("sidebar")
                                    .collapsed(self.sidebar_collapsed)
                                    .header(SidebarHeader::new().child("Mail"))
                                    .child(SidebarGroup::new("Navigation").child(SidebarMenu::new().children(sidebar_items))),
                            )
                            .child(
                                v_flex()
                                    .flex_1()
                                    .p_3()
                                    .gap_2()
                                    .child(div().font_semibold().child(self.sidebar_item.clone()))
                                    .child(
                                        Button::new("collapse-sidebar")
                                            .small()
                                            .outline()
                                            .label(if self.sidebar_collapsed { "Expand" } else { "Collapse" })
                                            .on_click(cx.listener(|this, _, _, cx| {
                                                this.sidebar_collapsed = !this.sidebar_collapsed;
                                                this.log.push(format!("sidebar collapsed -> {}", this.sidebar_collapsed));
                                                cx.notify();
                                            })),
                                    ),
                            ),
                    )
                    .child(ui::hint(
                        "A desktop sidebar squeezed into a phone: check it collapses to icons.",
                        cx,
                    )),
            )
            .child(self.log.render(cx))
    }
}
