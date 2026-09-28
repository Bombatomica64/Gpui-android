use gpui::{
    AppContext as _, Context, Entity, IntoElement, ParentElement, Render, SharedString, Styled,
    Subscription, Window, div, prelude::*, px,
};
use gpui_kit::component::{
    Size,
    button::Button,
    h_flex,
    tab::{Tab, TabBar},
    table::{
        Column, ColumnSort, DataTable, Table, TableBody, TableCaption, TableCell, TableDelegate,
        TableEvent, TableHead, TableHeader, TableRow, TableState,
    },
    tag::Tag,
    v_flex,
};

use crate::ui::{self, EventLog, prelude::*};

const ROWS: usize = 1_000;

#[derive(Clone)]
struct Stock {
    symbol: String,
    price: f64,
    change: f64,
    volume: u64,
}

/// Deterministic pseudo-random data (no RNG dependency, same on every run).
fn stocks() -> Vec<Stock> {
    let mut seed: u64 = 0x2545_f491_4f6c_dd1d;
    let mut next = move || {
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        seed
    };
    (0..ROWS)
        .map(|i| {
            let letters: String = (0..3).map(|k| (b'A' + ((i * 7 + k * 11) % 26) as u8) as char).collect();
            Stock {
                symbol: format!("{letters}{i:03}"),
                price: 10.0 + (next() % 50_000) as f64 / 100.0,
                change: ((next() % 2_000) as f64 - 1_000.0) / 100.0,
                volume: next() % 5_000_000,
            }
        })
        .collect()
}

struct StockDelegate {
    rows: Vec<Stock>,
    columns: Vec<Column>,
}

impl StockDelegate {
    fn new() -> Self {
        Self {
            rows: stocks(),
            columns: vec![
                Column::new("#", "#").width(px(56.)).fixed_left(),
                Column::new("symbol", "Symbol").width(px(100.)).sortable(),
                Column::new("price", "Price").width(px(96.)).text_right().sortable(),
                Column::new("change", "Change %").width(px(96.)).text_right().sortable(),
                Column::new("volume", "Volume").width(px(120.)).text_right().sortable(),
            ],
        }
    }
}

impl TableDelegate for StockDelegate {
    fn columns_count(&self, _: &gpui::App) -> usize {
        self.columns.len()
    }

    fn rows_count(&self, _: &gpui::App) -> usize {
        self.rows.len()
    }

    fn column(&self, col_ix: usize, _: &gpui::App) -> Column {
        self.columns[col_ix].clone()
    }

    fn render_td(
        &mut self,
        row_ix: usize,
        col_ix: usize,
        _: &mut Window,
        cx: &mut Context<TableState<Self>>,
    ) -> impl IntoElement {
        let row = &self.rows[row_ix];
        match col_ix {
            0 => div().child(row_ix.to_string()),
            1 => div().child(row.symbol.clone()),
            2 => div().child(format!("{:.2}", row.price)),
            3 => div()
                .text_color(if row.change >= 0.0 { cx.theme().success } else { cx.theme().danger })
                .child(format!("{:+.2}", row.change)),
            _ => div().child(row.volume.to_string()),
        }
    }

    fn perform_sort(&mut self, col_ix: usize, sort: ColumnSort, _: &mut Window, _: &mut Context<TableState<Self>>) {
        let descending = matches!(sort, ColumnSort::Descending);
        self.rows.sort_by(|a, b| {
            let ordering = match col_ix {
                1 => a.symbol.cmp(&b.symbol),
                2 => a.price.total_cmp(&b.price),
                3 => a.change.total_cmp(&b.change),
                _ => a.volume.cmp(&b.volume),
            };
            if descending { ordering.reverse() } else { ordering }
        });
    }
}

pub struct TablesScreen {
    view: usize,
    table: Entity<TableState<StockDelegate>>,
    stripe: bool,
    size: Size,
    log: EventLog,
    _subscriptions: Vec<Subscription>,
}

