use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse_quote, visit_mut::VisitMut, Block, Expr, ExprMethodCall, Stmt};

pub fn transform_main_body(body: &Block) -> TokenStream {
    let mut visitor = MainBodyVisitor::new();
    let mut transformed_body = body.clone();
    visitor.visit_block_mut(&mut transformed_body);

    quote! {
        #transformed_body
    }
}

struct MainBodyVisitor;

impl MainBodyVisitor {
    fn new() -> Self {
        Self
    }
}

impl VisitMut for MainBodyVisitor {
    fn visit_expr_mut(&mut self, expr: &mut Expr) {
        // Visit children first
        syn::visit_mut::visit_expr_mut(self, expr);

        match expr {
            Expr::MethodCall(method_call) => {
                // Check if this is cx.open_window()
                if method_call.method == "open_window" {
                    if let Expr::Path(ref path) = *method_call.receiver {
                        if let Some(ident) = path.path.get_ident() {
                            if ident == "cx" {
                                // Found cx.open_window(), need to transform its closure
                                if method_call.args.len() >= 2 {
                                    if let Some(closure_arg) = method_call.args.iter_mut().nth(1) {
                                        self.transform_window_closure(closure_arg);
                                    }
                                }
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }
}

impl MainBodyVisitor {
    fn transform_window_closure(&mut self, closure_expr: &mut Expr) {
        if let Expr::Closure(closure) = closure_expr {
            // Visit the closure body and wrap cx.new() calls
            let mut body_visitor = ClosureBodyVisitor::new();
            body_visitor.visit_expr_mut(&mut closure.body);
        }
    }
}

struct ClosureBodyVisitor;

impl ClosureBodyVisitor {
    fn new() -> Self {
        Self
    }

    fn should_wrap_new_call(&self, method_call: &ExprMethodCall) -> bool {
        // Check if this is cx.new() that creates a view
        if method_call.method != "new" {
            return false;
        }

        if let Expr::Path(ref path) = *method_call.receiver {
            if let Some(ident) = path.path.get_ident() {
                return ident == "cx";
            }
        }

        false
    }

    fn wrap_view_creation(&self, original_expr: Expr) -> Expr {
        // Create wrapper that calls inspector hook
        parse_quote! {
            {
                let __inspector_view = #original_expr;

                // Hook into the view's render method
                gpui_inspector::hooks::install_render_hook(&__inspector_view);

                __inspector_view
            }
        }
    }
}

impl VisitMut for ClosureBodyVisitor {
    fn visit_expr_mut(&mut self, expr: &mut Expr) {
        // First visit children
        syn::visit_mut::visit_expr_mut(self, expr);

        match expr {
            Expr::MethodCall(method_call) => {
                if self.should_wrap_new_call(method_call) {
                    // Check if the closure creates a type that might implement Render
                    // We wrap all cx.new() calls to be safe
                    let original = expr.clone();
                    *expr = self.wrap_view_creation(original);
                }
            }
            Expr::Block(block) => {
                // Handle blocks that return views
                if let Some(last_stmt) = block.block.stmts.last_mut() {
                    if let Stmt::Expr(expr, None) = last_stmt {
                        if let Expr::MethodCall(method_call) = expr {
                            if self.should_wrap_new_call(method_call) {
                                let original = expr.clone();
                                *expr = self.wrap_view_creation(original);
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }

    fn visit_stmt_mut(&mut self, stmt: &mut Stmt) {
        // Visit nested statements
        syn::visit_mut::visit_stmt_mut(self, stmt);

        // Handle let bindings that create views
        if let Stmt::Local(local) = stmt {
            if let Some(init) = &mut local.init {
                if let Expr::MethodCall(method_call) = &mut *init.expr {
                    if self.should_wrap_new_call(method_call) {
                        let original = init.expr.clone();
                        *init.expr = self.wrap_view_creation(*original);
                    }
                }
            }
        }
    }
}
