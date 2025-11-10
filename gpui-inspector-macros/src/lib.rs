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

                let rendered_node_construction_code =
                    parser::generate_rendered_node_code(&type_name, original_method_body);

                // Transform the body to add inspection code before the return
                let mut new_stmts = original_method_body.stmts.clone();

                // Insert inspection code before the last statement (which is the return expression)
                if !new_stmts.is_empty() {
                    let inspection_code = quote! {
                        // Always capture render tree for inspection
                        let __rendered_node = {
                            #rendered_node_construction_code
                        };
                        gpui_inspector::set_rendered_tree(__rendered_node.clone());
                        gpui_inspector::print_rendered_node_to_console(&__rendered_node);
                    };

                    // Parse the inspection code as a statement
                    let inspection_stmt: syn::Stmt = syn::parse2(inspection_code).unwrap();

                    // Insert before the last statement (the return expression)
                    new_stmts.insert(new_stmts.len() - 1, inspection_stmt);
                }

                let new_body = syn::Block {
                    brace_token: original_method_body.brace_token,
                    stmts: new_stmts,
                };
                method.block = new_body;
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

                // Transform the body to add inspection code before the return
                let mut new_stmts = original_method_body.stmts.clone();

                // Insert inspection code before the last statement (which is the return expression)
                if !new_stmts.is_empty() {
                    let inspection_code = quote! {
                        // Only capture if auto-inspection is enabled
                        if gpui_inspector::hooks::is_auto_inspection_enabled() {
                            let __rendered_node = {
                                #rendered_node_construction_code
                            };

                            // Always save to ALL_TREES so entity expansion can work
                            gpui_inspector::set_rendered_tree(__rendered_node.clone());

                            // But always print to console for debugging
                            gpui_inspector::print_rendered_node_to_console(&__rendered_node);
                        }
                    };

                    // Parse the inspection code as a statement
                    let inspection_stmt: syn::Stmt = syn::parse2(inspection_code).unwrap();

                    // Insert before the last statement (the return expression)
                    new_stmts.insert(new_stmts.len() - 1, inspection_stmt);
                }

                let new_body = syn::Block {
                    brace_token: original_method_body.brace_token,
                    stmts: new_stmts,
                };
                method.block = new_body;
            }
        }
    }

    quote! {
        #output
    }
    .into()
}
