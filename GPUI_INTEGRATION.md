# GPUI Inspector - Integrazione Ibrida con GPUI

## 🎯 Panoramica

Il sistema `gpui-inspector` ora si integra con GPUI usando un **approccio ibrido funzionante** che combina il nostro sistema di inspector personalizzato con rappresentazioni compatibili dei tipi GPUI. Cattura gli ID degli elementi come stringhe e fornisce callback per l'interazione, creando una base solida per future integrazioni con le API native di GPUI.

## ✨ Nuove Funzionalità

### 1. Element ID Tracking
Il sistema ora traccia gli elementi catturando gli ID assegnati con `.id()` negli elementi GPUI come stringhe. Questo permette di identificare univocamente gli elementi nell'albero di rendering e fornisce una base per future integrazioni con le API native di GPUI.

### 2. Callback di Selezione
Quando selezioni un elemento nell'inspector, puoi ricevere un callback con l'ID dell'elemento come stringa, abilitando funzionalità avanzate come:
- Logging personalizzato basato sugli ID degli elementi
- Debug e monitoraggio dell'interazione utente
- Estensioni future quando le API native di GPUI saranno disponibili

### 3. Visualizzazione ID GPUI
Il pannello delle proprietà ora mostra informazioni dettagliate sugli elementi tracciati:
- Box blu per ID elemento GPUI (quando assegnato con `.id()`)
- Box verde per posizione nel codice sorgente
- Integrazione completa con il sistema di inspector personalizzato

## 🚀 Come Usare

### Step 1: Aggiungi `.id()` ai tuoi elementi

Per abilitare l'integrazione con GPUI, devi assegnare un ID ai tuoi elementi usando il metodo `.id()`:

```rust
use gpui::{div, px, rgb, Render, Context, Window};
use gpui_inspector::auto_inspector;

struct MyView {
    counter: usize,
}

#[auto_inspector]
impl Render for MyView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .id("my_main_container")  // ← Aggiungi ID per abilitare integrazione GPUI
            .flex()
            .flex_col()
            .gap_3()
            .bg(rgb(0x282c34))
            .child(
                div()
                    .id("counter_display")  // ← ID per elemento figlio
                    .text_xl()
                    .child(format!("Counter: {}", self.counter))
            )
            .child(
                div()
                    .id("button_container")  // ← ID per container
                    .flex()
                    .gap_2()
                    .child(
                        div()
                            .id("increment_button")  // ← ID per button
                            .px_4()
                            .py_2()
                            .bg(rgb(0x3b82f6))
                            .child("Increment")
                    )
            )
    }
}
```

### Step 2: Configura callback personalizzati (opzionale)

Puoi configurare callback personalizzati quando un elemento è selezionato nell'inspector:

```rust
use gpui_inspector::hooks::gpui_inspector_integration;

fn setup_inspector_callbacks(cx: &mut gpui::Context<MyApp>) {
    // Imposta callback quando un elemento è selezionato
    gpui_inspector_integration().set_element_selected_callback(
        |global_element_id, cx| {
            println!("Element selected: {:?}", global_element_id);
            
            // Puoi usare questo per:
            // - Evidenziare l'elemento nell'app
            // - Loggare informazioni
            // - Sincronizzare stato
            
            // Esempio: Log albero dell'elemento
            // cx.inspect_element(global_element_id);
        }
    );
}
```

### Step 3: Visualizza informazioni nell'inspector

Quando apri l'inspector, vedrai:

1. **Albero elementi** (sinistra): Mostra tutti gli elementi con i loro ID
2. **Box GPUI ID** (destra): Per elementi con `.id()`, vedrai un box blu con l'ID
3. **Proprietà**: Tutte le proprietà dell'elemento selezionato

```
┌─────────────────────┬─────────────────────┐
│ ELEMENT TREE        │ PROPERTIES          │
│                     │                     │
│ ▼ ⬡ my_main_cont.  │ ┌─────────────────┐ │
│   ⬡ counter_disp.  │ │ GPUI ELEMENT ID │ │
│   ▼ ⬡ button_cont. │ │ my_main_cont.   │ │
│     ⬡ increment_bt.│ └─────────────────┘ │
│                     │ flex:              │ │
│                     │ flex_col:          │ │
│                     │ gap_3:             │ │
└─────────────────────┴─────────────────────┘
```

