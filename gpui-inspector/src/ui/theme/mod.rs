// Theme module: UI styling and color definitions

use gpui::rgb;

/// Color palette for the inspector UI (Zed-inspired dark theme)
pub struct InspectorTheme;

impl InspectorTheme {
    // Background colors
    pub fn bg_primary() -> gpui::Rgba {
        rgb(0x282c34) // Dark background
    }

    pub fn bg_secondary() -> gpui::Rgba {
        rgb(0x21252b) // Slightly darker for headers
    }

    pub fn bg_hover() -> gpui::Rgba {
        rgb(0x2f3440) // Hover state
    }

    pub fn bg_selected() -> gpui::Rgba {
        rgb(0x2b4f6a) // Blue selection
    }

    // Border colors
    pub fn border() -> gpui::Rgba {
        rgb(0x181a1f)
    }

    // Text colors
    pub fn text_primary() -> gpui::Rgba {
        rgb(0xabb2bf) // Main text
    }

    pub fn text_secondary() -> gpui::Rgba {
        rgb(0x8a8f98) // Secondary/muted text
    }

    pub fn text_highlight() -> gpui::Rgba {
        rgb(0xe5e9f0) // Highlighted text
    }

    pub fn text_muted() -> gpui::Rgba {
        rgb(0x5c6370) // Disabled/placeholder text
    }

    // Accent colors
    pub fn accent_blue() -> gpui::Rgba {
        rgb(0x61afef) // For icons, links
    }

    pub fn accent_purple() -> gpui::Rgba {
        rgb(0xc678dd) // For keys, properties
    }

    pub fn accent_green() -> gpui::Rgba {
        rgb(0x98c379) // For values, success
    }
}
