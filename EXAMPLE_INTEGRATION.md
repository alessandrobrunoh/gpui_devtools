# Esempio Pratico: Integrazione GPUI Inspector

Questo esempio mostra come usare l'integrazione tra `gpui-inspector` e il sistema di inspector di GPUI in un'applicazione reale.

## 📦 Setup Iniziale

### 1. Aggiungi dipendenze in `Cargo.toml`

```toml
[workspace]
members = ["gpui-inspector", "gpui-inspector-macros", "my_app"]

[workspace.dependencies]
gpui = "*"
gpui-inspector = { path = "gpui-inspector" }
```

```toml
# my_app/Cargo.toml
[package]
name = "my_app"
version = "0.1.0"
edition = "2021"

[dependencies]
gpui = { workspace = true }
gpui-inspector = { workspace = true }
```

## 🚀 Esempio Completo

### File: `my_app/src/main.rs`

```rust
use gpui::{
    div, prelude::*, px, rgb, App, Application, Bounds, Context, Entity, 
    InteractiveElement, IntoElement, ParentElement, Render, SharedString, 
    Size, Window, WindowOptions,
};
use gpui_inspector::{auto_inspector, inspector_main};
use gpui_inspector::inspector_view::InspectorView;
use gpui_inspector::hooks::gpui_inspector_integration;

// ============================================
// STRUTTURA DELL'APP
// ============================================

struct MainView {
    title: SharedString,
    counter: usize,
    items: Vec<TodoItem>,
}

struct TodoItem {
    id: usize,
    text: String,
    completed: bool,
}

// ============================================
// IMPLEMENTAZIONE CON INSPECTOR
// ============================================

#[auto_inspector]
impl Render for MainView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .id("app_root")  // ← ROOT: ID per integrazione GPUI
            .flex()
            .flex_col()
            .size(px(800.0))
            .bg(rgb(0x1e1e1e))
            .text_color(rgb(0xffffff))
            .child(self.render_header(cx))
            .child(self.render_counter(cx))
            .child(self.render_todo_list(cx))
    }
}

impl MainView {
    fn render_header(&self, cx: &Context<Self>) -> impl IntoElement {
        div()
            .id("header")  // ← ID GPUI
            .flex()
            .items_center()
            .justify_between()
            .p_4()
            .bg(rgb(0x2d2d2d))
            .border_b_1()
            .border_color(rgb(0x404040))
            .child(
                div()
                    .id("app_title")  // ← ID GPUI
                    .text_xl()
                    .font_weight(FontWeight::BOLD)
                    .child(self.title.clone())
            )
            .child(
                div()
                    .id("version_badge")  // ← ID GPUI
                    .px_3()
                    .py_1()
                    .bg(rgb(0x007acc))
                    .rounded_md()
                    .text_sm()
                    .child("v1.0")
            )
    }

    fn render_counter(&self, cx: &Context<Self>) -> impl IntoElement {
        div()
            .id("counter_section")  // ← ID GPUI
            .flex()
            .items_center()
            .gap_4()
            .p_4()
            .child(
                div()
                    .id("counter_label")  // ← ID GPUI
                    .text_sm()
                    .text_color(rgb(0x808080))
                    .child("Counter:")
            )
            .child(
                div()
                    .id("counter_value")  // ← ID GPUI
                    .text_2xl()
                    .font_weight(FontWeight::BOLD)
                    .text_color(rgb(0x4ec9b0))
                    .child(format!("{}", self.counter))
            )
            .child(
                div()
                    .id("increment_button")  // ← ID GPUI
                    .px_4()
                    .py_2()
                    .bg(rgb(0x0e639c))
                    .rounded_md()
                    .cursor_pointer()
                    .hover(|style| style.bg(rgb(0x1177bb)))
                    .on_click(cx.listener(|this, _event, _window, cx| {
                        this.counter += 1;
                        cx.notify();
                    }))
                    .child("Increment")
            )
    }

    fn render_todo_list(&self, _cx: &Context<Self>) -> impl IntoElement {
        div()
            .id("todo_list")  // ← ID GPUI
            .flex()
            .flex_col()
            .gap_2()
            .p_4()
            .children(
                self.items
                    .iter()
                    .enumerate()
                    .map(|(index, item)| self.render_todo_item(item, index))
            )
    }

    fn render_todo_item(&self, item: &TodoItem, index: usize) -> impl IntoElement {
        let item_id = format!("todo_item_{}", item.id);  // ← ID dinamico con indice
        
        div()
            .id(item_id)  // ← ID GPUI dinamico
            .flex()
            .items_center()
            .gap_3()
            .p_3()
            .bg(if item.completed {
                rgb(0x2d2d2d)
            } else {
                rgb(0x3d3d3d)
            })
            .border_1()
            .border_color(rgb(0x404040))
            .rounded_md()
            .child(
                div()
                    .id(format!("checkbox_{}", item.id))  // ← ID GPUI
                    .size(px(20.0))
                    .rounded(px(4.0))
                    .border_2()
                    .border_color(if item.completed {
                        rgb(0x4ec9b0)
                    } else {
                        rgb(0x808080)
                    })
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(if item.completed { "✓" } else { "" })
            )
            .child(
                div()
                    .id(format!("todo_text_{}", item.id))  // ← ID GPUI
                    .flex_1()
                    .text_color(if item.completed {
                        rgb(0x808080)
                    } else {
                        rgb(0xffffff)
                    })
                    .line_through(if item.completed {
                        gpui::LineThroughStyle::Solid
                    } else {
                        gpui::LineThroughStyle::None
                    })
                    .child(item.text.clone())
            )
    }
}

// ============================================
// SETUP DELL'INSPECTOR
// ============================================

#[inspector_main]
fn main() {
    let app = Application::new();

    app.run(move |cx| {
        // Setup callback per integrazione GPUI
        setup_inspector_callbacks(cx);

        cx.spawn(|mut cx| async move {
            // Crea finestra principale
            let mut main_options = WindowOptions::default();
            main_options.window_bounds = Some(WindowBounds::Windowed(Bounds {
                origin: Point { x: px(100.0), y: px(100.0) },
                size: Size { width: px(800.0), height: px(600.0) },
            }));

            let main_window = cx.open_window(main_options, |window, cx| {
                cx.new(|cx| MainView {
                    title: "Todo App con Inspector".into(),
                    counter: 0,
                    items: vec![
                        TodoItem {
                            id: 1,
                            text: "Learn GPUI".to_string(),
                            completed: true,
                        },
                        TodoItem {
                            id: 2,
                            text: "Build inspector integration".to_string(),
                            completed: false,
                        },
                        TodoItem {
                            id: 3,
                            text: "Ship to production".to_string(),
                            completed: false,
                        },
                    ],
                })
            })?;

            // Crea finestra inspector
            let mut inspector_options = WindowOptions::default();
            inspector_options.window_bounds = Some(WindowBounds::Windowed(Bounds {
                origin: Point { x: px(920.0), y: px(100.0) },
                size: Size { width: px(400.0), height: px(600.0) },
            }));

            cx.open_window(inspector_options, |_, cx| {
                cx.new(InspectorView::new)
            })?;

            Ok::<_, anyhow::Error>(())
        })
        .detach();
    });
}

fn setup_inspector_callbacks(cx: &mut App) {
    // Configura callback quando un elemento è selezionato nell'inspector
    gpui_inspector_integration().set_element_selected_callback(
        |global_element_id, cx| {
            // Log della selezione
            println!("🔍 Element selected: {:?}", global_element_id);
            
            // Qui puoi aggiungere logica personalizzata:
            // - Evidenziare l'elemento nell'app
            // - Loggare informazioni dettagliate
            // - Sincronizzare con altri strumenti di debug
            
            // Esempio: Log albero dell'elemento (se disponibile)
            // if let Some(element) = cx.get_element_by_id(global_element_id) {
            //     println!("  Type: {:?}", element.type_name());
            //     println!("  Bounds: {:?}", element.bounds());
            // }
        }
    );
    
    println!("✅ Inspector callbacks configured");
}
```

