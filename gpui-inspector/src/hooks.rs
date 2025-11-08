use gpui::{Entity, Render};
use parking_lot::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// Global flag to enable automatic inspection
static AUTO_INSPECTION_ENABLED: AtomicBool = AtomicBool::new(false);

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

    println!("[Inspector] Registered view: {} (id: {})", type_name, view_id);
}

/// Install a render hook on a view
/// This intercepts render calls and captures the element tree
pub fn install_render_hook<V: Render + 'static>(view: &Entity<V>) {
    let type_name = std::any::type_name::<V>().to_string();
    let view_id = view.entity_id().as_u64();

    println!("[Inspector] Installing render hook for: {} (id: {})", type_name, view_id);

    // Note: Since we can't actually hook into GPUI's render system at runtime,
    // we rely on the macro-based approach to inject inspection code.
    // This function serves as a marker that a view should be inspected.
    register_view(view);
}

/// Check if a view is registered for inspection
pub fn is_view_registered(view_id: u64) -> bool {
    VIEW_REGISTRY.lock().iter().any(|info| info.view_id == view_id)
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