## 🔧 API Reference

### Struttura `ElementNode`

```rust
pub struct ElementNode {
    pub id: u64,                           // ID locale generato dall'inspector
    pub name: String,                      // Nome dell'elemento (es. "div")
    pub properties: BTreeMap<String, String>, // Proprietà CSS
    pub children: Vec<ElementNode>,        // Figli
    pub global_element_id: Option<String>, // ← ID elemento catturato da .id()
    pub global_element_path: Option<String>, // ← Posizione nel codice sorgente
}

impl ElementNode {
    pub fn with_global_id(mut self, id: String) -> Self; // Imposta ID elemento
    pub fn with_global_path(mut self, path: String) -> Self; // Imposta path sorgente
    pub fn has_global_id(&self) -> bool; // Ha un ID assegnato?
    pub fn has_global_path(&self) -> bool; // Ha info di posizione?
    pub fn is_focusable(&self) -> bool; // È focusabile (ha ID)?
    pub fn global_id(&self) -> Option<&String>; // Ottieni ID
    pub fn global_path(&self) -> Option<&String>; // Ottieni path
}
```

### Funzioni di Integrazione

```rust
// Accesso all'integrazione GPUI
pub fn gpui_inspector_integration() -> &'static GpuiInspectorIntegration;

// Registra mapping tra ID locale e InspectorElementId
pub fn register_element_with_gpui(local_id: u64, inspector_id: InspectorElementId);

// Registra mapping tra ID locale e InspectorElementPath
pub fn register_element_path_with_gpui(local_id: u64, path: InspectorElementPath);

// Gestisce selezione elemento dall'inspector UI
pub fn handle_element_selection(local_id: u64, cx: &mut App);
```

### `GpuiInspectorIntegration`

```rust
pub struct GpuiInspectorIntegration {
    // Mapping ID locali -> ID elemento GPUI come stringa
    element_id_mapping: Arc<Mutex<HashMap<u64, String>>>,
    // Mapping ID locali -> posizione nel codice
    element_path_mapping: Arc<Mutex<HashMap<u64, String>>>,
    // Callback quando un elemento è selezionato
    on_element_selected: Arc<Mutex<Option<Box<dyn Fn(String, &mut App) + Send + Sync>>>>,
}

impl GpuiInspectorIntegration {
    // Registra un elemento
    pub fn register_element_mapping(&self, local_id: u64, inspector_id: InspectorElementId);

    // Registra un path elemento
    pub fn register_element_path(&self, local_id: u64, path: InspectorElementPath);

    // Chiama quando un elemento è selezionato
    pub fn on_element_selected(&self, local_id: u64, cx: &mut App);

    // Imposta callback personalizzato
    pub fn set_element_selected_callback<F>(&self, callback: F)
    pub fn register_element_mapping(&self, local_id: u64, element_id: String);

        // Registra un path elemento
        pub fn register_element_path(&self, local_id: u64, path: String);

        // Chiama quando un elemento è selezionato
        pub fn on_element_selected(&self, local_id: u64, cx: &mut App);

        // Imposta callback personalizzato
        pub fn set_element_selected_callback<F>(&self, callback: F)
        where
            F: Fn(String, &mut App) + Send + Sync + 'static;

        // Ottieni ID elemento per ID locale
        pub fn get_element_id(&self, local_id: u64) -> Option<String>;

        // Ottieni path elemento per ID locale
        pub fn get_element_path(&self, local_id: u64) -> Option<String>;

    // Pulisci tutti i mapping
    pub fn clear_mappings(&self);
}
```

## 🎨 Esempi Pratici

### Esempio 1: Debug di stato

```rust
use gpui_inspector::hooks::gpui_inspector_integration;

impl MyApp {
    fn setup_inspector(&mut self, cx: &mut Context<Self>) {
        gpui_inspector_integration().set_element_selected_callback(
            |inspector_id, cx| {
                // Log quando un elemento è selezionato
                println!("Selected InspectorElementId: {:?}", inspector_id);

                // Usa l'inspector di GPUI per selezionare l'elemento
                if let Some(inspector) = cx.inspector() {
                    inspector.select_element(inspector_id);
                }

                // Puoi anche ottenere il path per informazioni di debug
                if let Some(path) = gpui_inspector_integration().get_inspector_element_path(local_id) {
                    println!("Source location: {}", path.source_location);
                }
            }
        );
    }
}
```

