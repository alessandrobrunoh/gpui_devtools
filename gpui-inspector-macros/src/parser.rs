use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote, ToTokens};
use syn::{Block, Expr, Ident, LitStr};

// This function will generate the code to construct an ElementNode tree.
// It returns a TokenStream representing the code, and the Ident of the variable holding the root ElementNode.
pub fn generate_element_node_code(expr: &Expr, id_counter: &mut u64) -> (TokenStream, Ident) {
    let current_id = *id_counter;
    *id_counter += 1;
    let node_var = format_ident!("__inspector_node_{}", current_id);

    let mut generated_code;

    match expr {
        Expr::MethodCall(method_call) => {
            let (receiver_code, receiver_var) =
                generate_element_node_code(&method_call.receiver, id_counter);
            let method_name = &method_call.method;
            let method_name_str = method_name.to_string();

            generated_code = quote! {
                #receiver_code
                let mut #node_var = #receiver_var; // Start with the receiver node
            };

            if method_name_str == "child" {
                if let Some(arg) = method_call.args.first() {
                    let (child_code, child_var) = generate_element_node_code(arg, id_counter);
                    generated_code = quote! {
                        #generated_code
                        #child_code
                        #node_var.children.push(#child_var);
                    };
                }
            } else {
                let args_str = if method_call.args.is_empty() {
                    quote! { "".to_string() }
                } else {
                    let args_code: Vec<_> = method_call
                        .args
                        .iter()
                        .map(|arg| {
                            if let Expr::Lit(syn::ExprLit {
                                lit: syn::Lit::Str(lit_str),
                                ..
                            }) = arg
                            {
                                quote! { #lit_str.to_string() }
                            } else if matches!(arg, Expr::Closure(_)) {
                                quote! { "<closure>".to_string() }
                            } else {
                                let arg_str = arg.to_token_stream().to_string();
                                quote! { #arg_str.to_string() }
                            }
                        })
                        .collect();
                    quote! { vec![#(#args_code),*].join(", ") }
                };
                generated_code = quote! {
                    #generated_code
                    #node_var.properties.insert(stringify!(#method_name).to_string(), #args_str);
                };
            }
            (generated_code, node_var)
        }
        Expr::Call(call) => {
            let name = call.func.to_token_stream().to_string();
            generated_code = quote! {
                let mut #node_var = gpui_inspector::tree::ElementNode {
                    id: #current_id,
                    name: #name.to_string(),
                    properties: std::collections::BTreeMap::new(),
                    children: Vec::new(),
                };
            };
            if !call.args.is_empty() {
                let args_code: Vec<_> = call
                    .args
                    .iter()
                    .map(|arg| {
                        if let Expr::Lit(syn::ExprLit {
                            lit: syn::Lit::Str(lit_str),
                            ..
                        }) = arg
                        {
                            quote! { #lit_str.to_string() }
                        } else if matches!(arg, Expr::Closure(_)) {
                            quote! { "<closure>".to_string() }
                        } else {
                            let arg_str = arg.to_token_stream().to_string();
                            quote! { #arg_str.to_string() }
                        }
                    })
                    .collect();
                generated_code = quote! {
                    #generated_code
                    #node_var.properties.insert("args".to_string(), vec![#(#args_code),*].join(", "));
                };
            }
            (generated_code, node_var)
        }
        _ => {
            let name = expr.to_token_stream().to_string();
            generated_code = quote! {
                let mut #node_var = gpui_inspector::tree::ElementNode {
                    id: #current_id,
                    name: #name.to_string(),
                    properties: std::collections::BTreeMap::new(),
                    children: Vec::new(),
                };
            };
            (generated_code, node_var)
        }
    }
}

// Main entry point for the parser
pub fn generate_rendered_node_code(component_name: &str, body: &Block) -> TokenStream {
    let mut id_counter = 0;
    let (element_tree_setup_code, element_tree_var) =
        if let Some(syn::Stmt::Expr(expr, _)) = body.stmts.last() {
            generate_element_node_code(expr, &mut id_counter)
        } else {
            (quote! {}, format_ident!("__empty_node"))
        };

    let component_name_lit = LitStr::new(component_name, Span::call_site());

    let element_tree_var_name = element_tree_var.to_string();
    let is_empty = element_tree_var_name == "__empty_node";

    if is_empty {
        quote! {
            let __rendered_node = gpui_inspector::tree::RenderedNode {
                component_name: #component_name_lit.to_string(),
                element_tree: None,
            };
            __rendered_node
        }
    } else {
        quote! {
            #element_tree_setup_code
            let __rendered_node = gpui_inspector::tree::RenderedNode {
                component_name: #component_name_lit.to_string(),
                element_tree: Some(#element_tree_var),
            };
            __rendered_node
        }
    }
}
