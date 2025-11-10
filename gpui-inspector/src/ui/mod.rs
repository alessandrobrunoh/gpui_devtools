// UI module: All inspector UI-related code

pub mod theme;
pub mod views;

// Re-export the main inspector view
pub use views::inspector_view::InspectorView;
