use gpui::{App, AppContext, Entity, Render};
use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// Global flag to enable automatic inspection
static AUTO_INSPECTION_ENABLED: AtomicBool = AtomicBool::new(false);

/// GPUI Inspector integration structure
pub struct GpuiInspectorIntegration {
    /// Mapping from local element IDs to GPUI element IDs (as strings for now)
    element_id_mapping: Arc<Mutex<HashMap<u64, String>>>,
    /// Mapping from local element IDs to GPUI element paths (as strings for now)
    element_path_mapping: Arc<Mutex<HashMap<u64, String>>>,
    /// Currently selected element ID
    selected_element_id: Arc<Mutex<Option<String>>>,
    /// The main window handle to refresh when selection changes
    main_window: Arc<Mutex<Option<gpui::AnyWindowHandle>>>,
    /// Callback when an element is selected in the inspector
    on_element_selected: Arc<Mutex<Option<Box<dyn Fn(String, &mut App) + Send + Sync>>>>,
}

impl GpuiInspectorIntegration {
    pub fn new() -> Self {
        Self {
            element_id_mapping: Arc::new(Mutex::new(HashMap::new())),
            element_path_mapping: Arc::new(Mutex::new(HashMap::new())),
            selected_element_id: Arc::new(Mutex::new(None)),
            main_window: Arc::new(Mutex::new(None)),
            on_element_selected: Arc::new(Mutex::new(None)),
        }
    }

    /// Set the main window handle
    pub fn set_main_window(&self, handle: gpui::AnyWindowHandle) {
        *self.main_window.lock() = Some(handle);
    }

    /// Register a mapping between local ID and GPUI element ID (as string)
    pub fn register_element_mapping(&self, local_id: u64, element_id: String) {
        self.element_id_mapping.lock().insert(local_id, element_id);
    }

    /// Register a mapping between local ID and GPUI element path (as string)
    pub fn register_element_path(&self, local_id: u64, path: String) {
        self.element_path_mapping.lock().insert(local_id, path);
    }

    /// Called when an element is selected in the inspector
    pub fn on_element_selected(&self, local_id: u64, cx: &mut App) {
        if let Some(inspector_id) = self.element_id_mapping.lock().get(&local_id).cloned() {
            // Update global selected ID
            *self.selected_element_id.lock() = Some(inspector_id.clone());
            
            // Trigger a refresh on the main window if available
            if let Some(handle) = *self.main_window.lock() {
                let _ = cx.update_window(handle, |_, w, _| {
                    w.refresh();
                });
            }

            // Call custom callback if set
            if let Some(callback) = self.on_element_selected.lock().as_ref() {
                callback(inspector_id, cx);
            }
        }
    }

    /// Check if an element is currently selected
    pub fn is_selected(&self, element_id: &str) -> bool {
        self.selected_element_id.lock().as_deref() == Some(element_id)
    }

    /// Set a callback for element selection
    pub fn set_element_selected_callback<F>(&self, callback: F)
    where
        F: Fn(String, &mut App) + Send + Sync + 'static,
    {
        *self.on_element_selected.lock() = Some(Box::new(callback));
    }

    /// Get the GPUI element ID (as string) for a local element ID
    pub fn get_element_id(&self, local_id: u64) -> Option<String> {
        self.element_id_mapping.lock().get(&local_id).cloned()
    }

    /// Get the GPUI element path (as string) for a local element ID
    pub fn get_element_path(&self, local_id: u64) -> Option<String> {
        self.element_path_mapping.lock().get(&local_id).cloned()
    }

    /// Clear all element mappings
    pub fn clear_mappings(&self) {
        self.element_id_mapping.lock().clear();
        self.element_path_mapping.lock().clear();
    }
}

/// Global instance of GPUI inspector integration
static GPUI_INSPECTOR_INTEGRATION: once_cell::sync::Lazy<GpuiInspectorIntegration> =
    once_cell::sync::Lazy::new(|| GpuiInspectorIntegration::new());

/// Access the GPUI inspector integration
pub fn gpui_inspector_integration() -> &'static GpuiInspectorIntegration {
    &GPUI_INSPECTOR_INTEGRATION
}

