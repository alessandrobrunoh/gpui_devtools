use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote, ToTokens};
use syn::{Block, Expr, Ident, LitStr};

// Helper to convert an argument expression to a runtime-evaluated string
fn arg_to_runtime_string(arg: &Expr) -> TokenStream {
    // String literals - just use the literal
    if let Expr::Lit(syn::ExprLit {
        lit: syn::Lit::Str(lit_str),
        ..
    }) = arg
    {
        return quote! { #lit_str.to_string() };
    }

    // Closures
    if matches!(arg, Expr::Closure(_)) {
        return quote! { "<closure>".to_string() };
    }

    // Check for special patterns that don't implement Display
    let arg_str = arg.to_token_stream().to_string();

    if arg_str.contains(".into_any_element()") {
        return quote! { "<AnyElement>".to_string() };
    }

    if arg_str.contains("listener") {
        return quote! { "<listener>".to_string() };
    }

    // Check for Field access (like self.children)
    if matches!(arg, Expr::Field(_)) {
        let field_str = arg_str.replace(" ", "");
        if field_str.contains("children") {
            return quote! { "<children>".to_string() };
        }
    }

    if arg_str.starts_with("vec") || arg_str.contains(".children") {
        return quote! { "<children>".to_string() };
    }

    // Check for function calls that might return non-Display types
    if let Expr::Call(call) = arg {
        let func_name = call.func.to_token_stream().to_string().replace(" ", "");
        if func_name.contains("rgb") {
            // Return the literal representation of the color call
            return quote! { stringify!(#arg).to_string() };
        }
    }

    // For everything else, try to use Display trait at runtime
    // This will work for String, format!(), numbers, bools, etc.
    quote! { format!("{}", #arg) }
}

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
                    let args_code: Vec<_> =
                        method_call.args.iter().map(arg_to_runtime_string).collect();
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
                let args_code: Vec<_> = call.args.iter().map(arg_to_runtime_string).collect();
                generated_code = quote! {
                    #generated_code
                    #node_var.properties.insert("args".to_string(), vec![#(#args_code),*].join(", "));
                };
            }
            (generated_code, node_var)
        }
        Expr::Match(match_expr) => {
            // For match expressions, we need to generate runtime code that executes the match
            // and captures the result as an ElementNode
            let scrutinee = &match_expr.expr;

            // Generate code for each arm that will build the ElementNode when that arm is executed
            let arms: Vec<_> = match_expr
                .arms
                .iter()
                .map(|arm| {
                    let pat = &arm.pat;
                    let guard = &arm.guard;
                    let body = &arm.body;

                    // Generate element node code for this arm's body
                    let (arm_node_code, arm_node_var) =
                        generate_element_node_code(body, id_counter);

                    // Build the match arm that returns the ElementNode
                    if let Some((if_token, guard_expr)) = guard {
                        quote! {
                            #pat #if_token #guard_expr => {
                                #arm_node_code
                                #arm_node_var
                            }
                        }
                    } else {
                        quote! {
                            #pat => {
                                #arm_node_code
                                #arm_node_var
                            }
                        }
                    }
                })
                .collect();

            // Generate the match expression that returns the selected ElementNode
            generated_code = quote! {
                let #node_var = match #scrutinee {
                    #(#arms),*
                };
            };

            (generated_code, node_var)
        }
        _ => {
            let name = expr.to_token_stream().to_string();

            // Check if this looks like an Entity clone (e.g., self.tab1_view.clone())
            if name.contains(".clone()") && (name.contains("_view") || name.contains("Entity")) {
                let entity_name = name.replace(".clone()", "").replace("self.", "");
                generated_code = quote! {
                    let mut #node_var = gpui_inspector::tree::ElementNode {
                        id: #current_id,
                        name: format!("<Entity: {}>", #entity_name),
                        properties: std::collections::BTreeMap::new(),
                        children: Vec::new(),
                    };
                };
            } else {
                generated_code = quote! {
                    let mut #node_var = gpui_inspector::tree::ElementNode {
                        id: #current_id,
                        name: #name.to_string(),
                        properties: std::collections::BTreeMap::new(),
                        children: Vec::new(),
                    };
                };
            }
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
