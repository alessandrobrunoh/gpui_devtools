use quote::quote;
use syn::{Expr, Stmt, Block, parse_quote, punctuated::Punctuated, token::Comma, ExprMethodCall, ExprCall};

pub fn instrument_block(block: &Block) -> Block {
    let mut new_stmts = Vec::new();
    for stmt in &block.stmts {
        new_stmts.push(instrument_stmt(stmt));
    }
    Block {
        brace_token: block.brace_token,
        stmts: new_stmts,
    }
}

fn instrument_stmt(stmt: &Stmt) -> Stmt {
    match stmt {
        Stmt::Expr(expr, semi) => Stmt::Expr(instrument_expr(expr), *semi),
        Stmt::Local(local) => {
            let mut new_local = local.clone();
            if let Some(init) = new_local.init.as_mut() {
                init.expr = Box::new(instrument_expr(&init.expr));
            }
            Stmt::Local(new_local)
        }
        _ => stmt.clone(),
    }
}

fn analyze_chain(expr: &Expr) -> Option<(ExprCall, Vec<ExprMethodCall>)> {
    let mut current = expr;
    let mut methods = Vec::new();

    loop {
        match current {
            Expr::MethodCall(m) => {
                methods.push(m.clone());
                current = &m.receiver;
            }
            Expr::Call(c) => {
                methods.reverse();
                return Some((c.clone(), methods));
            }
            _ => return None,
        }
    }
}