impl TablesScreen {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let table = cx.new(|cx| TableState::new(StockDelegate::new(), window, cx));
        let subscription = cx.subscribe(&table, |this, _, event: &TableEvent, cx| {
            let message = match event {
                TableEvent::SelectRow(row) => format!("select row {row}"),
                TableEvent::DoubleClickedRow(row) => format!("double-tap row {row}"),
                TableEvent::SelectColumn(col) => format!("select column {col}"),
                TableEvent::SelectCell(row, col) => format!("select cell {row},{col}"),
                TableEvent::ColumnWidthsChanged(_) => "column widths changed".into(),
                TableEvent::MoveColumn(from, to) => format!("move column {from} -> {to}"),
                TableEvent::ClearSelection => "clear selection".into(),
                TableEvent::DoubleClickedCell(row, col) => format!("double-tap cell {row},{col}"),
                TableEvent::RightClickedRow(row) => format!("right-click row {row:?}"),
                TableEvent::RightClickedCell(row, col) => format!("right-click cell {row},{col}"),
            };
            this.log.push(message);
            cx.notify();
        });
        Self {
            view: 1,
            table,
            stripe: true,
            size: Size::Medium,
            log: EventLog::default(),
            _subscriptions: vec![subscription],
        }
    }

    fn render_static(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let invoices = [
            ("INV001", "Paid", "Card", "$250.00"),
            ("INV002", "Pending", "PayPal", "$150.00"),
            ("INV003", "Unpaid", "Transfer", "$350.00"),
            ("INV004", "Paid", "Card", "$450.00"),
            ("INV005", "Paid", "PayPal", "$550.00"),
        ];
        v_flex()
            .p_3()
            .gap_2()
            .child(
                Table::new()
                    .with_size(self.size)
                    .child(
                        TableHeader::new().child(
                            TableRow::new()
                                .child(TableHead::new().w(px(80.)).child("Invoice"))
                                .child(TableHead::new().child("Status"))
                                .child(TableHead::new().child("Method"))
                                .child(TableHead::new().text_right().child("Amount")),
                        ),
                    )
                    .child(TableBody::new().children(invoices.iter().map(|(id, status, method, amount)| {
                        let tag = match *status {
                            "Paid" => Tag::success(),
                            "Pending" => Tag::warning(),
                            _ => Tag::danger(),
                        };
                        TableRow::new()
                            .child(TableCell::new().w(px(80.)).child(*id))
                            .child(TableCell::new().child(tag.small().child(*status)))
                            .child(TableCell::new().child(*method))
                            .child(TableCell::new().text_right().child(*amount))
                    })))
                    .child(TableCaption::new().child("Five invoices on a 411 pt wide screen.")),
            )
            .child(ui::hint("Stateless Table: check column squeezing and wrapping.", cx))
    }
}

impl Render for TablesScreen {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let (selected_row, selection) = {
            let state = self.table.read(cx);
            (state.selected_row(), format!("{:?}", state.selection()))
        };
        let body = if self.view == 0 {
            self.render_static(cx).into_any_element()
        } else {
            v_flex()
                .flex_1()
                .min_h_0()
                .p_2()
                .gap_2()
                .child(
                    h_flex()
                        .flex_wrap()
                        .gap_1()
                        .child(Button::new("stripe").small().outline().selected(self.stripe).label("Stripe").on_click(
                            cx.listener(|this, _, _, cx| {
                                this.stripe = !this.stripe;
                                cx.notify();
                            }),
                        ))
                        .child(Button::new("size").small().outline().label(format!("Size: {:?}", self.size)).on_click(
                            cx.listener(|this, _, _, cx| {
                                this.size = match this.size {
                                    Size::Small => Size::Medium,
                                    Size::Medium => Size::Large,
                                    _ => Size::Small,
                                };
                                cx.notify();
                            }),
                        ))
                        .child(Button::new("jump").small().outline().label("Scroll to row 900").on_click(cx.listener(
                            |this, _, _, cx| {
                                this.table.update(cx, |state, cx| state.scroll_to_row(900, cx));
                                this.log.push("scroll_to_row(900)");
                                cx.notify();
                            },
                        ))),
                )
                .child(ui::value_row(
                    "Selection",
                    format!("{selection} (row {selected_row:?}) — {ROWS} rows, virtualized"),
                    cx,
                ))
                .child(
                    div()
                        .flex_1()
                        .min_h_0()
                        .child(DataTable::new(&self.table).stripe(self.stripe).with_size(self.size)),
                )
                .child(self.log.render(cx))
                .into_any_element()
        };
        v_flex()
            .size_full()
            .child(
                TabBar::new("table-kind")
                    .underline()
                    .selected_index(self.view)
                    .child(Tab::new().label("Table"))
                    .child(Tab::new().label(SharedString::from(format!("DataTable ({ROWS})"))))
                    .on_click(cx.listener(|this, ix: &usize, _, cx| {
                        this.view = *ix;
                        cx.notify();
                    })),
            )
            .child(body)
    }
}
