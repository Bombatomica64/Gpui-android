use std::rc::Rc;

use gpui::{
    AppContext as _, Context, Entity, IntoElement, ParentElement, Pixels, SharedString, Size,
    Styled, Subscription, Task, Window, div, px, size,
};
use gpui_kit::component::{
    Icon, IconName, IndexPath, VirtualListScrollHandle, h_flex,
    list::{List, ListDelegate, ListEvent, ListItem, ListState},
    tab::{Tab, TabBar},
    tree::{TreeEvent, TreeItem, TreeState, tree},
    v_flex, v_virtual_list,
};

use crate::ui::{self, EventLog, prelude::*};

const CONTACTS: usize = 1_000;
const FIRST: &[&str] = &[
    "Ada", "Bruno", "Chiara", "Dmitri", "Emi", "Farah", "Giulia", "Hiro", "Ines", "Jae",
];
const LAST: &[&str] = &[
    "Rossi", "Tanaka", "Kim", "Müller", "García", "Nguyen", "Okafor", "Silva", "Novak", "Li",
];

fn contact(ix: usize) -> SharedString {
    format!(
        "{} {} #{ix:04}",
        FIRST[ix % FIRST.len()],
        LAST[(ix / 7) % LAST.len()]
    )
    .into()
}

struct ContactDelegate {
    all: Vec<SharedString>,
    matches: Vec<usize>,
    selected: Option<IndexPath>,
}

impl ListDelegate for ContactDelegate {
    type Item = ListItem;

    fn perform_search(
        &mut self,
        query: &str,
        _: &mut Window,
        cx: &mut Context<ListState<Self>>,
    ) -> Task<()> {
        let query = query.to_lowercase();
        self.matches = (0..self.all.len())
            .filter(|ix| query.is_empty() || self.all[*ix].to_lowercase().contains(&query))
            .collect();
        cx.notify();
        Task::ready(())
    }

    fn items_count(&self, _: usize, _: &gpui::App) -> usize {
        self.matches.len()
    }

    fn render_item(
        &mut self,
        ix: IndexPath,
        _: &mut Window,
        cx: &mut Context<ListState<Self>>,
    ) -> Option<ListItem> {
        let contact_ix = *self.matches.get(ix.row)?;
        Some(
            ListItem::new(ix.row)
                .selected(self.selected == Some(ix))
                .child(
                    h_flex()
                        .gap_2()
                        .child(
                            div()
                                .size_8()
                                .rounded_full()
                                .bg(cx.theme().accent)
                                .flex()
                                .items_center()
                                .justify_center()
                                .child(
                                    self.all[contact_ix]
                                        .chars()
                                        .next()
                                        .unwrap_or('?')
                                        .to_string(),
                                ),
                        )
                        .child(
                            v_flex().child(self.all[contact_ix].clone()).child(
                                div()
                                    .text_xs()
                                    .text_color(cx.theme().muted_foreground)
                                    .child(format!("+39 02 {contact_ix:06}")),
                            ),
                        ),
                ),
        )
    }

    fn set_selected_index(
        &mut self,
        ix: Option<IndexPath>,
        _: &mut Window,
        cx: &mut Context<ListState<Self>>,
    ) {
        self.selected = ix;
        cx.notify();
    }
}

fn tree_items() -> Vec<TreeItem> {
    let file = |path: &str, name: &str| TreeItem::new(format!("{path}/{name}"), name.to_string());
    vec![
        TreeItem::new("src", "src")
            .expanded(true)
            .child(file("src", "lib.rs"))
            .child(file("src", "app.rs"))
            .child(
                TreeItem::new("src/screens", "screens")
                    .children((0..30).map(|i| file("src/screens", &format!("screen_{i:02}.rs")))),
            ),
        TreeItem::new("android", "android").child(
            TreeItem::new("android/app", "app")
                .child(file("android/app", "build.gradle.kts"))
                .child(file("android/app", "AndroidManifest.xml")),
        ),
        TreeItem::new("vendor", "vendor").child(
            TreeItem::new("vendor/gpui-mobile", "gpui-mobile")
                .child(file("vendor/gpui-mobile", "Cargo.toml")),
        ),
        file("", "Cargo.toml"),
        file("", "README.md"),
        TreeItem::new("disabled", "disabled folder").disabled(true),
    ]
}

pub struct ListsScreen {
    view: usize,
    list: Entity<ListState<ContactDelegate>>,
    tree: Entity<TreeState>,
    item_sizes: Rc<Vec<Size<Pixels>>>,
    virtual_scroll: VirtualListScrollHandle,
    visible: std::ops::Range<usize>,
    log: EventLog,
    _subscriptions: Vec<Subscription>,
}

