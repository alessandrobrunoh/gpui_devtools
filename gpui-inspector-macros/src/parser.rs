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
                let mut #node_var = #receiver_var; 
            };

            if method_name_str == "child" || method_name_str == "children" {
                for arg in &method_call.args {
                    let (child_code, child_var) = generate_element_node_code(arg, id_counter);
                    generated_code = quote! {
                        #generated_code
                        #child_code
                        #node_var.children.push(#child_var);
                    };
                }
            } else if method_name_str == "id" {
                if let Some(arg) = method_call.args.first() {
                    let arg_str = arg.to_token_stream().to_string().replace("\"", "");
                    generated_code = quote! {
                        #generated_code
                        #node_var.global_element_id = Some(#arg_str.to_string());
                        let __source_location = format!("{}:{}", file!(), line!());
                        #node_var.global_element_path = Some(__source_location.clone());
                        gpui_inspector::hooks::register_element_with_gpui(#current_id, #arg_str.to_string());
                        gpui_inspector::hooks::register_element_path_with_gpui(#current_id, __source_location);
                    };
                }
            } else if method_name_str == "when" || method_name_str == "when_some" {
                // For 'when', the last argument is the closure that modifies the element
                if let Some(last_arg) = method_call.args.last() {
                    if let Expr::Closure(closure) = last_arg {
                        // We can't easily execute the closure at compile time,
                        // but we can try to parse its body if it's a simple expression
                        let _closure_body = &closure.body;
                        // For now, just mark it in properties
                        generated_code = quote! {
                            #generated_code
                            #node_var.properties.insert(stringify!(#method_name).to_string(), "conditional".to_string());
                        };
                    }
                }
            } else {
                let method_name_str = method_name.to_string();
                let args_code: Vec<_> = method_call
                    .args
                    .iter()
                    .map(|arg| {
                        quote! {
                            {
                                use gpui_inspector::tree::Inspectable;
                                (&#arg).inspect()
                            }
                        }
                    })
                    .collect();

                generated_code = quote! {
                    #generated_code
                    let __args = vec![#(#args_code),*];
                    #node_var.properties.insert(#method_name_str.to_string(), __args.join(", "));
                };
            }
            (generated_code, node_var)
        }
        Expr::Call(call) => {
            let name = call.func.to_token_stream().to_string();
            let mut global_id = None;
            
            // Special handling for Button::new("id")
            if name.contains("Button :: new") || name == "Button::new" {
                if let Some(arg) = call.args.first() {
                    global_id = Some(arg.to_token_stream().to_string().replace("\"", ""));
                }
            }

            let global_id_code = if let Some(id) = global_id {
                quote! { Some(#id.to_string()) }
            } else {
                quote! { None }
            };

            generated_code = quote! {
                let mut #node_var = gpui_inspector::tree::ElementNode {
                    id: #current_id,
                    name: #name.to_string(),
                    properties: std::collections::BTreeMap::new(),
                    children: Vec::new(),
                    global_element_id: #global_id_code,
                    global_element_path: None,
                };
            };
            
            if name.contains("Button :: new") || name == "Button::new" {
                if let Some(arg) = call.args.first() {
                    let gid = arg.to_token_stream().to_string().replace("\"", "");
                    generated_code = quote! {
                        #generated_code
                        let __source_location = format!("{}:{}", file!(), line!());
                        #node_var.global_element_path = Some(__source_location.clone());
                        gpui_inspector::hooks::register_element_with_gpui(#current_id, #gid.to_string());
                        gpui_inspector::hooks::register_element_path_with_gpui(#current_id, __source_location);
                    };
                }
            }

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
                    global_element_id: None,
                    global_element_path: None,
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
                version: 0,
                component_name: #component_name_lit.to_string(),
                element_tree: None,
                gpui_element_ids: std::collections::HashMap::new(),
            };
            __rendered_node
        }
    } else {
        quote! {
            #element_tree_setup_code
            let __rendered_node = gpui_inspector::tree::RenderedNode {
                version: 0,
                component_name: #component_name_lit.to_string(),
                element_tree: Some(#element_tree_var),
                gpui_element_ids: std::collections::HashMap::new(),
            };
            __rendered_node
        }
    }
}
