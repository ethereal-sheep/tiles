use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::token::Brace;
use syn::{braced, Expr, Ident, Pat, Token};

struct UiBlock {
    stmts: Vec<UiStmt>,
}

enum UiStmt {
    Widget {
        expr: Expr,
        children: Option<UiBlock>,
    },
    If {
        cond: Expr,
        body: UiBlock,
    },
    For {
        pat: Pat,
        iter: Expr,
        body: UiBlock,
    },
    Raw {
        ident: Ident,
        body: TokenStream2,
    },
    Let {
        pat: Pat,
        widget_expr: Expr,
        children: Option<UiBlock>,
    },
}

impl Parse for UiBlock {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut stmts = Vec::new();
        while !input.is_empty() {
            stmts.push(input.parse::<UiStmt>()?);
        }
        Ok(UiBlock { stmts })
    }
}

impl Parse for UiStmt {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // @ if ... { ... }
        if input.peek(Token![@]) {
            input.parse::<Token![@]>()?;
            if input.peek(Token![if]) {
                input.parse::<Token![if]>()?;
                let cond = Expr::parse_without_eager_brace(input)?;
                let content;
                braced!(content in input);
                let body: UiBlock = content.parse()?;
                return Ok(UiStmt::If { cond, body });
            } else if input.peek(Token![for]) {
                input.parse::<Token![for]>()?;
                let pat: Pat = Pat::parse_multi(input)?;
                input.parse::<Token![in]>()?;
                let iter = Expr::parse_without_eager_brace(input)?;
                let content;
                braced!(content in input);
                let body: UiBlock = content.parse()?;
                return Ok(UiStmt::For { pat, iter, body });
            } else {
                return Err(input.error("expected `if` or `for` after `@`"));
            }
        }

        // |ident| { ... } raw escape
        if input.peek(Token![|]) {
            let fork = input.fork();
            if fork.parse::<Token![|]>().is_ok() {
                if let Ok(_ident) = fork.parse::<Ident>() {
                    if fork.peek(Token![|]) {
                        input.parse::<Token![|]>()?;
                        let ident: Ident = input.parse()?;
                        input.parse::<Token![|]>()?;
                        let content;
                        braced!(content in input);
                        let body: TokenStream2 = content.parse()?;
                        return Ok(UiStmt::Raw { ident, body });
                    }
                }
            }
        }

        // let binding: let pat = expr { children? };
        if input.peek(Token![let]) {
            input.parse::<Token![let]>()?;
            let pat: Pat = Pat::parse_single(input)?;
            input.parse::<Token![=]>()?;
            let expr: Expr = input.parse()?;
            let children = if input.peek(Brace) {
                let content;
                braced!(content in input);
                Some(content.parse::<UiBlock>()?)
            } else {
                None
            };
            // Consume optional semicolon
            if input.peek(Token![;]) {
                input.parse::<Token![;]>()?;
            }
            return Ok(UiStmt::Let {
                pat,
                widget_expr: expr,
                children,
            });
        }

        // Widget expression, possibly followed by { children }
        let expr = Expr::parse_without_eager_brace(input)?;
        let children = if input.peek(Brace) {
            let content;
            braced!(content in input);
            Some(content.parse::<UiBlock>()?)
        } else {
            None
        };

        // Consume optional semicolon
        if input.peek(Token![;]) {
            input.parse::<Token![;]>()?;
        }

        Ok(UiStmt::Widget { expr, children })
    }
}

fn expand_block(block: &UiBlock) -> TokenStream2 {
    let mut pushes = Vec::new();
    for stmt in &block.stmts {
        pushes.push(expand_stmt(stmt));
    }
    quote! {
        {
            let mut __children: Vec<_> = Vec::new();
            #(#pushes)*
            __children
        }
    }
}

fn expand_stmt(stmt: &UiStmt) -> TokenStream2 {
    match stmt {
        UiStmt::Widget { expr, children } => {
            if let Some(block) = children {
                let children_expr = expand_block(block);
                quote! {
                    __children.push(#expr.children(#children_expr));
                }
            } else {
                quote! {
                    __children.push(#expr);
                }
            }
        }
        UiStmt::If { cond, body } => {
            let stmts: Vec<_> = body.stmts.iter().map(|s| expand_stmt(s)).collect();
            quote! {
                if #cond {
                    #(#stmts)*
                }
            }
        }
        UiStmt::For { pat, iter, body } => {
            let stmts: Vec<_> = body.stmts.iter().map(|s| expand_stmt(s)).collect();
            quote! {
                for #pat in #iter {
                    #(#stmts)*
                }
            }
        }
        UiStmt::Raw { ident, body } => {
            quote! {
                {
                    let #ident = &mut __children;
                    #body
                }
            }
        }
        UiStmt::Let { pat, widget_expr, children } => {
            if let Some(block) = children {
                let children_expr = expand_block(block);
                quote! {
                    let #pat = #widget_expr.children(#children_expr);
                    __children.push(#pat);
                }
            } else {
                quote! {
                    let #pat = #widget_expr;
                    __children.push(#pat);
                }
            }
        }
    }
}

/// Builds a `Vec<Node<A>>` from a UI description block.
///
/// Usage:
/// ```ignore
/// let children = ui! {
///     row().gap(4) {
///         button().size(5, 3).color(RED).on_click(|app: &mut MyApp, state| { ... });
///         col() {
///             Node::new().size(10, 5).color(BLUE);
///         }
///     }
///     @ if show_extra {
///         Node::new().size(3, 3).color(GREEN);
///     }
///     @ for item in items {
///         button().size(5, 3).color(item.color);
///     }
/// };
/// ```
#[proc_macro]
pub fn ui(input: TokenStream) -> TokenStream {
    let block = syn::parse_macro_input!(input as UiBlock);
    let expanded = expand_block(&block);
    TokenStream::from(expanded)
}
