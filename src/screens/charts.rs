use std::time::Duration;

use gpui::{
    AnyElement, App, Bounds, Context, ElementId, Hsla, IntoElement, ParentElement, Pixels, Point,
    Render, SharedString, Styled, Task, Window, div, fill, point, px, size,
};
use gpui_kit::component::{
    button::Button,
    chart::{AreaChart, BarChart, CandlestickChart, LineChart, PieChart, RadarChart, SankeyChart},
    h_flex,
    plot::{
        Plot, PlotElement, TooltipState,
        scale::{Scale, ScaleBand, ScaleLinear},
        shape::SankeyLink,
    },
    v_flex,
};

use crate::ui::{self, EventLog, prelude::*};

const MONTHS: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

#[derive(Clone)]
struct Metric {
    month: SharedString,
    revenue: f64,
    users: f64,
    mobile: f64,
}

#[derive(Clone)]
struct Candle {
    day: SharedString,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
}

#[derive(Clone)]
struct Share {
    name: &'static str,
    value: f32,
}

#[derive(Clone)]
struct Score {
    dimension: &'static str,
    alpha: f64,
    beta: f64,
}

#[derive(Clone)]
struct Node {
    name: &'static str,
    color: Hsla,
}

/// Deterministic series: the same numbers every launch, shifted by `tick` for live updates.
fn metrics(tick: usize) -> Vec<Metric> {
    (0..12)
        .map(|i| {
            let t = (i + tick) as f64;
            Metric {
                month: MONTHS[(i + tick) % 12].into(),
                revenue: 40.0 + 25.0 * (t * 0.7).sin() + 3.0 * t % 17.0 - 8.0,
                users: 120.0 + 60.0 * (t * 0.45).cos() + t * 2.0 % 13.0,
                mobile: 80.0 + 40.0 * (t * 0.6).sin().abs(),
            }
        })
        .collect()
}

fn candles(tick: usize) -> Vec<Candle> {
    let mut price = 100.0;
    (0..20)
        .map(|i| {
            let t = (i + tick) as f64;
            let open = price;
            let close = open + 6.0 * (t * 1.3).sin();
            let high = open.max(close) + 2.0 + (t * 0.7).cos().abs() * 3.0;
            let low = open.min(close) - 2.0 - (t * 0.9).sin().abs() * 3.0;
            price = close;
            Candle {
                day: format!("D{}", i + 1).into(),
                open,
                high,
                low,
                close,
            }
        })
        .collect()
}

fn shares() -> Vec<Share> {
    vec![
        Share {
            name: "Chrome",
            value: 62.0,
        },
        Share {
            name: "Safari",
            value: 19.0,
        },
        Share {
            name: "Edge",
            value: 6.0,
        },
        Share {
            name: "Firefox",
            value: 4.0,
        },
        Share {
            name: "Other",
            value: 9.0,
        },
    ]
}

fn scores() -> Vec<Score> {
    vec![
        Score {
            dimension: "Speed",
            alpha: 80.,
            beta: 60.,
        },
        Score {
            dimension: "Stability",
            alpha: 65.,
            beta: 90.,
        },
        Score {
            dimension: "Design",
            alpha: 90.,
            beta: 70.,
        },
        Score {
            dimension: "Docs",
            alpha: 55.,
            beta: 75.,
        },
        Score {
            dimension: "Mobile",
            alpha: 40.,
            beta: 50.,
        },
    ]
}

/// A minimal custom plot on the v0.7.0 primitives (PlotElement + scales + tooltip hooks).
struct Histogram {
    values: Vec<f64>,
    color: Hsla,
    muted: Hsla,
}

impl IntoElement for Histogram {
    type Element = PlotElement<Self>;
    fn into_element(self) -> Self::Element {
        PlotElement::new(self)
    }
}

impl Histogram {
    fn scales(&self, bounds: &Bounds<Pixels>) -> (ScaleBand<usize>, ScaleLinear<f64>) {
        let width = bounds.size.width.as_f32();
        let height = bounds.size.height.as_f32();
        let x = ScaleBand::new(0..self.values.len(), [0., width]).max_band_width(24.);
        let y = ScaleLinear::new(self.values.iter().copied().chain([0.0]), [height, 0.]);
        (x, y)
    }
}

