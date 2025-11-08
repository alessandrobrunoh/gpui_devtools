# GPUI Inspector - Usage Guide

## 🎯 Overview

The GPUI Inspector provides **automatic UI inspection** for GPUI applications using a simple macro-based approach. Add two macros and get a full-featured inspector with a collapsible tree view and property inspector!

## 🚀 Quick Start

### Step 1: Add `#[inspector_main]` to your main function

```rust
use gpui_inspector::inspector_main;

#[inspector_main]
fn main() {
    let app = Application::new();
    
    app.run(move |cx| {
        // Your app code here
    });
}
```

### Step 2: Add `#[auto_inspector]` to your Render implementations

```rust
use gpui_inspector::auto_inspector;

struct MyView {
    text: String,
}

#[auto_inspector]
impl Render for MyView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .child("Hello, World!")
            .child(format!("Text: {}", self.text))
    }
}
```

### Step 3: Also works with RenderOnce!

```rust
use gpui_inspector::auto_inspector;

#[derive(IntoElement)]
struct Button {
    label: SharedString,
}

#[auto_inspector]
impl RenderOnce for Button {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        div()
            .bg(rgb(0x3b82f6))
            .px_4()
            .py_2()
            .child(self.label)
    }
}
```

**That's it!** The inspector will automatically capture and display your UI tree.

## 📚 Complete Example

```rust
use gpui::{
    div, prelude::*, px, rgb, Application, Bounds, Context, Point, 
    SharedString, Size, Window, WindowOptions,
};
use gpui_inspector::{auto_inspector, inspector_main};
use gpui_inspector::inspector_view::InspectorView;

struct MainView {
    text: SharedString,
}

// Add this attribute to capture this view
#[auto_inspector]
impl Render for MainView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .gap_3()
            .bg(rgb(0x505050))
            .size(px(500.0))
            .justify_center()
            .items_center()
            .text_xl()
            .text_color(rgb(0xffffff))
            .child(format!("Hello, {}!", &self.text))
            .child(
                div()
                    .flex()
                    .gap_2()
                    .child(div().size_8().bg(gpui::red()))
                    .child(div().size_8().bg(gpui::green()))
                    .child(div().size_8().bg(gpui::blue()))
            )
    }
}

// Enable inspection globally
#[inspector_main]
fn main() {
    let app = Application::new();

    app.run(move |cx| {
        cx.spawn(async move |cx| {
            // Create your main window
            cx.open_window(WindowOptions::default(), |window, cx| {
                cx.new(|_| MainView {
                    text: "World".into(),
                })
            })?;

            // Create the inspector window
            let mut inspector_options = WindowOptions::default();
            inspector_options.window_bounds = Some(gpui::WindowBounds::Windowed(
                Bounds {
                    origin: Point { x: px(0.0), y: px(0.0) },
                    size: Size { width: px(400.0), height: px(600.0) },
                }
            ));
            cx.open_window(inspector_options, |_, cx| {
                cx.new(InspectorView::new)
            })?;

            Ok::<_, anyhow::Error>(())
        })
        .detach();
    });
}
```

## 🔧 How It Works

1. **`#[inspector_main]`** - Enables auto-inspection globally when your app starts
2. **`#[auto_inspector]`** - Marks a `Render` or `RenderOnce` impl for inspection
3. The inspector automatically captures the render tree when enabled
4. Only root views (MainView, Root) are captured to avoid overhead
5. Child components are automatically included in the parent's tree

## 🎨 Inspector UI Features

### Tree View
- **Collapsible tree** - Click ▶/▼ to expand/collapse nodes with children
- **Visual hierarchy** - Indentation shows nesting levels (20px per level)
- **Element icons** - ⬡ icon for each element
- **Selection** - Click any element to select and view its properties
- **Hover effects** - Subtle highlight on mouse hover

### Properties Panel
- **Alphabetically sorted** - Properties are always in consistent order
- **Syntax highlighting** - Color-coded keys (purple) and values (green)
- **Easy scanning** - Each property on its own line with hover effect
- **Type information** - Shows closures as `<closure>`, preserves other types

### Zed-Inspired Design
- **Background**: `#282c34` (dark gray)
- **Selection**: `#2b4f6a` (Zed blue)
- **Hover**: `#2f3440` (medium gray)
- **Borders**: `#181a1f` (almost black)
- **Icons**: `#61afef` (blue)
- **Property keys**: `#c678dd` (purple)
- **Property values**: `#98c379` (green)
- **Text**: `#abb2bf` (light gray)

## 📖 Alternative Methods

### Option 1: Old `#[inspector]` macro (Always On)

If you don't want runtime control, use the original macro:

```rust
use gpui_inspector::inspector;

#[inspector]
impl Render for MyView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div().child("Always inspected")
    }
}
```

This doesn't require `#[inspector_main]` but always captures (no enable/disable).

## 🆚 Comparison

| Method | Annotations Needed | Runtime Control | Overhead | Best For |
|--------|-------------------|-----------------|----------|----------|
| **`#[auto_inspector]` (Recommended)** | `#[inspector_main]` + per impl | ✅ Yes | Minimal | Most cases |
| **`#[inspector]`** | Per impl only | ❌ No | Always on | Quick debugging |