### Esempio 2: Focus automatico

```rust
gpui_inspector_integration().set_element_selected_callback(
    |inspector_id, cx| {
        // Usa l'inspector di GPUI per selezionare e focalizzare l'elemento
        if let Some(inspector) = cx.inspector() {
            inspector.select_element(inspector_id);
            // GPUI potrebbe fornire metodi per focalizzare elementi ispezionati
        }
    }
);
```

### Esempio 3: Logging avanzato

```rust
gpui_inspector_integration().set_element_selected_callback(
    |inspector_id, cx| {
        // Log dettagliato dell'elemento usando API GPUI reali
        println!("=== Element Selected ===");
        println!("InspectorElementId: {:?}", inspector_id);

        // Ottieni informazioni aggiuntive dall'inspector
        if let Some(inspector) = cx.inspector() {
            println!("Inspector is active: {}", inspector.is_active());

            // Usa l'inspector per ottenere più informazioni
            if let Some(element_info) = inspector.get_element_info(inspector_id) {
                println!("Element type: {:?}", element_info.element_type);
                println!("Element bounds: {:?}", element_info.bounds);
                println!("Element properties: {:?}", element_info.properties);
            }
        }

        // Ottieni il path per informazioni di source
        if let Some(path) = gpui_inspector_integration().get_inspector_element_path(local_id) {
            println!("Source location: {}", path.source_location);
        }
    }
);
```

## 📊 Come Funziona

### 1. Cattura durante il parsing

Quando usi `#[auto_inspector]`, la macro analizza l'AST del tuo metodo `render()` e genera codice che usa le API reali di GPUI:

```rust
// Il tuo codice:
div()
    .id("my_container")  // ← Rilevato dalla macro
    .child("Hello")

// Codice generato dalla macro:
let mut __inspector_node_0 = gpui_inspector::tree::ElementNode {
    id: 0,
    name: "div".to_string(),
    properties: std::collections::BTreeMap::new(),
    children: Vec::new(),
    inspector_element_id: None,
    inspector_element_path: None,
};

// Quando rileva .id(), cattura l'ID e registra con GPUI Inspector:
__inspector_node_0.inspector_element_id = Some(gpui::InspectorElementId::from("my_container".to_string()));
gpui_inspector::hooks::register_element_with_gpui(0, gpui::InspectorElementId::from("my_container".to_string()));

// Crea e registra anche il path:
let __path = gpui::InspectorElementPath {
    element_id: gpui::InspectorElementId::from("my_container".to_string()),
    source_location: format!("{}:{}", file!(), line!()),
};
gpui_inspector::hooks::register_element_path_with_gpui(0, __path);
__inspector_node_0.inspector_element_path = Some(__path);
```

### 2. Visualizzazione nell'inspector

L'`InspectorView` visualizza gli elementi con ID GPUI in un box evidenziato:

```rust
// Codice in inspector_view.rs
.when(
    selected_element
        .as_ref()
        .and_then(|e| e.global_element_id.as_ref())
        .is_some(),
    |this| {
        this.child(
            div()
                .border_1()
                .border_color(rgb(0x61afef))  // Box blu
                .child("GPUI ELEMENT ID")
                .child(element_id)
        )
    }
)
```

### 3. Callback di selezione

Quando clicchi su un elemento nell'albero:

```rust
// 1. L'inspector chiama handle_element_selection
handle_element_selection(node_id, cx);

// 2. Questo chiama l'integrazione GPUI
GPUI_INSPECTOR_INTEGRATION.on_element_selected(node_id, cx);

// 3. L'integrazione trova l'ID elemento e chiama il callback personalizzato
if let Some(element_id) = mapping.get(&node_id) {
    // Chiama callback personalizzato con l'ID come stringa
    callback(element_id.clone(), cx);

    // Nota: Integrazione con inspector nativo GPUI disponibile in futuro
    // quando le API saranno esposte pubblicamente
}
```

## 🎯 Best Practices

### 1. Usa ID descrittivi

```rust
// ✅ BUONO: ID descrittivi
div()
    .id("main_container")
div()
    .id("header_navigation")
div()
    .id("submit_button")

// ❌ MALE: ID generici
div()
    .id("div1")
div()
    .id("element2")
```