fn get_root_name(call: &ExprCall) -> String {
    quote!(#call.func).to_string().replace(" ", "")
}

fn is_control_flow(expr: &Expr) -> bool {
    matches!(expr, Expr::If(_) | Expr::Match(_) | Expr::Block(_) | Expr::Loop(_) | Expr::ForLoop(_))
}

fn instrument_chain(root: ExprCall, methods: Vec<ExprMethodCall>) -> Expr {
    let root_name = get_root_name(&root);
    
    let mut new_root = root.clone();
    new_root.args = root.args.iter().map(|arg| instrument_expr(arg)).collect();
    
    let mut method_stmts: Vec<Stmt> = Vec::new();
    
    for m in methods {
        let method_name = m.method.to_string();
        let pass_through_methods = [
            "iter", "iter_mut", "into_iter", "filter", "map", "flat_map", 
            "collect", "unwrap", "expect", "as_ref", "as_mut", "clone", 
            "len", "is_empty", "push", "pop", "insert", "remove", "get", "find",
            "contains", "to_string", "to_lowercase", "to_uppercase", "trim",
            "then", "then_some", "ok_or", "ok_or_else", "enumerate",
            "on_click", "on_mouse_down", "on_mouse_up", "on_mouse_move", "on_scroll_wheel",
            "overflow_y_scroll", "overflow_x_scroll"
        ];
        
        if pass_through_methods.contains(&method_name.as_str()) {
             let mut new_m = m.clone();
             new_m.receiver = parse_quote!(__base);
             new_m.args = m.args.iter().map(|arg| instrument_expr(arg)).collect();
             method_stmts.push(parse_quote! {
                 let __base = #new_m;
             });
             continue;
        }

        if method_name == "child" || method_name == "children" {
            let args: Vec<Expr> = m.args.iter().map(|arg| {
                let instrumented_arg = instrument_expr(arg);
                
                let (should_wrap, wrap_name) = if let Some((c, _)) = analyze_chain(arg) {
                    (false, get_root_name(&c))
                } else if let Expr::Call(c) = arg {
                    (false, get_root_name(c))
                } else if is_control_flow(arg) {
                    (false, "control_flow".to_string())
                } else {
                    (true, "child".to_string())
                };

                if should_wrap {
                    parse_quote! {
                        {
                            gpui_inspector::builder_enter(#wrap_name);
                            let _guard = gpui_inspector::tree::NodeGuard;
                            #instrumented_arg
                        }
                    }
                } else {
                    instrumented_arg
                }
            }).collect();
            
            let mut new_m = m.clone();
            new_m.receiver = parse_quote!(__base);
            new_m.args = args.into_iter().collect();
             method_stmts.push(parse_quote! {
                 let __base = #new_m;
             });

        } else if method_name == "id" {
             if let Some(arg) = m.args.first() {
                 method_stmts.push(parse_quote! {
                    {
                        let __id = #arg;
                        let __id_str = __id.to_string();
                        gpui_inspector::builder_id(__id_str.clone());
                        gpui_inspector::builder_path(format!("{}:{}", file!(), line!()));
                    }
                 });
                 method_stmts.push(parse_quote! {
                    let __base = __base.when(gpui_inspector::hooks::is_selected(&#arg.to_string()), |e| {
                         e.border_2().border_color(gpui::rgb(0xffa500))
                    });
                 });
             }
             let mut new_m = m.clone();
             new_m.receiver = parse_quote!(__base);
             method_stmts.push(parse_quote! {
                 let __base = #new_m;
             });

        } else if method_name == "when" || method_name == "when_some" {
            let mut new_m = m.clone();
             new_m.receiver = parse_quote!(__base);
             new_m.args = m.args.iter().map(|arg| instrument_expr(arg)).collect();
             method_stmts.push(parse_quote! {
                 let __base = #new_m;
             });
        } else {
             for arg in &m.args {
                 if let Expr::Closure(_) = arg { continue; }
                 if let Expr::Path(p) = arg {
                        if p.path.is_ident("cx") || p.path.is_ident("window") || p.path.is_ident("_cx") || p.path.is_ident("_window") {
                            continue;
                        }
                 }
                 
                 let name = method_name.clone();
                 method_stmts.push(parse_quote! {
                    {
                        let __v = #arg;
                        #[allow(unused_imports)]
                        use gpui_inspector::tree::InspectorDebugFallback;
                        gpui_inspector::builder_prop(#name, gpui_inspector::tree::InspectorDebugValue(&__v).inspect());
                    }
                 });
             }
             
             let mut new_m = m.clone();
             new_m.receiver = parse_quote!(__base);
             new_m.args = m.args.iter().map(|arg| instrument_expr(arg)).collect();
             method_stmts.push(parse_quote! {
                 let __base = #new_m;
             });
        }
    }

    parse_quote! {
        {
            gpui_inspector::builder_enter(#root_name);
            let _guard = gpui_inspector::tree::NodeGuard;
            let __base = #new_root;
            #(#method_stmts)*
            __base
        }
    }
}

fn instrument_expr(expr: &Expr) -> Expr {
    if let Some((root, methods)) = analyze_chain(expr) {
        return instrument_chain(root, methods);
    }

    match expr {
        Expr::MethodCall(m) => {
             let method_name = m.method.to_string();
             let mut new_m = m.clone();
             new_m.receiver = Box::new(instrument_expr(&m.receiver));
             new_m.args = m.args.iter().map(|arg| instrument_expr(arg)).collect();
             
             parse_quote! {
                 {
                     gpui_inspector::builder_enter(#method_name);
                     let _guard = gpui_inspector::tree::NodeGuard;
                     #new_m
                 }
             }
        }
        Expr::Call(c) => {
             let func_name = get_root_name(c);
             let mut new_c = c.clone();
             new_c.args = c.args.iter().map(|arg| instrument_expr(arg)).collect();
             
             parse_quote! {
                 {
                     gpui_inspector::builder_enter(#func_name);
                     let _guard = gpui_inspector::tree::NodeGuard;
                     #new_c
                 }
             }
        }
        Expr::Closure(c) => {
            let mut new_c = c.clone();
            new_c.body = Box::new(instrument_expr(&c.body));
            Expr::Closure(new_c)
        }
        Expr::Block(b) => {
            let mut new_b = b.clone();
            new_b.block = instrument_block(&b.block);
            Expr::Block(new_b)
        }
        Expr::If(i) => {
            let mut new_i = i.clone();
            new_i.cond = Box::new(instrument_expr(&i.cond));
            new_i.then_branch = instrument_block(&i.then_branch);
            if let Some((else_token, else_branch)) = &i.else_branch {
                new_i.else_branch = Some((*else_token, Box::new(instrument_expr(else_branch))));
            }
            Expr::If(new_i)
        }
        Expr::Match(m) => {
            let mut new_m = m.clone();
            new_m.expr = Box::new(instrument_expr(&m.expr));
            new_m.arms = m.arms.iter().map(|arm| {
                let mut new_arm = arm.clone();
                new_arm.body = Box::new(instrument_expr(&arm.body));
                new_arm
            }).collect();
            Expr::Match(new_m)
        }
        _ => expr.clone(),
    }
}