impl ListsScreen {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let delegate = ContactDelegate {
            all: (0..CONTACTS).map(contact).collect(),
            matches: (0..CONTACTS).collect(),
            selected: None,
        };
        let list = cx.new(|cx| ListState::new(delegate, window, cx).searchable(true));
        let tree = cx.new(|cx| TreeState::new(cx));
        tree.update(cx, |state, cx| state.set_items(tree_items(), cx));
        let subscriptions = vec![
            cx.subscribe(&list, |this, _, event: &ListEvent, cx| {
                let message = match event {
                    ListEvent::Select(ix) => format!("list select row {}", ix.row),
                    ListEvent::Confirm(ix) => format!("list confirm row {}", ix.row),
                    ListEvent::Cancel => "list cancel".into(),
                };
                this.log.push(message);
                cx.notify();
            }),
            cx.subscribe(&tree, |this, _, event: &TreeEvent, cx| {
                let message = match event {
                    TreeEvent::Expanded(id) => format!("tree expanded {id}"),
                    TreeEvent::Collapsed(id) => format!("tree collapsed {id}"),
                };
                this.log.push(message);
                cx.notify();
            }),
        ];
        // Variable heights (32–88 px), deterministic.
        let item_sizes = Rc::new(
            (0..1_000)
                .map(|i| size(px(0.), px(32. + ((i * 37) % 57) as f32)))
                .collect(),
        );
        Self {
            view: 0,
            list,
            tree,
            item_sizes,
            virtual_scroll: VirtualListScrollHandle::new(),
            visible: 0..0,
            log: EventLog::default(),
            _subscriptions: subscriptions,
        }
    }
}

crate::hot_render!(ListsScreen);

impl ListsScreen {
    fn render_view(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let body = match self.view {
            0 => v_flex()
                .flex_1()
                .min_h_0()
                .child(ui::hint(
                    "1,000 contacts, virtualized, with built-in search. Fling hard.",
                    cx,
                ))
                .child(
                    div()
                        .flex_1()
                        .min_h_0()
                        .child(List::new(&self.list).search_placeholder("Search contacts…")),
                )
                .into_any_element(),
            1 => {
                let selected = self
                    .tree
                    .read(cx)
                    .selected_entry()
                    .map(|entry| entry.item().id.clone());
                v_flex()
                    .flex_1()
                    .min_h_0()
                    .child(ui::value_row("Selected", format!("{selected:?}"), cx))
                    .child(div().flex_1().min_h_0().child(tree(
                        &self.tree,
                        |ix, entry, selected, _, _| {
                            let item = entry.item();
                            let (chevron, icon) = if item.is_folder() {
                                if entry.is_expanded() {
                                    (Some(IconName::ChevronDown), "📂")
                                } else {
                                    (Some(IconName::ChevronRight), "📁")
                                }
                            } else {
                                (None, "📄")
                            };
                            ListItem::new(ix)
                                .selected(selected)
                                .pl(px(12. + 16. * entry.depth() as f32))
                                .child(
                                    h_flex()
                                        .gap_1()
                                        .child(
                                            div()
                                                .w_4()
                                                .children(chevron.map(|c| Icon::new(c).small())),
                                        )
                                        .child(format!("{icon} {}", item.label)),
                                )
                        },
                    )))
                    .into_any_element()
            }
            _ => v_flex()
                .flex_1()
                .min_h_0()
                .child(ui::value_row(
                    "Visible range",
                    format!("{:?} of 1000 (variable heights)", self.visible),
                    cx,
                ))
                .child(
                    v_virtual_list(
                        cx.entity(),
                        "virtual",
                        self.item_sizes.clone(),
                        |this, range, _, cx| {
                            this.visible = range.clone();
                            range
                                .map(|ix| {
                                    let height = this.item_sizes[ix].height;
                                    h_flex()
                                        .w_full()
                                        .h(height)
                                        .px_3()
                                        .border_b_1()
                                        .border_color(cx.theme().border)
                                        .text_sm()
                                        .child(format!(
                                            "Virtual row {ix} — {:.0} px",
                                            height.as_f32()
                                        ))
                                })
                                .collect()
                        },
                    )
                    .track_scroll(&self.virtual_scroll)
                    .flex_1(),
                )
                .into_any_element(),
        };
        v_flex()
            .size_full()
            .child(
                TabBar::new("list-kind")
                    .underline()
                    .selected_index(self.view)
                    .child(Tab::new().label("List (1,000)"))
                    .child(Tab::new().label("Tree"))
                    .child(Tab::new().label("VirtualList"))
                    .on_click(cx.listener(|this, ix: &usize, _, cx| {
                        this.view = *ix;
                        cx.notify();
                    })),
            )
            .child(div().flex_1().min_h_0().p_2().flex().flex_col().child(body))
            .child(div().p_2().child(self.log.render(cx)))
    }
}