/// Register an element with GPUI Inspector (called from generated code)
pub fn register_element_with_gpui(local_id: u64, element_id: String) {
    GPUI_INSPECTOR_INTEGRATION.register_element_mapping(local_id, element_id);
}

/// Register an element path with GPUI Inspector (called from generated code)
pub fn register_element_path_with_gpui(local_id: u64, path: String) {
    GPUI_INSPECTOR_INTEGRATION.register_element_path(local_id, path);
}

/// The target view type name to capture (first non-inspector view)
static TARGET_VIEW: once_cell::sync::Lazy<Arc<Mutex<Option<String>>>> =
    once_cell::sync::Lazy::new(|| Arc::new(Mutex::new(None)));

/// Global registry of views that should be inspected
static VIEW_REGISTRY: once_cell::sync::Lazy<Arc<Mutex<Vec<ViewInfo>>>> =
    once_cell::sync::Lazy::new(|| Arc::new(Mutex::new(Vec::new())));

struct ViewInfo {
    view_id: u64,
    type_name: String,
}

/// Enable automatic inspection for all views
/// This is called by the inspector_main macro
pub fn enable_auto_inspection() {
    AUTO_INSPECTION_ENABLED.store(true, Ordering::Relaxed);
    println!("[Inspector] Auto-inspection enabled");
}

/// Check if auto-inspection is enabled
pub fn is_auto_inspection_enabled() -> bool {
    AUTO_INSPECTION_ENABLED.load(Ordering::Relaxed)
}

/// Register a view for inspection
/// This is called automatically by the inspector_main macro
pub fn register_view<V: Render + 'static>(view: &Entity<V>) {
    let type_name = std::any::type_name::<V>().to_string();
    let view_id = view.entity_id().as_u64();

    // Set the first non-InspectorView as the target
    if !type_name.contains("InspectorView") {
        let mut target = TARGET_VIEW.lock();
        if target.is_none() {
            *target = Some(type_name.clone());
            println!("[Inspector] Target view set to: {}", type_name);
        }
    }

    VIEW_REGISTRY.lock().push(ViewInfo {
        view_id,
        type_name: type_name.clone(),
    });

    println!(
        "[Inspector] Registered view: {} (id: {})",
        type_name, view_id
    );
}

/// Install a render hook on a view
/// This intercepts render calls and captures the element tree
pub fn install_render_hook<V: Render + 'static>(view: &Entity<V>) {
    let type_name = std::any::type_name::<V>().to_string();
    let view_id = view.entity_id().as_u64();

    println!(
        "[Inspector] Installing render hook for: {} (id: {})",
        type_name, view_id
    );

    // Note: Since we can't actually hook into GPUI's render system at runtime,
    // we rely on the macro-based approach to inject inspection code.
    // This function serves as a marker that a view should be inspected.
    register_view(view);
}

/// Check if a view is registered for inspection
pub fn is_view_registered(view_id: u64) -> bool {
    VIEW_REGISTRY
        .lock()
        .iter()
        .any(|info| info.view_id == view_id)
}

/// Get the type name of a registered view
pub fn get_view_type_name(view_id: u64) -> Option<String> {
    VIEW_REGISTRY
        .lock()
        .iter()
        .find(|info| info.view_id == view_id)
        .map(|info| info.type_name.clone())
}

/// Clear all registered views
pub fn clear_registry() {
    VIEW_REGISTRY.lock().clear();
}

/// Get count of registered views
pub fn registered_view_count() -> usize {
    VIEW_REGISTRY.lock().len()
}

/// Check if a type should be captured (is it the target view?)
pub fn should_capture_type(type_name: &str) -> bool {
    // Never capture the inspector itself
    if type_name.contains("InspectorView") {
        return false;
    }

    // Capture only MainView or Root (not child components like Text, Button)
    type_name.contains("MainView") || type_name.contains("Root")
}

/// Check if an element is currently selected by its ID
pub fn is_selected(element_id: &str) -> bool {
    GPUI_INSPECTOR_INTEGRATION.is_selected(element_id)
}

/// Handle element selection from the inspector UI
pub fn handle_element_selection(local_id: u64, cx: &mut App) {
    GPUI_INSPECTOR_INTEGRATION.on_element_selected(local_id, cx);
}