## 🐛 Troubleshooting

### Inspector window is empty
- ✅ Check console for `[Inspector] Auto-inspection enabled`
- ✅ Check console for `[Inspector] Target view set to: ...`
- ✅ Ensure you added `#[inspector_main]` to `main()`
- ✅ Ensure you added `#[auto_inspector]` to your views
- ✅ Make sure your view is a root view (MainView or Root component)

### Only showing "div" with minimal tree
- ✅ The inspector might be capturing the wrong view
- ✅ Check console logs to see which view is being rendered
- ✅ Only MainView and Root are captured by default
- ✅ Child components (Text, Button) are included in parent's tree

### Properties appear in random order
- ✅ **Fixed!** Properties are now alphabetically sorted using `BTreeMap`
- ✅ Order is consistent regardless of how many times you select an element

### Elements not expanding/collapsing
- ✅ Make sure the element has children (▶ icon appears)
- ✅ Click directly on the ▶/▼ icon, not the element name
- ✅ Check console for any errors

### Compilation errors
- ✅ Ensure `gpui-inspector` is in your `Cargo.toml`:
  ```toml
  [dependencies]
  gpui-inspector = { path = "../gpui-inspector" }
  ```
- ✅ Import the macros: `use gpui_inspector::{auto_inspector, inspector_main};`
- ✅ Check that proc-macro crate is building correctly

### Performance issues
- ✅ Inspection only runs when `#[inspector_main]` is active
- ✅ Only root views are captured (not every component)
- ✅ Atomic flag checks are very fast (nanoseconds)
- ✅ Remove `#[inspector_main]` in release builds:
  ```rust
  #[cfg(debug_assertions)]
  #[inspector_main]
  fn main() { /* ... */ }
  ```

## 📝 Technical Details

### What Gets Captured?
- ✅ Element hierarchy (div, button, etc.)
- ✅ Element properties (bg, size, flex, etc.)
- ✅ Method calls with arguments
- ✅ Nested children (recursive)
- ✅ Component names
- ✅ Closures (shown as `<closure>`)

### What Doesn't Get Captured?
- ❌ InspectorView itself (excluded automatically)
- ❌ Child components like Text, Button (unless marked separately)
- ❌ Non-root views (only MainView and Root by default)

### Smart Filtering
The inspector automatically:
- Excludes `InspectorView` to prevent recursion
- Captures only root views to minimize overhead
- Registers the first non-inspector view as the target
- Uses atomic flags for zero-cost checks when disabled

## 💡 Best Practices

### 1. Use `#[auto_inspector]` on root views only
```rust
#[auto_inspector]
impl Render for MainView { /* ... */ }

// Don't need it here - included in MainView's tree
impl RenderOnce for Button { /* ... */ }
```

### 2. Create inspector window with specific bounds
```rust
let mut inspector_options = WindowOptions::default();
inspector_options.window_bounds = Some(gpui::WindowBounds::Windowed(
    Bounds {
        origin: Point { x: px(0.0), y: px(0.0) },
        size: Size { width: px(400.0), height: px(600.0) },
    }
));
```

### 3. Enable only in debug builds
```rust
#[cfg(debug_assertions)]
#[inspector_main]
#[cfg(not(debug_assertions))]
fn main() {
    // Production build without inspection overhead
}
```

### 4. Check console for debugging info
The inspector prints useful information:
- `[Inspector] Auto-inspection enabled`
- `[Inspector] Target view set to: ...`
- `[Inspector] Registered view: ...`
- `Render ViewName:` with full tree structure

## 🎉 Benefits

✨ **Simple setup** - Two macros and you're done  
✨ **Runtime control** - Enable/disable via `#[inspector_main]`  
✨ **Zero overhead** - When disabled, just an atomic bool check  
✨ **Type-safe** - All checks at compile-time  
✨ **Beautiful UI** - Zed-inspired design with syntax highlighting  
✨ **Consistent ordering** - Properties always alphabetically sorted  
✨ **Smart filtering** - Automatically excludes inspector itself  

## 🔍 Example Output

Console:
```
[Inspector] Auto-inspection enabled
[Inspector] Installing render hook for: my_app::MainView (id: 4294967297)
[Inspector] Target view set to: my_app::MainView
Render MainView:
div(id: 16)
.bg("rgb(0x505050)")
.flex("")
.flex_col("")
.gap_3("")
.size("px(500.0)")
  format!("Hello, {}!", &self.text)(id: 17)
  div(id: 30)
  .flex("")
  .gap_2("")
    div(id: 33)
    .bg("gpui::red()")
    .size_8("")
```

Inspector UI:
```
ELEMENT TREE              │ PROPERTIES
▼ ⬡ div                   │ MainView
  ⬡ format!(...)          │ 
  ▼ ⬡ div                 │ bg: rgb(0x505050)
    ⬡ div                 │ flex: 
    ⬡ div                 │ flex_col: 
    ⬡ div                 │ gap_3: 
                          │ size: px(500.0)
```

---

**Need help?** Check the console logs for detailed information about what's being captured.

**Found a bug?** The inspector is still in development - feedback welcome!

Enjoy debugging your GPUI apps! 🚀