impl Plot for Histogram {
    fn paint(&mut self, bounds: Bounds<Pixels>, window: &mut Window, _: &mut App) {
        let (x, y) = self.scales(&bounds);
        let band = x.band_width();
        for (ix, value) in self.values.iter().enumerate() {
            let (Some(left), Some(top)) = (x.tick(&ix), y.tick(value)) else {
                continue;
            };
            let origin = bounds.origin + point(px(left), px(top));
            let bar = Bounds::new(origin, size(px(band), bounds.size.height - px(top)));
            window.paint_quad(fill(bar, if ix % 2 == 0 { self.color } else { self.muted }));
        }
    }

    fn id(&self) -> Option<ElementId> {
        Some("custom-histogram".into())
    }

    fn tooltip_state(
        &self,
        position: Point<Pixels>,
        bounds: Bounds<Pixels>,
        _: &App,
    ) -> Option<TooltipState> {
        let (x, y) = self.scales(&bounds);
        let ix = x
            .nearest_index(position.x.as_f32())
            .min(self.values.len().saturating_sub(1));
        let left = x.tick(&ix)? + x.band_width() / 2.;
        let top = y.tick(&self.values[ix])?;
        Some(TooltipState::new(
            ix,
            point(px(left), px(0.)),
            vec![point(px(left), px(top))],
        ))
    }

    fn tooltip(
        &self,
        state: &TooltipState,
        _: Point<Pixels>,
        _: Bounds<Pixels>,
        _: &mut Window,
        _: &mut App,
    ) -> Option<AnyElement> {
        let value = self.values.get(state.index)?;
        Some(
            div()
                .px_2()
                .py_1()
                .rounded_md()
                .bg(gpui::black().opacity(0.8))
                .text_color(gpui::white())
                .text_xs()
                .child(format!("bin {}: {value:.1}", state.index))
                .into_any_element(),
        )
    }
}

pub struct ChartsScreen {
    tick: usize,
    live: Option<Task<()>>,
    narrow: bool,
    log: EventLog,
}

impl ChartsScreen {
    pub fn new(_: &mut Window, _: &mut Context<Self>) -> Self {
        Self {
            tick: 0,
            live: None,
            narrow: false,
            log: EventLog::default(),
        }
    }

    fn toggle_live(&mut self, cx: &mut Context<Self>) {
        if self.live.take().is_some() {
            self.log.push("live updates stopped");
        } else {
            self.log.push("live updates started (2 Hz)");
            self.live = Some(cx.spawn(async move |this, cx| {
                loop {
                    cx.background_executor()
                        .timer(Duration::from_millis(500))
                        .await;
                    if this
                        .update(cx, |this, cx| {
                            this.tick += 1;
                            cx.notify();
                        })
                        .is_err()
                    {
                        break;
                    }
                }
            }));
        }
        cx.notify();
    }
}

fn money(value: f64) -> SharedString {
    format!("${value:.0}k").into()
}

