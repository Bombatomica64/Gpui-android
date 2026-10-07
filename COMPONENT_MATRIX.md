# GPUI Kit v0.7.1 — Android component matrix

Every user-facing component exported by `gpui-kit` 0.7.0 (`gpui-component`,
`gpui-base`, `gpui-fps`), and where the Mobile Lab APK exercises it. The app
embeds this file (`include_str!`) and builds its catalog, status counters and
notes from the tables below, so this document and the APK cannot disagree.

Status meanings:

- **WORKING** — renders and its tested interactions behave correctly on Android.
- **PARTIAL** — usable, but some interaction, state or layout is wrong or missing.
- **BROKEN** — cannot be used on a touch-only phone (it may still render).
- **NOT_APPLICABLE** — no meaning inside an Android app, or disabled on mobile by design.

"(patched)" means the status holds with the fixes our forks carry, listed in
[MOBILE_PATCHES.md](MOBILE_PATCHES.md). How statuses were determined, and the
limits of the test device, are in [README.md](README.md#how-statuses-were-verified).

## Actions

| Component | Included in APK | Mobile status | Demo screen | Notes |
|---|---|---|---|---|
| Button | Yes | WORKING | Buttons | All variants and sizes; disabled buttons ignore taps; a loading button ignores repeat taps. |
| ButtonGroup | Yes | WORKING | Buttons | Multi-select reports the selected indices. |
| DropdownButton | Yes | WORKING | Buttons | Menu flips above the trigger when there is no room below. The row under the finger keeps a hover highlight after the tap. |
| Toggle / ToggleGroup | Yes | WORKING | Buttons | Segmented group and single toggle report state. Checked items use `accent`, which is low-contrast in the dark theme. |
| Link | Yes | WORKING | Buttons | `on_click` fires. Opening an `href` (`cx.open_url`) was not verified: the test device has no browser. |
| Kbd | Yes | WORKING | Buttons | Display only; Linux-style labels. |
| Toolbar | Yes | WORKING | Toolbar | Buttons, toggles, sizes and the disabled state work. A toolbar wider than the screen is clipped (no overflow menu). |
| ToolbarGroup | Yes | WORKING | Toolbar | Groups render with separators; hosted controls work. |

## Forms

| Component | Included in APK | Mobile status | Demo screen | Notes |
|---|---|---|---|---|
| Checkbox | Yes | WORKING | Choice Controls | Wrapped labels are tappable; disabled ignores taps. |
| Radio / RadioGroup | Yes | PARTIAL | Choice Controls | Selection works, but a `Radio` marked `.disabled(true)` inside a `RadioGroup` can still be selected (not Android-specific; longbridge/gpui-kit#3324, fix in #3331). |
| Switch | Yes | WORKING | Choice Controls | Default and small sizes; disabled ignores taps. |
| Slider | Yes | WORKING | Choice Controls | (patched) Tap-to-set works upstream; dragging the thumb needed a fix (Slider only handled mouse drags), upstream since v0.7.1 (#3313). Horizontal, vertical and range. |
| Rating | Yes | WORKING | Choice Controls | Tap sets the value; disabled is read-only. |
| Stepper | Yes | WORKING | Choice Controls | Horizontal and vertical. |
| Form (v_form / h_form / field) | Yes | WORKING | Settings & Form | Labels, required marker, description and footer. |
| Settings / SettingGroup | Yes | PARTIAL | Settings & Form | Fields persist values, but the desktop sidebar takes ~70% of a 411 pt screen and the content column breaks words mid-word. |
| GroupBox | Yes | WORKING | Settings & Form | Normal, fill and outline variants with footer. |
| Questionnaire | Yes | WORKING | Questionnaire | Single and multiple choice, disabled choice, freeform input with keyboard, validator error, Previous/Skip/Next/Submit; emits Completed then Submit. |

## Text & Editing

| Component | Included in APK | Mobile status | Demo screen | Notes |
|---|---|---|---|---|
| Input | Yes | WORKING | Text Input | Typing (LatinIME composition and commit), backspace, CJK/Hangul/emoji, long-press word selection with handles, Cut/Copy/Paste/Select All menu, system clipboard (gpui-mobile fork, #24). Keyboard avoidance and IME dismissal come from the forks (see MOBILE_PATCHES.md), not yet re-verified on a phone. |
| Input (password / masked) | Yes | WORKING | Text Input | Bullets and the reveal toggle work. |
| Input (mask pattern) | Yes | WORKING | Text Input | Phone and number masks format while typing; a regex pattern filters characters. Opens the text keyboard, never the numeric one (gpui-mobile always requests the default keyboard type). |
| InputGroup | Yes | WORKING | Text Input | Addons and the addon button work. |
| Clipboard | Yes | WORKING | Text Input | `value_fn` + notification and static values copy to the system clipboard. |
| Label | Yes | WORKING | Text Input | Highlights, secondary text and masking render. |
| Textarea | Yes | PARTIAL | Textarea & Editor | Typing, newlines and `auto_grow` work; a `.rows(5)` textarea renders one row tall and clips its content (not Android-specific; longbridge/gpui-kit#3325, fix in #3330). |
| Inline tokens (InputToken) | Yes | WORKING | Textarea & Editor | Tokens render with icons; backspace deletes a whole token. |
| Editor | Yes | WORKING | Textarea & Editor | Line numbers, soft wrap, typing and internal scrolling. |
| Highlighter (tree-sitter) | No | NOT_APPLICABLE | Textarea & Editor | Behind the `tree-sitter` feature, not enabled to keep the APK small; the Editor runs without syntax colors. |
| NumberInput | Yes | PARTIAL | Number & OTP | +/− steps, clamping, app-handled steps and the decimal mask work; typing opens the full text keyboard instead of a numeric one. |
| OtpInput | Yes | BROKEN | Number & OTP | Renders and focuses, and completes with a hardware keyboard, but no soft keyboard appears on tap: OtpState has no text-input handler and never requests the virtual keyboard. |

## Selection

| Component | Included in APK | Mobile status | Demo screen | Notes |
|---|---|---|---|---|
| Select | Yes | WORKING | Select & Combobox | Confirm, searchable list with keyboard (list re-fits above it), fling. A popup anchors to the last-rendered element of a shared state, so each Select needs its own state. |
| Combobox | Yes | WORKING | Select & Combobox | Single (searchable) and multiple (Change per toggle, Confirm on close). |
| ColorPicker | Yes | WORKING | Select & Combobox | Palette swatches and hex value work. The HSLA tab's sliders use Slider but were not tested individually. |

## Navigation

| Component | Included in APK | Mobile status | Demo screen | Notes |
|---|---|---|---|---|
| Tabs (Tab / TabBar) | Yes | WORKING | Navigation | All four variants; an overflowing TabBar scrolls horizontally. |
| Breadcrumb | Yes | PARTIAL | Navigation | Taps work, but the trail never wraps, so the last levels are clipped at 411 pt. |
| Pagination | Yes | PARTIAL | Navigation | Taps work; the full variant overflows a 411 pt screen (Next is clipped). The compact variant fits. |
| Carousel | Yes | PARTIAL | Navigation | Swiping changes slides, but one fling skips several slides (each momentum delta counts as a step), and the prev/next buttons render outside the card, clipped at the screen edges. |
| Sidebar | Yes | WORKING | Navigation | Selection and collapse to icons; the header text overflows when collapsed. |
| Command | Yes | WORKING | Command | Inline and in a dialog: filter with keyboard, confirm closes the dialog. |
| Dock (DockArea / Panel) | Yes | PARTIAL | Dock | Tab switching, panel taps and the panel menu work; splitter resize and tab drag/split do nothing (GPUI drag-and-drop is mouse-only on touch). |

## Overlays

| Component | Included in APK | Mobile status | Demo screen | Notes |
|---|---|---|---|---|
| Dialog | Yes | WORKING | Dialogs & Sheets | Open, tap outside, back closes only the dialog, nested dialogs, input stays above the keyboard, background does not scroll, open/close sequences. |
| AlertDialog | Yes | WORKING | Dialogs & Sheets | OK/Cancel callbacks; slide-in animation. |
| Sheet | Yes | WORKING | Dialogs & Sheets | Bottom/top/left/right, back and outside tap close, content scrolls. Closing a sheet during a fling hands the remaining momentum to the page behind. |
| Notification | Yes | WORKING | Dialogs & Sheets | Toasts fit a 411 pt screen; sticky notification with tap handler. |
| Popover | Yes | PARTIAL | Popovers & Menus | Open, outside tap, controlled mode and input content work; a popover near the right edge is not moved back into the viewport and is clipped. |
| HoverCard | Yes | WORKING | Popovers & Menus | Tap-to-open on mobile, outside tap closes. |
| Tooltip | Yes | NOT_APPLICABLE | Popovers & Menus | Kit disables its tooltip overlay on iOS/Android by design; a raw GPUI `.tooltip()` did not appear on long-press either. |
| PopupMenu (dropdown menu) | Yes | PARTIAL | Popovers & Menus | Items, checks, links and a 40-item scrolling menu work; submenus open to the left and are clipped at the screen edge. |
| Context menu (ContextMenuExt) | Yes | WORKING | Popovers & Menus | (patched) Long-press opens the menu at the finger and items fire; upstream opens it only on the right mouse button, so this needs the Kit fork's long-press fix (longbridge/gpui-kit#3392, fix in #3393). |
| NativeMenu | Yes | WORKING | Popovers & Menus | GPUI-drawn fallback shows at the tap position; actions dispatch along the focus path. |

## Data Display

| Component | Included in APK | Mobile status | Demo screen | Notes |
|---|---|---|---|---|
| Table | Yes | PARTIAL | Tables | Renders, but the last column is clipped at 411 pt (no shrinking or horizontal scroll). |
| DataTable | Yes | WORKING | Tables | 1,000 virtualized rows: row/column selection, sort chevron, pinned column with horizontal scroll, fling, `scroll_to_row`. Column resize/reorder are drag gestures and do not work on touch. A tap also emitted `RightClickedRow(None)`. |
| List | Yes | WORKING | Lists & Trees | 1,000 virtualized rows, fling, tap-to-stop, select/confirm, built-in search with keyboard, empty state. |
| Tree | Yes | WORKING | Lists & Trees | Expand/collapse, select, scroll. |
| VirtualList | Yes | WORKING | Lists & Trees | 1,000 variable-height rows, fling. |
| DescriptionList | Yes | PARTIAL | Display | The two-column horizontal layout breaks words mid-word at 411 pt; the vertical layout is fine. |
| Avatar / AvatarGroup | Yes | WORKING | Display | Initials (including Hangul), placeholder, group with limit. |
| Badge | Yes | WORKING | Display | Count, 99+, dot, icon; updates live. |
| Tag | Yes | WORKING | Display | All variants, outline, rounded. |
| Icon / IconName | Yes | WORKING | Display | Default icons and the full Lucide catalog (the app registers `AllAssets`). |
| Image (img) | Yes | WORKING | Display | In-memory SVG bytes, asset-source SVG and fallback. Network images not tested (no HTTP client configured). |
| Empty | Yes | WORKING | Display | Media, title, description, action. |
| Accordion | Yes | PARTIAL | Disclosure | Toggling and animation work; a long title does not wrap and pushes the chevron out of view. |
| Collapsible | Yes | WORKING | Disclosure | App-driven open state with motion. |

## Feedback

| Component | Included in APK | Mobile status | Demo screen | Notes |
|---|---|---|---|---|
| Alert | Yes | WORKING | Feedback | All variants, banner, close and restore. |
| Progress / ProgressCircle | Yes | WORKING | Feedback | Values, animation and indeterminate mode. |
| Spinner | Yes | WORKING | Feedback | Sizes, colors, custom icon. |
| Skeleton | Yes | WORKING | Feedback | Pulse animation. |
| ShimmerText | Yes | PARTIAL | Feedback | Repeating and one-shot sweeps animate, but in dark mode the default highlight is invisible on `foreground` text: it is the text color mixed 80% toward `foreground`, i.e. the text's own color (not Android-specific; longbridge/gpui-kit#3327, fix in #3328). Muted text shimmers normally. The demo uses muted text with a `foreground` highlight. |
| Marker | Yes | WORKING | Feedback | Separator, spinner and shimmer loading styles. |
| StatusBar | Yes | WORKING | Feedback | Left/right items with a tappable button. |

## Layout

| Component | Included in APK | Mobile status | Demo screen | Notes |
|---|---|---|---|---|
| Separator | Yes | WORKING | Layout | Horizontal, vertical, dashed, labelled. |
| Scrollbar / ScrollableElement | Yes | WORKING | Layout | Thin mobile scrollbars; nested vertical scroller and horizontal strips. Vertical-only containers need `restrict_scroll_to_axis()` on touch (see README). |
| Resizable panels | Yes | BROKEN | Layout | Renders, but dividers cannot be dragged by touch (GPUI drag-and-drop is mouse-only), so panels cannot be resized on a phone. |
| h_flex / v_flex | Yes | WORKING | Layout | Truncation and fixed/flexible children at phone width. |
| Root (overlay host) | Yes | WORKING | Dialogs & Sheets | v0.7.0 Root hosts dialogs, sheets and notifications with no manual layers. |
| TitleBar | No | NOT_APPLICABLE | — | Desktop window chrome (traffic lights, caption buttons, window dragging). Android owns the status bar. |
| WindowBorder | No | NOT_APPLICABLE | — | Linux client-side-decoration shadow/resize border; no-op outside Linux. |

## Date & Time

| Component | Included in APK | Mobile status | Demo screen | Notes |
|---|---|---|---|---|
| Calendar | Yes | WORKING | Date & Time | Selection and month navigation. |
| DatePicker | Yes | WORKING | Date & Time | Single date with disabled weekends; presets panel fits beside the calendar. |
| DatePicker (range) | Yes | WORKING | Date & Time | Range selection. |
| DatePicker (date + time) | Yes | PARTIAL | Date & Time | The date part works; its embedded TimeField can only be edited with a hardware keyboard (see TimeField). |
| TimeField | Yes | BROKEN | Date & Time | 24h/12h and minute/second formats render and `set_time` works, but tapping a segment shows no soft keyboard (no text-input handler; gpui-mobile's `show_soft_keyboard` is a no-op), so the time cannot be edited on a phone without a hardware keyboard. |

## Charts

| Component | Included in APK | Mobile status | Demo screen | Notes |
|---|---|---|---|---|
| LineChart | Yes | WORKING | Charts | Long-press opens the tooltip, dragging moves it, lifting closes it; live data and a 220 pt narrow layout work (x labels then overlap). |
| BarChart | Yes | WORKING | Charts | Negative values with a conditional fill. |
| AreaChart | Yes | WORKING | Charts | Two series. |
| PieChart | Yes | WORKING | Charts | Donut with labels. |
| RadarChart | Yes | WORKING | Charts | Two series. |
| CandlestickChart | Yes | WORKING | Charts | Renders; x-axis labels overlap at phone width. |
| SankeyChart | Yes | WORKING | Charts | Nodes, links, labels. |
| Plot primitives (custom Plot) | Yes | WORKING | Charts | A custom `Plot` on `PlotElement` + `ScaleBand`/`ScaleLinear`; its tooltip hook works on long-press. |

## Rich Content

| Component | Included in APK | Mobile status | Demo screen | Notes |
|---|---|---|---|---|
| TextView (Markdown) | Yes | PARTIAL | Rich Text | Headings, lists, tasks, quotes, code, tables, images, CJK/emoji and link taps work, but **bold and italic render as regular** (Android's Roboto is a variable font and the text system does not apply weight/italic). A 2,000-section document renders but scrolls very slowly without `scrollable(true)`. |
| TextView (HTML) | Yes | WORKING | Rich Text | Paragraphs, lists, links, table. |
| Text selection (TextView) | Yes | PARTIAL | Rich Text | Long-press shows handles and Copy/Select All, and Copy reaches the system clipboard, but the initial selection is not reliably the pressed word (collapsed or starting mid-word). |
| Message / MessageGroup | Yes | WORKING | Chat | Avatars, headers, footers, alignment. |
| Bubble | Yes | PARTIAL | Chat | All variants render, but a TextView inside a `Filled` bubble ignores the bubble's `primary_foreground` and paints the theme `foreground`, so in dark mode it is white on white (not Android-specific). The demo passes a `TextViewStyle` with `primary_foreground` text and links and code backgrounds derived from it (longbridge/gpui-kit#3326, fix in #3329). |
| MessageScroller | Yes | WORKING | Chat | Tail-follow, append, `scroll_to_end`, jump-to-latest button. |
| Attachment | Yes | WORKING | Chat | Status, progress, remove. Without an AttachmentGroup a second card overflows the edge. |

## Miscellaneous

| Component | Included in APK | Mobile status | Demo screen | Notes |
|---|---|---|---|---|
| Theme / ThemeRegistry | Yes | WORKING | Themes | Runtime System/Light/Dark and bundled Kit themes. System follows Android dark mode once the host forwards `uiMode` (see README). |
| FPS monitor (gpui-fps) | Yes | WORKING | Diagnostics | (patched) Opening the HUD panicked (`monospace` font unresolvable on Android) until a fix that is upstream since v0.7.1 (#3313). FPS/frame time and CPU/memory rows populate. |
| Touch input (tap / long press / fling) | Yes | WORKING | Touch Lab | Tap, double-tap (`click_count` 2), claimed long-press (Started/Ended), touch drag, fling, tap-to-stop-fling without activating the row. |
| Scrolling stress | Yes | WORKING | Scroll Stress | 100 plain rows, 10,000 virtual rows, nested axes and long text; one axis per gesture once containers restrict their axis. |
| Rendering stress | Yes | WORKING | Stress Test | Start/Stop/Reset; per-frame `render()` cost is reported alongside frame intervals. |