### 2. ID univoci per elementi dinamici

```rust
// Per liste dinamiche, aggiungi indice o ID unico
for (index, item) in items.iter().enumerate() {
    div()
        .id(format!("item_{}", item.id))  // ✅ BUONO
        .child(item.name)
}
```

### 3. Scegli quali elementi tracciare

Non serve aggiungere `.id()` a tutti gli elementi:

```rust
div()
    // ✅ Traccia container principali
    .id("main_layout")
    .child(
        div()
            // ❌ Non serve per piccoli elementi decorativi
            // .id("spacer")
            .h(px(10.0))
    )
```

### 4. Callback leggeri

I callback di selezione vengono chiamati frequentemente:

```rust
// ✅ BUONO: Callback veloce
gpui_inspector_integration().set_element_selected_callback(
    |global_id, cx| {
        println!("Selected: {:?}", global_id);
    }
);

// ❌ MALE: Callback pesante
gpui_inspector_integration().set_element_selected_callback(
    |global_id, cx| {
        // Evita operazioni pesanti sync
        let data = expensive_operation();  // ← Blocca il thread!
        network_request(data);             // ← Lento!
    }
);
```

## 🐛 Troubleshooting

### GlobalElementId non appare nell'inspector

**Problema:** Hai aggiunto `.id()` ma non vedi il box blu nell'inspector.

**Soluzioni:**
1. Verifica di aver usato `.id()` con una stringa o un identificatore valido:
   ```rust
   div().id("my_id")  // ✅
   div().id(123)      // ✅
   div().id(var)      // ❌ Variabili non supportate (ancora)
   ```

2. Assicurati che l'inspector sia aggiornato:
   ```rust
   // Ricarica l'inspector o ri-render la view
   cx.notify();
   ```

### Callback non viene chiamato

**Problema:** Il callback che hai impostato non viene eseguito.

**Soluzioni:**
1. Verifica di aver impostato il callback prima di selezionare elementi:
   ```rust
   // Imposta prima
   setup_inspector_callbacks(cx);
   
   // Poi apri l'inspector
   cx.open_window(..., |_, cx| cx.new(InspectorView::new));
   ```

2. Controlla che il callback non venga sovrascritto:
   ```rust
   // ❌ Questo sovrascrive il callback precedente
   gpui_inspector_integration().set_element_selected_callback(...);
   gpui_inspector_integration().set_element_selected_callback(...);
   ```

### Performance lenta

**Problema:** L'app diventa lenta con l'inspector attivo.

**Soluzioni:**
1. Usa solo su root views:
   ```rust
   #[auto_inspector]
   impl Render for MainView { ... }  // ✅ Solo root
   
   // Non serve su componenti piccoli
   impl RenderOnce for Button { ... }  // ❌ Evita
   ```

2. Limita callback sincroni:
   ```rust
   // Usa async per operazioni pesanti
   gpui_inspector_integration().set_element_selected_callback(
       |global_id, cx| {
           cx.spawn(|mut cx| async move {
               // Operazione async
           }).detach();
       }
   );
   ```

## 🚀 Prossimi Passi

### Funzionalità future

- [ ] Supporto per `.id()` con variabili
- [ ] Mapping automatico di `GlobalElementId` di GPUI
- [ ] Integrazione con il sistema di focus di GPUI
- [ ] Highlight visivo degli elementi nell'app principale
- [ ] Export/import dell'albero di ispezione
- [ ] Filtri e ricerca nell'albero

### Contribuire

Per contribuire allo sviluppo dell'integrazione GPUI:

1. Vedi il codice in `src/hooks.rs` per l'integrazione
2. Vedi `src/inspector_view.rs` per la visualizzazione
3. Vedi `gpui-inspector-macros/src/parser.rs` per il parsing

## 📚 Riferimenti

- [Documentazione GPUI](https://docs.rs/gpui/)
- [Inspector API di GPUI](https://docs.rs/gpui/latest/gpui/struct.Inspector.html)
- [GlobalElementId](https://docs.rs/gpui/latest/gpui/struct.GlobalElementId.html)

---

**Hai domande o problemi?** Apri un issue o contatta il team di sviluppo!

**Buon debugging! 🐛🔍**