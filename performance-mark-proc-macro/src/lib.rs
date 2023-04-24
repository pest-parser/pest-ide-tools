use proc_macro::TokenStream;
use quote::ToTokens;
use syn::{
    parse,
    parse_quote,
    visit_mut::{visit_stmt_mut, VisitMut},
    Expr, Item, Stmt,
};

#[proc_macro_attribute]
pub fn performance_mark(_: TokenStream, item: TokenStream) -> TokenStream {
    let mut function = match parse::<Item>(item).unwrap() {
        Item::Fn(function) => function,
        _ => panic!("Only functions can be marked with #[performance_mark]"),
    };

    let start_stmt: Stmt = parse_quote!(let start = std::time::Instant::now(););

    function.block.stmts.insert(
        0,
        start_stmt,
    );

    let function_name = function.sig.ident.to_string();
    let end_stmts: Vec<Stmt> = parse_quote! {
        if self.config.enable_performance_logging {
            let end = std::time::Instant::now();
            let duration = end.duration_since(start);
            self.client
                .log_message(
                    tower_lsp::lsp_types::MessageType::INFO,
                    format!("(perf) {} took {:?}", #function_name, duration),
                )
                .await;
        }
    };

    let mut visitor = InsertBeforeReturnVisitor(end_stmts);
    visitor.visit_item_fn_mut(&mut function);

    function.into_token_stream().into()
}

struct InsertBeforeReturnVisitor(Vec<Stmt>);

impl InsertBeforeReturnVisitor {
    fn construct_expr(return_stmt: &Stmt, stmts: &Vec<Stmt>) -> Expr {
        let stmts = VecStmt(stmts);
        Expr::Await(parse_quote! {
            async {
                #stmts
                #return_stmt
            }.await
        })
    }
}

impl VisitMut for InsertBeforeReturnVisitor {
    fn visit_stmt_mut(&mut self, stmt: &mut Stmt) {
        let original_stmt = stmt.clone();

        match stmt {
            Stmt::Expr(Expr::Return(return_expr), _) => {
                return_expr
                    .expr
                    .replace(Box::new(InsertBeforeReturnVisitor::construct_expr(
                        &original_stmt,
                        &self.0,
                    )));
            }
            Stmt::Expr(ref mut return_expr, None) => {
                match return_expr {
                    Expr::ForLoop(_) | Expr::If(_) | Expr::Loop(_) | Expr::While(_) => {
                        return visit_stmt_mut(self, stmt);
                    }
                    _ => {}
                }

                *return_expr = InsertBeforeReturnVisitor::construct_expr(
                    &original_stmt,
                    &self.0,
                );
            }
            _ => {},
        }
    }

    fn visit_expr_closure_mut(&mut self, _: &mut syn::ExprClosure) {
        // NO-OP, do not visit the inside of closures
    }
}

struct VecStmt<'a>(&'a Vec<Stmt>);

impl<'a> ToTokens for VecStmt<'a> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        for stmt in self.0.iter() {
            stmt.to_tokens(tokens);
        }
    }
}
