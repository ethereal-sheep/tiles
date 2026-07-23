use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::spanned::Spanned;
use syn::token::Brace;
use syn::{
    Expr, FnArg, Ident, ImplItem, ItemImpl, LitStr, Pat, Path, Token, Type, braced,
    parse_macro_input,
};

// =============================================================================
// widget! macro
// =============================================================================

struct WidgetInput {
    block: UiBlock,
}

impl Parse for WidgetInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let block: UiBlock = input.parse()?;
        Ok(WidgetInput { block })
    }
}

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
    Splice {
        expr: Expr,
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
                let expr: Expr = input.parse()?;
                if input.peek(Token![;]) {
                    input.parse::<Token![;]>()?;
                }
                return Ok(UiStmt::Splice { expr });
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

fn has_explicit_id(expr: &Expr) -> bool {
    match expr {
        Expr::MethodCall(mc) => {
            if mc.method == "id" {
                return true;
            }
            has_explicit_id(&mc.receiver)
        }
        _ => false,
    }
}

fn should_inject_id(expr: &Expr) -> bool {
    if has_explicit_id(expr) {
        return false;
    }
    // Only inject on method chains (e.g. pane().size(10, 10)).
    // For bare function calls like my_widget(), the root is a Call with
    // no method chain — these may set their own ID internally, and
    // resolve_id provides a fallback if they don't.
    match expr {
        Expr::MethodCall(_) => true,
        Expr::Call(_) => false,
        _ => false,
    }
}

fn auto_id_for_expr(expr: &Expr, in_loop: bool) -> TokenStream2 {
    if !should_inject_id(expr) {
        return quote! {};
    }
    let span = match expr {
        Expr::MethodCall(mc) => mc.receiver.span(),
        Expr::Call(call) => call.func.span(),
        _ => expr.span(),
    };
    let line = span.start().line;
    let col = span.start().column;
    if in_loop {
        let base = format!("{}:{}/", line, col);
        quote! { .id(&format!(concat!(#base, "{}"), __widget_idx)) }
    } else {
        let id_str = format!("{}:{}", line, col);
        quote! { .id(#id_str) }
    }
}

/// Built-in constructors (`pane`, `row`, `col`, ...) — never call `signal()`
/// internally, so never need widget-id scoping.
const BUILTIN_NODE_CONSTRUCTORS: &[&str] = &["pane", "row", "col", "text", "img", "paint"];

fn is_builtin_constructor_call(func: &Expr) -> bool {
    match func {
        Expr::Path(path) => {
            path.path.segments.last().is_some_and(|seg| {
                BUILTIN_NODE_CONSTRUCTORS.contains(&seg.ident.to_string().as_str())
            })
        }
        _ => false,
    }
}

fn expand_widget_block(block: &UiBlock, in_loop: bool) -> TokenStream2 {
    let mut pushes = Vec::new();
    for stmt in &block.stmts {
        pushes.push(expand_widget_stmt(stmt, in_loop));
    }
    quote! {
        {
            let mut __children = Vec::new();
            #(#pushes)*
            __children
        }
    }
}

/// Walks down through `.method()` chains to find the root call — the
/// expression a chain like `widget_fn(...).fill_w().on_press(f)` bottoms
/// out on. Returns `None` if the chain doesn't bottom out on a bare call
/// (e.g. it's a plain identifier or literal).
fn root_call_of(expr: &Expr) -> Option<&syn::ExprCall> {
    match expr {
        Expr::Call(call) => Some(call),
        Expr::MethodCall(mc) => root_call_of(&mc.receiver),
        _ => None,
    }
}

/// True if `expr` is a call to a user-defined widget function (bare, or
/// chained with builder calls like `.fill_w()`/`.on_press(...)`) — i.e. not
/// one of the built-in constructors, which never call `signal()` internally
/// and so never need widget-id scoping.
fn is_widget_fn_call(expr: &Expr) -> bool {
    root_call_of(expr).is_some_and(|call| !is_builtin_constructor_call(&call.func))
}

fn signal_context_for_expr(expr: &Expr, in_loop: bool) -> (TokenStream2, TokenStream2) {
    if !is_widget_fn_call(expr) {
        return (quote! {}, quote! {});
    }
    let root_call = root_call_of(expr).expect("is_widget_fn_call implies root_call_of is Some");
    let span = root_call.func.span();
    let line = span.start().line as u64;
    let col = span.start().column as u64;
    let widget_id = if in_loop {
        quote! {
            {
                let __base = ::tiles::__private::__widget_id("", #line as u32, #col as u32);
                __base.wrapping_mul(2654435761).wrapping_add(__widget_idx as u64)
            }
        }
    } else {
        quote! { ::tiles::__private::__widget_id("", #line as u32, #col as u32) }
    };
    let before = quote! { ::tiles::__private::__push_widget(#widget_id); };
    let after = quote! { ::tiles::__private::__pop_widget(); };
    (before, after)
}

fn expand_widget_stmt(stmt: &UiStmt, in_loop: bool) -> TokenStream2 {
    match stmt {
        UiStmt::Widget { expr, children } => {
            let auto_id = auto_id_for_expr(expr, in_loop);
            let children_expr = if let Some(block) = children {
                expand_widget_block(block, in_loop)
            } else {
                quote! { vec![] }
            };
            let (sig_before, sig_after) = signal_context_for_expr(expr, in_loop);
            quote! {
                #sig_before
                __children.push(
                    ::tiles::__private::Widget::render(#expr #auto_id, #children_expr)
                );
                #sig_after
            }
        }
        UiStmt::If { cond, body } => {
            let stmts: Vec<_> = body
                .stmts
                .iter()
                .map(|s| expand_widget_stmt(s, in_loop))
                .collect();
            quote! {
                if #cond {
                    #(#stmts)*
                }
            }
        }
        UiStmt::For { pat, iter, body } => {
            let stmts: Vec<_> = body
                .stmts
                .iter()
                .map(|s| expand_widget_stmt(s, true))
                .collect();
            quote! {
                for (__widget_idx, #pat) in (#iter).into_iter().enumerate() {
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
        UiStmt::Let {
            pat,
            widget_expr,
            children,
        } => {
            let auto_id = auto_id_for_expr(widget_expr, in_loop);
            let children_expr = if let Some(block) = children {
                expand_widget_block(block, in_loop)
            } else {
                quote! { vec![] }
            };
            quote! {
                let #pat = ::tiles::__private::Widget::render(#widget_expr #auto_id, #children_expr);
                __children.push(#pat);
            }
        }
        UiStmt::Splice { expr } => {
            quote! {
                __children.extend(#expr);
            }
        }
    }
}

/// Builds a UI widget tree using the Widget trait.
///
/// Supports `@children` to splice a `Vec<Node>` variable into the children list.
#[proc_macro]
pub fn widget(input: TokenStream) -> TokenStream {
    let WidgetInput { block } = syn::parse_macro_input!(input as WidgetInput);

    // Single top-level widget → return Node directly
    if block.stmts.len() == 1 {
        if let UiStmt::Widget { expr, children } = &block.stmts[0] {
            let children_expr = if let Some(child_block) = children {
                expand_widget_block(child_block, false)
            } else {
                quote! { vec![] }
            };
            let expanded = quote! {
                ::tiles::__private::Widget::render(#expr, #children_expr)
            };
            return TokenStream::from(expanded);
        }
    }

    let expanded = expand_widget_block(&block, false);
    TokenStream::from(expanded)
}

/// Computes the outer (caller-facing) parameter list and generics for a
/// widget-fn-style function: `impl Trait` params get `+ 'static`, bare `&T`
/// params get an `'a` lifetime (and the generics gain `'a` accordingly,
/// since the params are captured into a closure). Also returns whether any
/// borrow was found, so callers can decide how to spell their return type's
/// lifetime.
fn compute_outer_signature(
    generics: &syn::Generics,
    params: &[&syn::FnArg],
) -> (Vec<TokenStream2>, TokenStream2, bool) {
    let has_borrows = params.iter().any(|p| match p {
        syn::FnArg::Typed(pat_type) => matches!(*pat_type.ty, syn::Type::Reference(_)),
        _ => false,
    });

    let outer_params: Vec<TokenStream2> = params
        .iter()
        .map(|p| match p {
            syn::FnArg::Typed(pat_type) => {
                let pat = &pat_type.pat;
                let ty = &pat_type.ty;
                match &**ty {
                    syn::Type::Reference(r) if has_borrows => {
                        if r.lifetime.is_none() {
                            let mutability = &r.mutability;
                            let elem = &r.elem;
                            quote! { #pat: &'a #mutability #elem }
                        } else {
                            quote! { #p }
                        }
                    }
                    syn::Type::ImplTrait(impl_trait) => {
                        let has_lifetime_bound = impl_trait
                            .bounds
                            .iter()
                            .any(|b| matches!(b, syn::TypeParamBound::Lifetime(_)));
                        if has_lifetime_bound {
                            quote! { #p }
                        } else {
                            let bounds = &impl_trait.bounds;
                            quote! { #pat: impl #bounds + 'static }
                        }
                    }
                    _ => quote! { #p },
                }
            }
            _ => quote! { #p },
        })
        .collect();

    let (impl_generics, _ty_generics, _where_clause) = generics.split_for_impl();
    let combined_generics = if has_borrows {
        let user_params = &generics.params;
        if user_params.is_empty() {
            quote! { <'a> }
        } else {
            quote! { <'a, #user_params> }
        }
    } else {
        quote! { #impl_generics }
    };

    (outer_params, combined_generics, has_borrows)
}

// =============================================================================
// #[widget_fn] attribute macro
// =============================================================================

/// Attribute macro for declaring widget functions that can receive the
/// caller's style/handlers/children via a trailing `Props` parameter.
///
/// If the last parameter's type is `Props`, the function is wrapped in
/// `WidgetFn` so the caller's `.style(...)`-affecting builder calls and
/// `.on_*` handlers are captured and handed to the function as `Props`
/// (any pattern is allowed — full destructure, partial, or a plain binding).
/// Fields the function doesn't bind are simply unavailable to it and never
/// applied to the returned node (`id` is the only exception — it's always
/// stamped onto the returned node automatically if the caller set one).
///
/// If there's no trailing `Props` parameter, the function is wrapped in
/// a `BlankWidgetFn` which does not allow styling or handling at the caller,
/// and silently ignores children.
///
/// ```ignore
/// #[widget_fn]
/// fn button(word: &str, Props { style, handlers, children }: Props) -> Node {
///     widget! {
///         col().style(style).handlers(handlers) {
///             text(word).padding(1)
///             @children
///         }
///     }
/// }
/// ```
#[proc_macro_attribute]
pub fn widget_fn(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let func: syn::ItemFn = syn::parse_macro_input!(item as syn::ItemFn);

    match impl_widget_fn(&func) {
        Ok(ts) => ts.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

/// Returns true if `ty`'s final path segment is `Props`.
fn is_props_type(ty: &Type) -> bool {
    match ty {
        Type::Path(type_path) => type_path
            .path
            .segments
            .last()
            .is_some_and(|seg| seg.ident == "Props"),
        _ => false,
    }
}

fn impl_widget_fn(func: &syn::ItemFn) -> syn::Result<TokenStream2> {
    let params: Vec<&syn::FnArg> = func.sig.inputs.iter().collect();

    let last_is_props = matches!(
        params.last(),
        Some(syn::FnArg::Typed(pat_type)) if is_props_type(&pat_type.ty)
    );

    let mut has_props = last_is_props;
    if !last_is_props {
        // Error if a Props param appears anywhere but last.
        for p in params.iter().rev().skip(1) {
            if let syn::FnArg::Typed(pat_type) = p {
                if is_props_type(&pat_type.ty) {
                    return Err(syn::Error::new_spanned(
                        pat_type,
                        "#[new_widget_fn] a `Props` parameter must be the last parameter",
                    ));
                }
            }
        }
        has_props = false;
    }

    let vis = &func.vis;
    let name = &func.sig.ident;
    let generics = &func.sig.generics;
    let (_impl_generics, _ty_generics, where_clause) = generics.split_for_impl();
    let body = &func.block;

    if !has_props {
        let (outer_params, combined_generics, has_borrows) =
            compute_outer_signature(generics, &params);

        // `BlankWidgetFn` is returned concretely (not as `impl Widget`) so callers
        // can chain its inherent builder methods (`.fill_w()`, `.on_press()`, `.id()`).
        let closure_bound = if has_borrows {
            quote! { impl FnOnce() -> ::tiles::Node + 'a }
        } else {
            quote! { impl FnOnce() -> ::tiles::Node }
        };
        let return_ty = quote! { ::tiles::__private::BlankWidgetFn<#closure_bound> };
        return Ok(quote! {
            #vis fn #name #combined_generics(#(#outer_params),*) -> #return_ty #where_clause {
                ::tiles::__private::BlankWidgetFn::new(move || #body)
            }
        });
    }

    let mut params = params;
    let props_param = params.pop().unwrap();

    // Error if another Props param appears earlier in the list.
    for p in &params {
        if let syn::FnArg::Typed(pat_type) = p {
            if is_props_type(&pat_type.ty) {
                return Err(syn::Error::new_spanned(
                    pat_type,
                    "#[new_widget_fn] a `Props` parameter must be the last parameter",
                ));
            }
        }
    }

    let (props_pat, props_ty) = match props_param {
        syn::FnArg::Typed(pat_type) => (&pat_type.pat, &pat_type.ty),
        syn::FnArg::Receiver(_) => {
            return Err(syn::Error::new_spanned(
                props_param,
                "#[new_widget_fn] cannot be used on methods with self",
            ));
        }
    };

    let (outer_params, combined_generics, has_borrows) = compute_outer_signature(generics, &params);

    // `NewWidgetFn` is returned concretely (not as `impl Widget`) so callers
    // can chain its inherent builder methods (`.fill_w()`, `.on_press()`, `.id()`).
    let closure_bound = if has_borrows {
        quote! { impl FnOnce(::tiles::Props) -> ::tiles::Node + 'a }
    } else {
        quote! { impl FnOnce(::tiles::Props) -> ::tiles::Node }
    };
    let return_ty = quote! { ::tiles::__private::WidgetFn<#closure_bound> };

    Ok(quote! {
        #vis fn #name #combined_generics(#(#outer_params),*) -> #return_ty #where_clause {
            ::tiles::__private::WidgetFn::new(move |#props_pat: #props_ty| #body)
        }
    })
}

// =============================================================================
// #[derive(Builders)] proc macro
// =============================================================================
//
// Generates builder methods from struct fields based on attributes:
//
//   #[builder]              — Option<T> field → fn field(mut self, v: T) -> Self
//   #[builder(combo(name = "size", fields = "w, h"))]
//                           — generates fn size(mut self, w: T, h: T) -> Self
//   #[builder(variant(name = "relative", variant = "Relative", args = "x: i32, y: i32"))]
//                           — generates fn relative(mut self, x: i32, y: i32) -> Self
//                             setting field = EnumType::Relative(x, y)
//
// The generated impl targets a different type (the "owner") specified via
// a struct-level attribute:
//   #[builders(owner = "PaneNode", access = "self.style")]

use syn::{Data, DeriveInput, Fields};

#[proc_macro_derive(Builders, attributes(builder, builders))]
pub fn derive_builders(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as DeriveInput);
    match impl_builders(&input) {
        Ok(ts) => ts.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

fn gen_default_method_for_field(
    field_name: &syn::Ident,
    field_ty: &Type,
    prefix: &TokenStream2,
) -> Vec<TokenStream2> {
    let mut out = Vec::new();
    if is_bool_type(field_ty) {
        out.push(quote! {
            pub fn #field_name(mut self) -> Self {
                self.#prefix #field_name = true;
                self
            }
        });
    } else {
        match extract_option_box_fn(field_ty) {
            Some(fn_args) => {
                out.push(quote! {
                    pub fn #field_name(mut self, f: impl Fn(#fn_args) + 'static) -> Self {
                        self.#prefix #field_name = Some(Box::new(f));
                        self
                    }
                });
            }
            _ => {
                if let Ok(inner_ty) = extract_option_inner(field_ty) {
                    out.push(quote! {
                        pub fn #field_name(mut self, v: #inner_ty) -> Self {
                            self.#prefix #field_name = Some(v);
                            self
                        }
                    });
                } else {
                    out.push(quote! {
                        pub fn #field_name(mut self, v: #field_ty) -> Self {
                            self.#prefix #field_name = v;
                            self
                        }
                    });
                }
            }
        }
    }
    out
}

fn gen_method_for_field(
    field_name: &syn::Ident,
    field_ty: &Type,
    attr: &syn::Attribute,
    fields: &syn::punctuated::Punctuated<syn::Field, syn::token::Comma>,
    prefix: &TokenStream2,
) -> syn::Result<Vec<TokenStream2>> {
    let mut out = Vec::new();

    if attr.meta.require_path_only().is_ok() {
        if is_bool_type(field_ty) {
            out.push(quote! {
                pub fn #field_name(mut self) -> Self {
                    self.#prefix #field_name = true;
                    self
                }
            });
        } else {
            match extract_option_box_fn(field_ty) {
                Some(fn_args) => {
                    out.push(quote! {
                        pub fn #field_name(mut self, f: impl Fn(#fn_args) + 'static) -> Self {
                            self.#prefix #field_name = Some(Box::new(f));
                            self
                        }
                    });
                }
                _ => {
                    if let Ok(inner_ty) = extract_option_inner(field_ty) {
                        out.push(quote! {
                            pub fn #field_name(mut self, v: #inner_ty) -> Self {
                                self.#prefix #field_name = Some(v);
                                self
                            }
                        });
                    } else {
                        out.push(quote! {
                            pub fn #field_name(mut self, v: #field_ty) -> Self {
                                self.#prefix #field_name = v;
                                self
                            }
                        });
                    }
                }
            }
        }
        return Ok(out);
    }

    let list = attr.meta.require_list()?;
    let tokens = list.tokens.to_string();

    if tokens.starts_with("dual_variant") {
        let var = parse_combo_variant_attr(&tokens)?;
        let method_name = syn::Ident::new(&var.name, field_name.span());
        let variant_ident = syn::Ident::new(&var.variant, field_name.span());
        let enum_ty = field_ty;

        let params: Vec<(syn::Ident, syn::Type)> = parse_args_list(&var.args, field_name.span())?;
        let param_names: Vec<&syn::Ident> = params.iter().map(|(n, _)| n).collect();
        let param_decls: Vec<TokenStream2> =
            params.iter().map(|(n, t)| quote! { #n: #t }).collect();
        let assignments: Vec<TokenStream2> = param_names
            .iter()
            .map(|name| quote! { self.#prefix #name = #enum_ty::#variant_ident(#name); })
            .collect();
        out.push(quote! {
            pub fn #method_name(mut self, #(#param_decls),*) -> Self {
                #(#assignments)*
                self
            }
        });
    } else if tokens.starts_with("combo") {
        let combo = parse_combo_attr(&tokens)?;
        let method_name = syn::Ident::new(&combo.name, field_name.span());
        let param_names: Vec<syn::Ident> = combo
            .fields
            .iter()
            .map(|f| syn::Ident::new(f, field_name.span()))
            .collect();
        let param_types: Vec<TokenStream2> = combo
            .fields
            .iter()
            .map(|f| {
                let field = fields
                    .iter()
                    .find(|fld| fld.ident.as_ref().unwrap() == f)
                    .unwrap();
                let inner = extract_option_inner(&field.ty).unwrap();
                quote! { #inner }
            })
            .collect();
        let assignments: Vec<TokenStream2> = param_names
            .iter()
            .map(|name| quote! { self.#prefix #name = Some(#name); })
            .collect();
        out.push(quote! {
            pub fn #method_name(mut self, #(#param_names: #param_types),*) -> Self {
                #(#assignments)*
                self
            }
        });
    } else if tokens.starts_with("variant") {
        let var = parse_variant_attr(&tokens)?;
        let method_name = syn::Ident::new(&var.name, field_name.span());
        let variant_ident = syn::Ident::new(&var.variant, field_name.span());
        let enum_ty = field_ty;
        if let Some(args) = var.args {
            let params: Vec<(syn::Ident, syn::Type)> = parse_args_list(&args, field_name.span())?;
            let param_names: Vec<&syn::Ident> = params.iter().map(|(n, _)| n).collect();
            let param_decls: Vec<TokenStream2> =
                params.iter().map(|(n, t)| quote! { #n: #t }).collect();
            out.push(quote! {
                pub fn #method_name(mut self, #(#param_decls),*) -> Self {
                    self.#prefix #field_name = #enum_ty::#variant_ident(#(#param_names),*);
                    self
                }
            });
        } else {
            out.push(quote! {
                pub fn #method_name(mut self) -> Self {
                    self.#prefix #field_name = #enum_ty::#variant_ident;
                    self
                }
            });
        }
    }

    Ok(out)
}

struct ForwardTarget {
    ty: String,
    via: String,
}

fn parse_forward_targets(input: &DeriveInput) -> Vec<ForwardTarget> {
    let mut targets = Vec::new();
    for attr in &input.attrs {
        if !attr.path().is_ident("builders") {
            continue;
        }
        if let Ok(list) = attr.meta.require_list() {
            let tokens = list.tokens.to_string();
            if tokens.starts_with("forward") {
                let after_keyword = tokens
                    .strip_prefix("forward")
                    .unwrap_or(&tokens)
                    .trim_start();
                if let Some(inner) = after_keyword
                    .strip_prefix('(')
                    .and_then(|s| s.strip_suffix(')'))
                {
                    let mut to = String::new();
                    let mut via = String::new();
                    let mut remaining = inner;
                    while !remaining.is_empty() {
                        remaining = remaining.trim_start();
                        if remaining.is_empty() {
                            break;
                        }
                        if let Some(eq_pos) = remaining.find('=') {
                            let key = remaining[..eq_pos].trim().trim_start_matches(',').trim();
                            remaining = remaining[eq_pos + 1..].trim();
                            if remaining.starts_with('"') {
                                let end_quote = remaining[1..].find('"').unwrap() + 1;
                                let val = &remaining[1..end_quote];
                                remaining = &remaining[end_quote + 1..];
                                remaining = remaining.trim_start_matches(',').trim_start();
                                match key {
                                    "to" => to = val.to_string(),
                                    "via" => via = val.to_string(),
                                    _ => {}
                                }
                            }
                        } else {
                            break;
                        }
                    }
                    if !to.is_empty() && !via.is_empty() {
                        targets.push(ForwardTarget { ty: to, via });
                    }
                }
            }
        }
    }
    targets
}

fn impl_builders(input: &DeriveInput) -> syn::Result<TokenStream2> {
    let fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(named) => &named.named,
            _ => return Err(syn::Error::new_spanned(input, "expected named fields")),
        },
        _ => return Err(syn::Error::new_spanned(input, "expected struct")),
    };

    let empty_prefix = quote! {};
    let mut methods = Vec::new();

    for field in fields {
        let field_name = field.ident.as_ref().unwrap();
        let builder_attrs: Vec<&syn::Attribute> = field
            .attrs
            .iter()
            .filter(|a| a.path().is_ident("builder"))
            .collect();

        if builder_attrs.is_empty() {
            let generated = gen_default_method_for_field(field_name, &field.ty, &empty_prefix);
            methods.extend(generated);
        } else {
            for attr in builder_attrs {
                if let Ok(list) = attr.meta.require_list() {
                    if list.tokens.to_string().trim() == "omit" {
                        continue;
                    }
                }
                let generated =
                    gen_method_for_field(field_name, &field.ty, attr, fields, &empty_prefix)?;
                methods.extend(generated);
            }
        }
    }

    let struct_name = &input.ident;
    let generics = &input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    // Generate forward impls
    let targets = parse_forward_targets(input);
    let mut forward_impls = Vec::new();

    for target in &targets {
        let target_ty: syn::Type = syn::parse_str(&target.ty)?;
        let via_expr: syn::Expr = syn::parse_str(&target.via)?;
        let prefix = quote! { #via_expr. };

        let mut fwd_methods = Vec::new();
        for field in fields {
            let field_name = field.ident.as_ref().unwrap();
            let builder_attrs: Vec<&syn::Attribute> = field
                .attrs
                .iter()
                .filter(|a| a.path().is_ident("builder"))
                .collect();

            if builder_attrs.is_empty() {
                let generated = gen_default_method_for_field(field_name, &field.ty, &prefix);
                fwd_methods.extend(generated);
            } else {
                for attr in builder_attrs {
                    if let Ok(list) = attr.meta.require_list() {
                        if list.tokens.to_string().trim() == "omit" {
                            continue;
                        }
                    }
                    let generated =
                        gen_method_for_field(field_name, &field.ty, attr, fields, &prefix)?;
                    fwd_methods.extend(generated);
                }
            }
        }

        // Parse generics from the target type string (e.g. "PaneNode<A: App>")
        // We need to extract the impl generics
        let target_generics = extract_target_generics(&target.ty);

        forward_impls.push(if let Some((impl_gen, target_bare)) = target_generics {
            let impl_gen_ts: TokenStream2 = impl_gen.parse().unwrap_or_default();
            let target_bare_ty: syn::Type = syn::parse_str(&target_bare)?;
            quote! {
                impl <#impl_gen_ts> #target_bare_ty {
                    #(#fwd_methods)*
                }
            }
        } else {
            quote! {
                impl #target_ty {
                    #(#fwd_methods)*
                }
            }
        });
    }

    Ok(quote! {
        impl #impl_generics #struct_name #ty_generics #where_clause {
            #(#methods)*
        }

        #(#forward_impls)*
    })
}

fn extract_target_generics(ty_str: &str) -> Option<(String, String)> {
    // "PaneNode<A: App>" -> Some(("A: App", "PaneNode<A>"))
    let open = ty_str.find('<')?;
    let close = ty_str.rfind('>')?;
    let name = &ty_str[..open];
    let params_full = &ty_str[open + 1..close]; // "A: App"

    // Strip bounds for the bare type: "A: App" -> "A"
    let params_bare: Vec<&str> = params_full
        .split(',')
        .map(|p| {
            let p = p.trim();
            p.split(':').next().unwrap().trim()
        })
        .collect();
    let bare = format!("{}<{}>", name, params_bare.join(", "));

    Some((params_full.to_string(), bare))
}

/// Checks if the type is `Option<Box<dyn Fn(A, B, ...) [+ 'static]>>`.
/// Returns the argument list tokens if so.
fn extract_option_box_fn(ty: &Type) -> Option<TokenStream2> {
    // Option<...>
    let inner = extract_option_inner(ty).ok()?;
    // Box<dyn Fn(...)>
    if let Type::Path(type_path) = inner {
        let seg = type_path.path.segments.last()?;
        if seg.ident != "Box" {
            return None;
        }
        if let syn::PathArguments::AngleBracketed(args) = &seg.arguments {
            if let Some(syn::GenericArgument::Type(Type::TraitObject(trait_obj))) =
                args.args.first()
            {
                // Find the Fn(...) bound
                for bound in &trait_obj.bounds {
                    if let syn::TypeParamBound::Trait(trait_bound) = bound {
                        let last_seg = trait_bound.path.segments.last()?;
                        if last_seg.ident == "Fn"
                            || last_seg.ident == "FnMut"
                            || last_seg.ident == "FnOnce"
                        {
                            if let syn::PathArguments::Parenthesized(paren) = &last_seg.arguments {
                                let inputs = &paren.inputs;
                                return Some(quote! { #inputs });
                            }
                        }
                    }
                }
            }
        }
    }
    None
}

fn is_bool_type(ty: &Type) -> bool {
    if let Type::Path(type_path) = ty {
        if let Some(seg) = type_path.path.segments.last() {
            return seg.ident == "bool";
        }
    }
    false
}

fn extract_option_inner(ty: &Type) -> syn::Result<&Type> {
    if let Type::Path(type_path) = ty {
        if let Some(seg) = type_path.path.segments.last() {
            if seg.ident == "Option" {
                if let syn::PathArguments::AngleBracketed(args) = &seg.arguments {
                    if let Some(syn::GenericArgument::Type(inner)) = args.args.first() {
                        return Ok(inner);
                    }
                }
            }
        }
    }
    Err(syn::Error::new_spanned(ty, "expected Option<T>"))
}

struct ComboAttr {
    name: String,
    fields: Vec<String>,
}

fn parse_combo_attr(s: &str) -> syn::Result<ComboAttr> {
    // combo(name = "size", fields = "w, h")
    // proc_macro2 may insert space: "combo (...)"
    let s_trimmed = s.trim();
    let after_keyword = s_trimmed
        .strip_prefix("combo")
        .unwrap_or(s_trimmed)
        .trim_start();
    let inner = after_keyword
        .strip_prefix('(')
        .and_then(|s| s.strip_suffix(')'))
        .ok_or_else(|| syn::Error::new(proc_macro2::Span::call_site(), "invalid combo attr"))?;
    let mut name = String::new();
    let mut fields = Vec::new();

    let mut remaining = inner;
    while !remaining.is_empty() {
        remaining = remaining.trim_start();
        if remaining.is_empty() {
            break;
        }
        if let Some(eq_pos) = remaining.find('=') {
            let key = remaining[..eq_pos].trim().trim_start_matches(',').trim();
            remaining = remaining[eq_pos + 1..].trim();
            if remaining.starts_with('"') {
                let end_quote = remaining[1..].find('"').unwrap() + 1;
                let val = &remaining[1..end_quote];
                remaining = &remaining[end_quote + 1..];
                remaining = remaining.trim_start_matches(',').trim_start();
                match key {
                    "name" => name = val.to_string(),
                    "fields" => {
                        fields = val
                            .split(',')
                            .map(|s| s.trim().to_string())
                            .filter(|s| !s.is_empty())
                            .collect();
                    }
                    _ => {}
                }
            }
        } else {
            break;
        }
    }
    Ok(ComboAttr { name, fields })
}

struct ComboVariantAttr {
    name: String,
    variant: String,
    args: String,
}

fn parse_combo_variant_attr(s: &str) -> syn::Result<ComboVariantAttr> {
    // combo_variant(name = "size", variant = "Fixed", args = "w: u32, h: u32")
    // proc_macro2 may insert space: "combo (...)"
    let s_trimmed = s.trim();
    let after_keyword = s_trimmed
        .strip_prefix("dual_variant")
        .unwrap_or(s_trimmed)
        .trim_start();
    let inner = after_keyword
        .strip_prefix('(')
        .and_then(|s| s.strip_suffix(')'))
        .ok_or_else(|| {
            syn::Error::new(proc_macro2::Span::call_site(), "invalid combo variant attr")
        })?;
    let mut name = String::new();
    let mut variant = String::new();
    let mut args = String::new();

    let mut remaining = inner;
    while !remaining.is_empty() {
        remaining = remaining.trim_start();
        if remaining.is_empty() {
            break;
        }
        if let Some(eq_pos) = remaining.find('=') {
            let key = remaining[..eq_pos].trim().trim_start_matches(',').trim();
            remaining = remaining[eq_pos + 1..].trim();
            if remaining.starts_with('"') {
                let end_quote = remaining[1..].find('"').unwrap() + 1;
                let val = &remaining[1..end_quote];
                remaining = &remaining[end_quote + 1..];
                remaining = remaining.trim_start_matches(',').trim_start();
                match key {
                    "name" => name = val.to_string(),
                    "variant" => variant = val.to_string(),
                    "args" => args = val.to_string(),
                    _ => {}
                }
            }
        } else {
            break;
        }
    }
    Ok(ComboVariantAttr {
        name,
        variant,
        args,
    })
}

struct VariantAttr {
    name: String,
    variant: String,
    args: Option<String>,
}

fn parse_variant_attr(s: &str) -> syn::Result<VariantAttr> {
    // variant(name = "relative", variant = "Relative", args = "x: i32, y: i32")
    // proc_macro2 may insert space: "variant (...)"
    let s_trimmed = s.trim();
    let after_keyword = s_trimmed
        .strip_prefix("variant")
        .unwrap_or(s_trimmed)
        .trim_start();
    let inner = after_keyword
        .strip_prefix('(')
        .and_then(|s| s.strip_suffix(')'))
        .ok_or_else(|| syn::Error::new(proc_macro2::Span::call_site(), "invalid variant attr"))?;
    let mut name = String::new();
    let mut variant = String::new();
    let mut args: Option<String> = None;

    // Parse key = "value" pairs carefully (args value contains commas)
    let mut remaining = inner;
    while !remaining.is_empty() {
        remaining = remaining.trim_start();
        if remaining.is_empty() {
            break;
        }
        if let Some(eq_pos) = remaining.find('=') {
            let key = remaining[..eq_pos].trim().trim_start_matches(',').trim();
            remaining = remaining[eq_pos + 1..].trim();
            if remaining.starts_with('"') {
                let end_quote = remaining[1..].find('"').unwrap() + 1;
                let val = &remaining[1..end_quote];
                remaining = &remaining[end_quote + 1..];
                remaining = remaining.trim_start_matches(',').trim_start();
                match key {
                    "name" => name = val.to_string(),
                    "variant" => variant = val.to_string(),
                    "args" => args = Some(val.to_string()),
                    _ => {}
                }
            }
        } else {
            break;
        }
    }

    Ok(VariantAttr {
        name,
        variant,
        args,
    })
}

fn parse_args_list(
    args: &str,
    span: proc_macro2::Span,
) -> syn::Result<Vec<(syn::Ident, syn::Type)>> {
    let mut result = Vec::new();
    for part in args.split(',') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        let colon_pos = part.find(':').ok_or_else(|| {
            syn::Error::new(
                span,
                format!("expected 'name: type' in args, got '{}'", part),
            )
        })?;
        let name = part[..colon_pos].trim();
        let ty_str = part[colon_pos + 1..].trim();
        let ident = syn::Ident::new(name, span);
        let ty: syn::Type = syn::parse_str(ty_str)?;
        result.push((ident, ty));
    }
    Ok(result)
}

struct DelegateArgs {
    target_ty: Path,
    getter: Ident,
}

impl Parse for DelegateArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let target_ty: Path = input.parse()?;
        input.parse::<Token![,]>()?;
        let getter_lit: LitStr = input.parse()?;
        let getter = Ident::new(&getter_lit.value(), getter_lit.span());
        Ok(DelegateArgs { target_ty, getter })
    }
}

#[proc_macro_attribute]
pub fn delegate(attr: TokenStream, item: TokenStream) -> TokenStream {
    let DelegateArgs { target_ty, getter } = parse_macro_input!(attr as DelegateArgs);
    let input = parse_macro_input!(item as ItemImpl);

    let mut forwarded = Vec::new();

    for member in &input.items {
        let ImplItem::Fn(method) = member else {
            continue;
        };

        let skip = method
            .attrs
            .iter()
            .any(|a| a.path().is_ident("no_delegate"));
        if skip {
            continue;
        }

        let sig = &method.sig;

        let is_ref_self = sig.inputs.iter().any(|a| match a {
            FnArg::Receiver(r) => r.reference.is_some() && r.mutability.is_none(),
            _ => false,
        });
        if !is_ref_self {
            continue;
        }

        let name = &sig.ident;
        let inputs = &sig.inputs;
        let output = &sig.output;
        let generics = &sig.generics;

        let arg_names: Vec<_> = inputs
            .iter()
            .filter_map(|arg| match arg {
                FnArg::Typed(pat_ty) => match &*pat_ty.pat {
                    Pat::Ident(p) => Some(p.ident.clone()),
                    _ => None,
                },
                FnArg::Receiver(_) => None,
            })
            .collect();

        forwarded.push(quote! {
            pub fn #name #generics (#inputs) #output {
                self.#getter().#name(#(#arg_names),*)
            }
        });
    }

    let mut cleaned = input.clone();
    for item in &mut cleaned.items {
        if let ImplItem::Fn(method) = item {
            method.attrs.retain(|a| !a.path().is_ident("no_delegate"));
        }
    }

    let expanded = quote! {
        #cleaned

        impl #target_ty {
            #(#forwarded)*
        }
    };

    expanded.into()
}