## 🎯 Cosa Succede

### 1. Quando avvii l'app

```bash
cargo run
```

Vedrai:
```
[Inspector] Auto-inspection enabled
[Inspector] Target view set to: my_app::MainView
[Inspector] Installing render hook for: my_app::MainView (id: 123456789)
[Inspector] Registered view: my_app::MainView (id: 123456789)
✅ Inspector callbacks configured
```

### 2. Finestra principale

L'app mostra:
- Header con titolo "Todo App con Inspector"
- Sezione counter con valore e button "Increment"
- Todo list con 3 items

### 3. Finestra inspector

L'inspector mostra:

```
┌─────────────────────────┬─────────────────────────┐
│ ELEMENT TREE            │ PROPERTIES              │
│                         │                         │
│ ▼ ⬡ app_root           │ ┌─────────────────────┐ │
│   ⬡ header             │ │ GPUI ELEMENT ID     │ │
│     ⬡ app_title        │ │ app_root            │ │
│     ⬡ version_badge    │ └─────────────────────┘ │
│   ⬡ counter_section    │                         │
│     ⬡ counter_label    │ flex:                  │
│     ⬡ counter_value    │ flex_col:              │
│     ⬡ increment_button │ size: px(800.0)        │
│   ▼ ⬡ todo_list        │ bg: rgb(0x1e1e1e)      │
│     ⬡ todo_item_1      │ text_color: ...        │
│     ⬡ todo_item_2      │                         │
│     ⬡ todo_item_3      │                         │
└─────────────────────────┴─────────────────────────┘
```

