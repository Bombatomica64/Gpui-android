use gpui::{Context, IntoElement, ParentElement, Render, SharedString, Styled, Window, div};
use gpui_kit::base::{SelectionFormat, TextView};
use gpui_kit::component::{
    button::Button,
    h_flex,
    tab::{Tab, TabBar},
    v_flex,
};

use crate::ui::{self, EventLog, prelude::*};

const MARKDOWN: &str = r#"# Heading 1
## Heading 2
### Heading 3

Plain paragraph with **bold**, *italic*, ~~strike~~, `inline code` and a [link to gpui-kit.com](https://gpui-kit.com).
Unicode: English Italiano àèéìòù 日本語 中文测试 한국어 😀🚀🦀

- Bullet one
- Bullet two with a long line that has to wrap on a phone screen because it keeps going and going
  - Nested bullet
- [x] Done task
- [ ] Open task

1. First
2. Second
3. Third

> A blockquote. Long-press to select text inside it.

```rust
fn main() {
    // A code block wider than the phone should scroll or wrap, not clip silently.
    let greeting = "Hello from a very long line of Rust code that exceeds the viewport width";
    println!("{greeting}");
}
```

| Component | Status | Notes |
|---|---|---|
| TextView | ? | Markdown table on a narrow screen |
| Input | ? | Wide tables must scroll horizontally |
| Chart | ? | 3 columns |

![GitHub icon from the asset source](icons/github.svg)

---

Final paragraph after a horizontal rule.
"#;

const HTML: &str = r#"<h2>HTML content</h2>
<p>Paragraph with <b>bold</b>, <i>italic</i>, <code>code</code> and <a href="https://example.com">a link</a>.</p>
<ul><li>First item</li><li>Second item</li></ul>
<table><tr><th>Key</th><th>Value</th></tr><tr><td>Alpha</td><td>1</td></tr><tr><td>Beta</td><td>2</td></tr></table>"#;

fn large_document(sections: usize) -> SharedString {
    let mut out = String::with_capacity(sections * 400);
    for i in 1..=sections {
        out.push_str(&format!(
            "## Section {i}\n\nParagraph {i}: Lorem ipsum dolor sit amet, consectetur adipiscing elit. \
             Integer nec odio. Praesent libero. Sed cursus ante dapibus diam. `code {i}` and **bold {i}**.\n\n\
             - item {i}.a\n- item {i}.b\n\n"
        ));
    }
    out.into()
}

pub struct RichTextScreen {
    tab: usize,
    source_selection: bool,
    large: Option<(usize, SharedString)>,
    log: EventLog,
}

impl RichTextScreen {
    pub fn new(_: &mut Window, _: &mut Context<Self>) -> Self {
        Self { tab: 0, source_selection: false, large: None, log: EventLog::default() }
    }

    fn load(&mut self, sections: usize, cx: &mut Context<Self>) {
        let started = std::time::Instant::now();
        let text = large_document(sections);
        self.log.push(format!(
            "generated {sections} sections ({} KB) in {:?}",
            text.len() / 1024,
            started.elapsed()
        ));
        self.large = Some((sections, text));
        cx.notify();
    }
}

impl Render for RichTextScreen {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let view = cx.entity();
        let on_link = move |href: &SharedString, _: &gpui::ClickEvent, _: &mut Window, cx: &mut gpui::App| {
            let href = href.clone();
            view.update(cx, |this, cx| {
                this.log.push(format!("link tapped: {href}"));
                cx.notify();
            });
        };
        let format = if self.source_selection { SelectionFormat::Source } else { SelectionFormat::Plain };
        let body = match self.tab {
            0 => TextView::markdown("markdown", MARKDOWN)
                .selectable(true)
                .selection_format(format)
                .on_link_click(on_link)
                .w_full()
                .min_w_0()
                .into_any_element(),
            1 => TextView::html("html", HTML)
                .selectable(true)
                .on_link_click(on_link)
                .w_full()
                .min_w_0()
                .into_any_element(),
            _ => v_flex()
                .gap_2()
                .child(
                    h_flex()
                        .gap_1()
                        .children([100usize, 500, 2000].map(|n| {
                            Button::new(("load", n)).small().outline().label(format!("{n} sections")).on_click(
                                cx.listener(move |this, _, _, cx| this.load(n, cx)),
                            )
                        })),
                )
                .child(match &self.large {
                    Some((sections, text)) => TextView::markdown(("large", *sections), text.clone())
                        .selectable(true)
                        .w_full()
                        .min_w_0()
                        .into_any_element(),
                    None => ui::hint("Pick a size; then fling through the document.", cx).into_any_element(),
                })
                .into_any_element(),
        };

        v_flex()
            .gap_3()
            .child(
                TabBar::new("rich-kind")
                    .underline()
                    .selected_index(self.tab)
                    .child(Tab::new().label("Markdown"))
                    .child(Tab::new().label("HTML"))
                    .child(Tab::new().label("Large"))
                    .on_click(cx.listener(|this, ix: &usize, _, cx| {
                        this.tab = *ix;
                        cx.notify();
                    })),
            )
            .child(
                h_flex().gap_2().child(
                    Button::new("selection-format")
                        .small()
                        .outline()
                        .selected(self.source_selection)
                        .label(if self.source_selection { "Copy as Markdown source" } else { "Copy as plain text" })
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.source_selection = !this.source_selection;
                            cx.notify();
                        })),
                ),
            )
            .child(ui::hint(
                "Long-press to select a word in the TextView (double-tap intentionally does not \
                 select there), drag the handles, then Copy. Paste into Text Input to verify.",
                cx,
            ))
            .child(div().w_full().child(body))
            .child(self.log.render(cx))
    }
}
