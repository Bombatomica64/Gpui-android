# GPUI Kit v0.7.0 — Android component matrix

Every user-facing component exported by `gpui-kit` 0.7.0 (`gpui-component`,
`gpui-base`, `gpui-fps`), and where the Mobile Lab APK exercises it. The app
embeds this file (`include_str!`) and builds its catalog, status counters and
notes from the tables below, so this document and the APK cannot disagree.

Status meanings:

- **WORKING** — renders and its interactions behave correctly on Android.
- **PARTIAL** — usable, but some interaction or state is wrong or missing.
- **BROKEN** — does not render or cannot be used.
- **NOT_APPLICABLE** — desktop-window concept with no meaning inside an Android app.

How statuses were determined is described in [README.md](README.md#how-statuses-were-verified).

## Actions

| Component | Included in APK | Mobile status | Demo screen | Notes |
|---|---|---|---|---|
| Button | Yes | UNTESTED | Buttons | |
| ButtonGroup | Yes | UNTESTED | Buttons | |
| DropdownButton | Yes | UNTESTED | Buttons | |
| Toggle / ToggleGroup | Yes | UNTESTED | Buttons | |
| Link | Yes | UNTESTED | Buttons | |
| Kbd | Yes | UNTESTED | Buttons | |
| Toolbar | Yes | UNTESTED | Toolbar | |
| ToolbarGroup | Yes | UNTESTED | Toolbar | |

## Forms

| Component | Included in APK | Mobile status | Demo screen | Notes |
|---|---|---|---|---|
| Checkbox | Yes | UNTESTED | Choice Controls | |
| Radio / RadioGroup | Yes | UNTESTED | Choice Controls | |
| Switch | Yes | UNTESTED | Choice Controls | |
| Slider | Yes | UNTESTED | Choice Controls | |
| Rating | Yes | UNTESTED | Choice Controls | |
| Stepper | Yes | UNTESTED | Choice Controls | |
| Form (v_form / h_form / field) | Yes | UNTESTED | Settings & Form | |
| Settings / SettingGroup | Yes | UNTESTED | Settings & Form | |
| GroupBox | Yes | UNTESTED | Settings & Form | |
| Questionnaire | Yes | UNTESTED | Questionnaire | |

## Text & Editing

| Component | Included in APK | Mobile status | Demo screen | Notes |
|---|---|---|---|---|
| Input | Yes | UNTESTED | Text Input | |
| Input (password / masked) | Yes | UNTESTED | Text Input | |
| Input (mask pattern) | Yes | UNTESTED | Text Input | |
| InputGroup | Yes | UNTESTED | Text Input | |
| Clipboard | Yes | UNTESTED | Text Input | |
| Label | Yes | UNTESTED | Text Input | |
| Textarea | Yes | UNTESTED | Textarea & Editor | |
| Inline tokens (InputToken) | Yes | UNTESTED | Textarea & Editor | |
| Editor | Yes | UNTESTED | Textarea & Editor | |
| Highlighter (tree-sitter) | No | NOT_APPLICABLE | Textarea & Editor | Behind the `tree-sitter` feature, not enabled to keep the APK small; the Editor runs without syntax colors. |
| NumberInput | Yes | UNTESTED | Number & OTP | |
| OtpInput | Yes | UNTESTED | Number & OTP | |

## Selection

| Component | Included in APK | Mobile status | Demo screen | Notes |
|---|---|---|---|---|
| Select | Yes | UNTESTED | Select & Combobox | |
| Combobox | Yes | UNTESTED | Select & Combobox | |
| ColorPicker | Yes | UNTESTED | Select & Combobox | |

## Navigation

| Component | Included in APK | Mobile status | Demo screen | Notes |
|---|---|---|---|---|
| Tabs (Tab / TabBar) | Yes | UNTESTED | Navigation | |
| Breadcrumb | Yes | UNTESTED | Navigation | |
| Pagination | Yes | UNTESTED | Navigation | |
| Carousel | Yes | UNTESTED | Navigation | |
| Sidebar | Yes | UNTESTED | Navigation | |
| Command | Yes | UNTESTED | Command | |
| Dock (DockArea / Panel) | Yes | UNTESTED | Dock | |

## Overlays

| Component | Included in APK | Mobile status | Demo screen | Notes |
|---|---|---|---|---|
| Dialog | Yes | UNTESTED | Dialogs & Sheets | |
| AlertDialog | Yes | UNTESTED | Dialogs & Sheets | |
| Sheet | Yes | UNTESTED | Dialogs & Sheets | |
| Notification | Yes | UNTESTED | Dialogs & Sheets | |
| Popover | Yes | UNTESTED | Popovers & Menus | |
| HoverCard | Yes | UNTESTED | Popovers & Menus | |
| Tooltip | Yes | UNTESTED | Popovers & Menus | |
| PopupMenu (dropdown menu) | Yes | UNTESTED | Popovers & Menus | |
| Context menu (ContextMenuExt) | Yes | UNTESTED | Popovers & Menus | |
| NativeMenu | Yes | UNTESTED | Popovers & Menus | |

## Data Display

| Component | Included in APK | Mobile status | Demo screen | Notes |
|---|---|---|---|---|
| Table | Yes | UNTESTED | Tables | |
| DataTable | Yes | UNTESTED | Tables | |
| List | Yes | UNTESTED | Lists & Trees | |
| Tree | Yes | UNTESTED | Lists & Trees | |
| VirtualList | Yes | UNTESTED | Lists & Trees | |
| DescriptionList | Yes | UNTESTED | Display | |
| Avatar / AvatarGroup | Yes | UNTESTED | Display | |
| Badge | Yes | UNTESTED | Display | |
| Tag | Yes | UNTESTED | Display | |
| Icon / IconName | Yes | UNTESTED | Display | |
| Image (img) | Yes | UNTESTED | Display | |
| Empty | Yes | UNTESTED | Display | |
| Accordion | Yes | UNTESTED | Disclosure | |
| Collapsible | Yes | UNTESTED | Disclosure | |

## Feedback

| Component | Included in APK | Mobile status | Demo screen | Notes |
|---|---|---|---|---|
| Alert | Yes | UNTESTED | Feedback | |
| Progress / ProgressCircle | Yes | UNTESTED | Feedback | |
| Spinner | Yes | UNTESTED | Feedback | |
| Skeleton | Yes | UNTESTED | Feedback | |
| ShimmerText | Yes | UNTESTED | Feedback | |
| Marker | Yes | UNTESTED | Feedback | |
| StatusBar | Yes | UNTESTED | Feedback | |

## Layout

| Component | Included in APK | Mobile status | Demo screen | Notes |
|---|---|---|---|---|
| Separator | Yes | UNTESTED | Layout | |
| Scrollbar / ScrollableElement | Yes | UNTESTED | Layout | |
| Resizable panels | Yes | UNTESTED | Layout | |
| h_flex / v_flex | Yes | UNTESTED | Layout | |
| Root (overlay host) | Yes | UNTESTED | Dialogs & Sheets | |
| TitleBar | No | NOT_APPLICABLE | — | Desktop window chrome (traffic lights, caption buttons, window dragging). Android owns the status bar. |
| WindowBorder | No | NOT_APPLICABLE | — | Linux client-side-decoration shadow/resize border; no-op outside Linux. |

## Date & Time

| Component | Included in APK | Mobile status | Demo screen | Notes |
|---|---|---|---|---|
| Calendar | Yes | UNTESTED | Date & Time | |
| DatePicker | Yes | UNTESTED | Date & Time | |
| DatePicker (range) | Yes | UNTESTED | Date & Time | |
| DatePicker (date + time) | Yes | UNTESTED | Date & Time | |
| TimeField | Yes | UNTESTED | Date & Time | |

## Charts

| Component | Included in APK | Mobile status | Demo screen | Notes |
|---|---|---|---|---|
| LineChart | Yes | UNTESTED | Charts | |
| BarChart | Yes | UNTESTED | Charts | |
| AreaChart | Yes | UNTESTED | Charts | |
| PieChart | Yes | UNTESTED | Charts | |
| RadarChart | Yes | UNTESTED | Charts | |
| CandlestickChart | Yes | UNTESTED | Charts | |
| SankeyChart | Yes | UNTESTED | Charts | |
| Plot primitives (custom Plot) | Yes | UNTESTED | Charts | |

## Rich Content

| Component | Included in APK | Mobile status | Demo screen | Notes |
|---|---|---|---|---|
| TextView (Markdown) | Yes | UNTESTED | Rich Text | |
| TextView (HTML) | Yes | UNTESTED | Rich Text | |
| Text selection (TextView) | Yes | UNTESTED | Rich Text | |
| Message / MessageGroup | Yes | UNTESTED | Chat | |
| Bubble | Yes | UNTESTED | Chat | |
| MessageScroller | Yes | UNTESTED | Chat | |
| Attachment | Yes | UNTESTED | Chat | |

## Miscellaneous

| Component | Included in APK | Mobile status | Demo screen | Notes |
|---|---|---|---|---|
| Theme / ThemeRegistry | Yes | UNTESTED | Themes | |
| FPS monitor (gpui-fps) | Yes | UNTESTED | Diagnostics | |
| Touch input (tap / long press / fling) | Yes | UNTESTED | Touch Lab | |
| Scrolling stress | Yes | UNTESTED | Scroll Stress | |
| Rendering stress | Yes | UNTESTED | Stress Test | |
