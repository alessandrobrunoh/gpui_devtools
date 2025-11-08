use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn, ItemImpl};

mod parser;
mod rewriter;

#[proc_macro_attribute]
pub fn inspector(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemImpl);

    let trait_path = match &input.trait_ {
        Some((_, path, _)) => path,
        None => return quote! { #input }.into(),
    };

    let is_render = trait_path.segments.iter().any(|s| s.ident == "Render");
    let is_render_once = trait_path.segments.iter().any(|s| s.ident == "RenderOnce");

    if !is_render && !is_render_once {
        // Not a Render or RenderOnce impl, so just return the original code
        return quote! { #input }.into();
    }

    let mut output = input.clone();

    for item in &mut output.items {
        if let syn::ImplItem::Fn(method) = item {
            if method.sig.ident == "render" {
                let original_method_body = &method.block;
                let type_name = if let syn::Type::Path(type_path) = &*output.self_ty {
                    let segments = &type_path.path.segments;
                    if let Some(last_segment) = segments.last() {
                        last_segment.ident.to_string()
                    } else {
                        "Unknown".to_string()
                    }
                } else {
                    "Unknown".to_string()
                };

                // Generate code to construct the RenderedNode
                let rendered_node_construction_code =
                    parser::generate_rendered_node_code(&type_name, original_method_body);

                let new_body = quote! {
                    {
                        let result = #original_method_body; // Execute original render method

                        // Check if this view is registered for inspection
                        let __view_id = {
                            // Try to get entity_id if available in context
                            // For now, always capture for registered types
                            let __type_name = #type_name;
                            __type_name
                        };

                        // Always capture render tree for inspection
                        let __rendered_node = {
                            #rendered_node_construction_code
                        };
                        gpui_inspector::set_rendered_tree(__rendered_node.clone());
                        gpui_inspector::print_rendered_node_to_console(&__rendered_node);

                        result
                    }
                };
                let new_body_str = new_body.to_string();
                method.block = syn::parse2(new_body).unwrap_or_else(|e| {
                    panic!(
                        "Failed to parse generated code: {}\nGenerated code:\n{}",
                        e, new_body_str
                    );
                });
            }
        }
    }

    quote! {
        #output
    }
    .into()
}

#[proc_macro_attribute]
pub fn inspector_main(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);

    let sig = &input.sig;
    let vis = &input.vis;
    let attrs = &input.attrs;
    let body = &input.block;

    // Transform the body to wrap all Entity::new calls with inspector hooks
    let transformed_body = rewriter::transform_main_body(body);

    quote! {
        #(#attrs)*
        #vis #sig {
            // Enable automatic inspection for all views
            gpui_inspector::hooks::enable_auto_inspection();

            #transformed_body
        }
    }
    .into()
}

/// Auto-inspector macro that applies inspector behavior when auto-inspection is enabled
/// Use this instead of #[inspector] when using #[inspector_main]
#[proc_macro_attribute]
pub fn auto_inspector(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemImpl);

    let trait_path = match &input.trait_ {
        Some((_, path, _)) => path,
        None => return quote! { #input }.into(),
    };

    let is_render = trait_path.segments.iter().any(|s| s.ident == "Render");
    let is_render_once = trait_path.segments.iter().any(|s| s.ident == "RenderOnce");

    if !is_render && !is_render_once {
        return quote! { #input }.into();
    }

    let mut output = input.clone();

    for item in &mut output.items {
        if let syn::ImplItem::Fn(method) = item {
            if method.sig.ident == "render" {
                let original_method_body = &method.block;
                let type_name = if let syn::Type::Path(type_path) = &*output.self_ty {
                    let segments = &type_path.path.segments;
                    if let Some(last_segment) = segments.last() {
                        last_segment.ident.to_string()
                    } else {
                        "Unknown".to_string()
                    }
                } else {
                    "Unknown".to_string()
                };

                let rendered_node_construction_code =
                    parser::generate_rendered_node_code(&type_name, original_method_body);

                let new_body = quote! {
                    {
                        let result = #original_method_body;

                        // Only capture if auto-inspection is enabled and this is the target view
                        if gpui_inspector::hooks::is_auto_inspection_enabled()
                            && gpui_inspector::hooks::should_capture_type(#type_name) {
                            let __rendered_node = {
                                #rendered_node_construction_code
                            };
                            gpui_inspector::set_rendered_tree(__rendered_node.clone());
                            gpui_inspector::print_rendered_node_to_console(&__rendered_node);
                        }

                        result
                    }
                };
                let new_body_str = new_body.to_string();
                method.block = syn::parse2(new_body).unwrap_or_else(|e| {
                    panic!(
                        "Failed to parse generated code: {}\nGenerated code:\n{}",
                        e, new_body_str
                    );
                });
            }
        }
    }

    quote! {
        #output
    }
    .into()
}
