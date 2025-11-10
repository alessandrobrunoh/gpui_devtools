# Implementazione Global Hooks per GPUI Inspector

Questo documento descrive come modificare GPUI per supportare l'inspector automaticamente senza bisogno di aggiungere `#[auto_inspector]` su ogni componente.

## Obiettivo

Far funzionare l'inspector catturando automaticamente tutti i render di tutti i componenti, permettendo a progetti grandi come Zed di usare l'inspector senza modificare il codice esistente.

## Modifiche Necessarie a GPUI

### 1. Creare il modulo hooks (gpui/src/hooks.rs)

```rust
use std::sync::Arc;
use parking_lot::Mutex;

/// Tipo per la callback che viene chiamata quando un elemento viene renderizzato
/// - primo parametro: nome del componente (tipo)
/// - secondo parametro: riferimento all'elemento renderizzato (come Any per type erasure)
type RenderHook = Arc<dyn Fn(&str, &dyn std::any::Any) + Send + Sync>;

/// Storage globale per tutti gli hook registrati
static RENDER_HOOKS: Mutex<Vec<RenderHook>> = Mutex::new(Vec::new());

/// Registra un hook globale che verrà chiamato ogni volta che un elemento viene renderizzato
pub fn register_render_hook<F>(hook: F) 
where 
    F: Fn(&str, &dyn std::any::Any) + Send + Sync + 'static 
{
    RENDER_HOOKS.lock().push(Arc::new(hook));
}

/// Notifica tutti gli hook registrati che un elemento è stato renderizzato
pub fn notify_render(component_name: &str, element: &dyn std::any::Any) {
    let hooks = RENDER_HOOKS.lock();
    for hook in hooks.iter() {
        hook(component_name, element);
    }
}

/// Rimuove tutti gli hook (utile per testing)
pub fn clear_hooks() {
    RENDER_HOOKS.lock().clear();
}
```

### 2. Registrare il modulo in gpui/src/lib.rs

```rust
// Aggiungi questa riga con gli altri moduli
pub mod hooks;
```

### 3. Modificare i trait Render e RenderOnce

Trova le implementazioni di `Render` e `RenderOnce` in GPUI e aggiungi le chiamate agli hook.

**In gpui/src/element.rs o dove sono definiti i trait:**

```rust
// Nel trait Render
pub trait Render: Sized + 'static {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let element = self.render_impl(cx);
        
        // Notifica agli hook
        let type_name = std::any::type_name::<Self>();
        crate::hooks::notify_render(type_name, &element);
        
        element
    }
    
    // Metodo da implementare dai componenti
    fn render_impl(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement;
}

// Simile per RenderOnce
pub trait RenderOnce: 'static {
    fn render_once(self, cx: &mut WindowContext) -> impl IntoElement {
        let element = self.render_once_impl(cx);
        
        // Notifica agli hook
        let type_name = std::any::type_name::<Self>();
        crate::hooks::notify_render(type_name, &element);
        
        element
    }
    
    fn render_once_impl(self, cx: &mut WindowContext) -> impl IntoElement;
}
```

**Nota:** La posizione esatta dipende da come GPUI implementa internamente questi trait. Potrebbe essere necessario:
- Modificare `ViewContext::render()` 
- Modificare `Element::draw()` 
- Aggiungere hook nel sistema di rendering di GPUI

### 4. Uso nell'Inspector

**In gpui-inspector/src/lib.rs:**

```rust
use once_cell::sync::Lazy;

/// Inizializza l'inspector registrando l'hook globale
pub fn initialize_global_inspector() {
    gpui::hooks::register_render_hook(|component_name, element_any| {
        // Prova a convertire l'elemento in un tipo conosciuto
        // Questo richiede che GPUI esponga il tipo Element o un trait comune
        
        // Per ora, usa reflection o type_name per identificare il componente
        let node = create_element_node_from_any(component_name, element_any);
        
        // Salva nel global state
        let mut all_trees = state::ALL_TREES.lock();
        all_trees.insert(
            component_name.to_string(),
            state::StoredTree {
                element_tree: Some(node),
                version: all_trees.get(component_name)
                    .map(|t| t.version + 1)
                    .unwrap_or(0),
            },
        );
        
        // Notifica i listener
        state::notify_tree_changed();
    });
}

/// Converte un elemento Any in un ElementNode
/// Questo richiede che GPUI esponga un modo per ispezionare gli elementi
fn create_element_node_from_any(
    component_name: &str, 
    element: &dyn std::any::Any
) -> tree::ElementNode {
    // Implementazione dipende da come GPUI espone gli elementi
    // Potrebbe richiedere un nuovo trait in GPUI:
    // pub trait Inspectable {
    //     fn to_element_node(&self) -> ElementNode;
    // }
    
    tree::ElementNode {
        name: component_name.to_string(),
        children: vec![],
        attributes: Default::default(),
    }
}
```

## Sfide e Considerazioni

### 1. Type Erasure
Il problema principale è che con `&dyn Any` perdiamo le informazioni sul tipo. Soluzioni:
- Aggiungere un trait `Inspectable` in GPUI che tutti gli elementi implementano
- Usare una rappresentazione intermedia serializzabile
- Modificare GPUI per mantenere metadati di debug sugli elementi

### 2. Performance
Gli hook globali potrebbero impattare le performance. Soluzioni:
- Usare feature flag per abilitare/disabilitare gli hook in build di release
- Usare atomic flags per check veloci prima di chiamare gli hook
- Limitare la profondità di ispezione

### 3. Thread Safety
Gli hook devono essere thread-safe. Soluzioni:
- Usare `Mutex` o `RwLock` per lo stato condiviso
- Considerare l'uso di channel per comunicazione asincrona
- Minimizzare il tempo di lock

## Alternativa: Proc Macro Globale

Un'alternativa più semplice potrebbe essere modificare `gpui_macros` per applicare automaticamente l'ispezione:

```rust
// In gpui_macros
#[proc_macro_attribute]
pub fn derive_render(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemImpl);
    
    // Se il feature "inspector" è abilitato, aggiungi automaticamente
    // il codice di ispezione in tutte le implementazioni di Render
    
    #[cfg(feature = "inspector")]
    {
        // Aggiungi codice di auto-inspection
    }
    
    quote!(#input).into()
}
```

Questo richiederebbe che tutti i componenti usino una derive macro, ma sarebbe meno invasivo che modificare il core di GPUI.

## Prossimi Passi

1. Decidere quale approccio usare (hook globali vs proc macro)
2. Implementare il sistema scelto
3. Testare con l'applicazione di esempio
4. Verificare le performance
5. Documentare l'API pubblica per gli utenti dell'inspector