## 🔍 Testing dell'Integrazione

### Test 1: Verifica ID GPUI

1. Apri l'inspector
2. Clicca su "app_root" nell'albero
3. Vedrai il box blu "GPUI ELEMENT ID" con valore "app_root"

### Test 2: Verifica callback

1. Clicca su qualsiasi elemento nell'albero
2. Guarda la console
3. Dovresti vedere: `🔍 Element selected: app_root`

### Test 3: Interazione con l'app

1. Clicca sul button "Increment"
2. Il counter aumenta
3. L'inspector si aggiorna automaticamente

### Test 4: Elementi dinamici

1. Osserva "todo_item_1", "todo_item_2", "todo_item_3"
2. Questi hanno ID dinamici basati sull'indice
3. Vengono catturati correttamente dall'inspector

## 🛠️ Comandi Utili

### Build e run

```bash
# Build tutto
cargo build

# Run l'app
cargo run

# Run con logging dettagliato
RUST_LOG=debug cargo run
```

### Test dell'inspector

```bash
# Verifica che l'inspector compili
cargo check -p gpui-inspector

# Test delle macro
cargo test -p gpui-inspector-macros
```

## 📊 Output Console

Esempio di output quando interagisci con l'inspector:

```
[Inspector] Auto-inspection enabled
[Inspector] Installing render hook for: my_app::MainView (id: 123456789)
✅ Inspector callbacks configured
Render MainView:
div(id: 0)
.id("app_root")
.flex("")
.flex_col("")
.size("px(800.0)")
  div(id: 1)
  .id("header")
  .flex("")
  .items_center("")
    div(id: 2)
    .id("app_title")
    .text_xl("")
    div(id: 3)
    .id("version_badge")
    .px_3("")
  div(id: 4)
  .id("counter_section")
  .flex("")
  .items_center("")
    div(id: 5)
    .id("counter_label")
    .text_sm("")
    div(id: 6)
    .id("counter_value")
    .text_2xl("")
    div(id: 7)
    .id("increment_button")
    .px_4("")
  div(id: 8)
  .id("todo_list")
  .flex("")
  .flex_col("")
    div(id: 9)
    .id("todo_item_1")
    .flex("")
    .items_center("")
    ...
```

Quando selezioni un elemento:

```
🔍 Element selected: app_root
🔍 Element selected: header
🔍 Element selected: increment_button
```

## 🎨 Bonus: Stili Avanzati

Puoi anche aggiungere stili condizionali basati sulla selezione:

```rust
impl MainView {
    fn render_with_highlight(&self, cx: &mut Context<Self>) -> impl IntoElement {
        // Verifica se questo elemento è selezionato nell'inspector
        let is_selected = gpui_inspector::hooks::is_element_selected("app_root");
        
        div()
            .id("app_root")
            .border_1()
            .border_color(if is_selected {
                rgb(0x4ec9b0)  // Blu quando selezionato
            } else {
                rgb(0x404040)  // Grigio normalmente
            })
            .child(/* ... */)
    }
}
```

## 🚀 Next Steps

1. **Prova tu stesso**: Copia il codice e crea un nuovo progetto
2. **Sperimenta**: Aggiungi più ID ai tuoi elementi
3. **Estendi**: Aggiungi callback personalizzati per il tuo use case
4. **Contribuisci**: Migliora l'integrazione con PR!

---

Buon debugging! 🐛🔍