impl Render for ChartsScreen {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let accent = cx.theme().primary;
        let deep = cx.theme().success;
        let danger = cx.theme().danger;
        let data = metrics(self.tick);
        let width: gpui::DefiniteLength = if self.narrow {
            px(220.).into()
        } else {
            gpui::relative(1.)
        };
        let chart_box = |title: &'static str, cx: &mut Context<Self>, chart: AnyElement| {
            ui::section(title, cx).child(div().w(width).h(px(200.)).child(chart))
        };
        let palette = [accent, deep, danger, cx.theme().warning, cx.theme().info];
        let nodes = vec![
            Node {
                name: "Revenue",
                color: accent,
            },
            Node {
                name: "Mobile",
                color: deep,
            },
            Node {
                name: "Desktop",
                color: cx.theme().info,
            },
            Node {
                name: "Costs",
                color: danger,
            },
            Node {
                name: "Profit",
                color: cx.theme().warning,
            },
        ];
        let links = vec![
            SankeyLink::new(1, 0, 60.),
            SankeyLink::new(2, 0, 40.),
            SankeyLink::new(0, 3, 70.),
            SankeyLink::new(0, 4, 30.),
        ];
        let histogram: Vec<f64> = (0..16)
            .map(|i| 5.0 + ((i * 7 + self.tick * 3) % 11) as f64 * 3.0)
            .collect();

        v_flex()
            .gap_3()
            .child(
                ui::section("Controls", cx)
                    .child(
                        h_flex()
                            .flex_wrap()
                            .gap_1()
                            .child(
                                Button::new("live")
                                    .small()
                                    .outline()
                                    .selected(self.live.is_some())
                                    .label("Live data")
                                    .on_click(cx.listener(|this, _, _, cx| this.toggle_live(cx))),
                            )
                            .child(
                                Button::new("narrow")
                                    .small()
                                    .outline()
                                    .selected(self.narrow)
                                    .label("Narrow (220 pt)")
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.narrow = !this.narrow;
                                        this.log.push(format!("narrow -> {}", this.narrow));
                                        cx.notify();
                                    })),
                            )
                            .child(
                                Button::new("step")
                                    .small()
                                    .outline()
                                    .label("Step data")
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.tick += 1;
                                        cx.notify();
                                    })),
                            ),
                    )
                    .child(ui::value_row("Data tick", self.tick.to_string(), cx))
                    .child(ui::hint(
                        "On Android, chart tooltips open on long press (not hover); drag to move \
                         the crosshair, lift to close.",
                        cx,
                    )),
            )
            .child(chart_box(
                "LineChart",
                cx,
                LineChart::new(data.clone())
                    .x(|d| d.month.clone())
                    .y(|d| d.revenue)
                    .stroke(accent)
                    .name("Revenue")
                    .dot()
                    .y_axis(true)
                    .y_tick_format(money)
                    .tooltip_value(|_, value| money(value))
                    .id("line-chart")
                    .into_any_element(),
            ))
            .child(chart_box(
                "BarChart",
                cx,
                BarChart::new(data.clone())
                    .band(|d| d.month.clone())
                    .value(|d| d.revenue - 30.0)
                    .name("Profit")
                    .fill(move |d, _, _, _| -> gpui::Background {
                        if d.revenue >= 30.0 {
                            accent.into()
                        } else {
                            danger.into()
                        }
                    })
                    .value_axis(true)
                    .value_tick_format(money)
                    .max_band_width(px(24.))
                    .id("bar-chart")
                    .into_any_element(),
            ))
            .child(chart_box(
                "AreaChart (two series)",
                cx,
                AreaChart::new(data.clone())
                    .x(|d| d.month.clone())
                    .y(|d| d.users)
                    .stroke(accent)
                    .fill(accent.opacity(0.3))
                    .name("Users")
                    .y(|d| d.mobile)
                    .stroke(deep)
                    .fill(deep.opacity(0.3))
                    .name("Mobile")
                    .id("area-chart")
                    .into_any_element(),
            ))
            .child(chart_box(
                "PieChart (donut)",
                cx,
                PieChart::new(shares())
                    .value(|d| d.value)
                    .inner_radius(50.)
                    .outer_radius(85.)
                    .pad_angle(0.02)
                    .color(move |d| {
                        palette[shares().iter().position(|s| s.name == d.name).unwrap_or(0)
                            % palette.len()]
                    })
                    .label(|d| d.name.into())
                    .tooltip_name(|d| d.name.into())
                    .tooltip_value(|_, value, percent| {
                        format!("{value:.0} ({:.0}%)", percent * 100.).into()
                    })
                    .name("Browsers")
                    .id("pie-chart")
                    .into_any_element(),
            ))
            .child(chart_box(
                "RadarChart (two series)",
                cx,
                RadarChart::new(scores())
                    .label(|d| d.dimension)
                    .value(|d| d.alpha)
                    .stroke(accent)
                    .fill(accent.opacity(0.25))
                    .name("Alpha")
                    .value(|d| d.beta)
                    .stroke(deep)
                    .fill(deep.opacity(0.25))
                    .name("Beta")
                    .max_value(100.)
                    .id("radar-chart")
                    .into_any_element(),
            ))
            .child(chart_box(
                "CandlestickChart",
                cx,
                CandlestickChart::new(candles(self.tick))
                    .x(|d| d.day.clone())
                    .open(|d| d.open)
                    .high(|d| d.high)
                    .low(|d| d.low)
                    .close(|d| d.close)
                    .tooltip_value(|_, _, value| format!("${value:.2}").into())
                    .id("candlestick-chart")
                    .into_any_element(),
            ))
            .child(chart_box(
                "SankeyChart",
                cx,
                SankeyChart::new(nodes, links)
                    .node_color(|d: &Node| d.color)
                    .node_label(|d: &Node| d.name.into())
                    .tooltip_name(|d: &Node| d.name.into())
                    .tooltip_value(|_, value| money(value))
                    .id("sankey-chart")
                    .into_any_element(),
            ))
            .child(chart_box(
                "Custom Plot (Plot trait + ScaleBand/ScaleLinear)",
                cx,
                Histogram {
                    values: histogram,
                    color: accent,
                    muted: accent.opacity(0.5),
                }
                .into_any_element(),
            ))
            .child(self.log.render(cx))
    }
}
