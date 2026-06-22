use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::token::Brace;
use syn::{braced, Expr, Ident, Pat, Token};

// =============================================================================
// ui! macro
// =============================================================================

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
            let mut __children: Vec<crate::ui::Node<_>> = Vec::new();
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
                    __children.push(#expr.children(#children_expr).into());
                }
            } else {
                quote! {
                    __children.push(#expr.into());
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
                    __children.push(#pat.into());
                }
            } else {
                quote! {
                    let #pat = #widget_expr;
                    __children.push(#pat.into());
                }
            }
        }
    }
}

/// Builds a `Vec<Node<A>>` from a UI description block.
#[proc_macro]
pub fn ui(input: TokenStream) -> TokenStream {
    let block = syn::parse_macro_input!(input as UiBlock);
    let expanded = expand_block(&block);
    TokenStream::from(expanded)
}

// =============================================================================
// #[derive(Builders)] proc macro
// =============================================================================
//
// Generates builder methods from struct fields based on attributes:
//
//   #[builder]              — Option<T> field → fn field(mut self, v: T) -> Self
//   #[builder(bool)]        — bool field → fn field(mut self) -> Self
//   #[builder(combo(name = "size", fields = "w, h"))]
//                           — generates fn size(mut self, w: T, h: T) -> Self
//   #[builder(variant(name = "relative", variant = "Relative", args = "x: i32, y: i32"))]
//                           — generates fn relative(mut self, x: i32, y: i32) -> Self
//                             setting field = EnumType::Relative(x, y)
//
// The generated impl targets a different type (the "owner") specified via
// a struct-level attribute:
//   #[builders(owner = "PaneNode<A: App>", access = "self.style")]

use syn::{Data, DeriveInput, Fields, Type};

#[proc_macro_derive(Builders, attributes(builder, builders))]
pub fn derive_builders(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as DeriveInput);
    match impl_builders(&input) {
        Ok(ts) => ts.into(),
        Err(e) => e.to_compile_error().into(),
    }
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
        } else if let Some(fn_args) = extract_option_box_fn(field_ty) {
            out.push(quote! {
                pub fn #field_name(mut self, f: impl Fn(#fn_args) + 'static) -> Self {
                    self.#prefix #field_name = Some(Box::new(f));
                    self
                }
            });
        } else if let Ok(inner_ty) = extract_option_inner(field_ty) {
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
        return Ok(out);
    }

    let list = attr.meta.require_list()?;
    let tokens = list.tokens.to_string();

    if tokens.starts_with("combo") {
        let combo = parse_combo_attr(&tokens)?;
        let method_name = syn::Ident::new(&combo.name, field_name.span());
        let param_names: Vec<syn::Ident> = combo.fields.iter()
            .map(|f| syn::Ident::new(f, field_name.span()))
            .collect();
        let param_types: Vec<TokenStream2> = combo.fields.iter()
            .map(|f| {
                let field = fields.iter().find(|fld| fld.ident.as_ref().unwrap() == f).unwrap();
                let inner = extract_option_inner(&field.ty).unwrap();
                quote! { #inner }
            })
            .collect();
        let assignments: Vec<TokenStream2> = param_names.iter()
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
        let params: Vec<(syn::Ident, syn::Type)> = parse_args_list(&var.args, field_name.span())?;
        let param_names: Vec<&syn::Ident> = params.iter().map(|(n, _)| n).collect();
        let param_decls: Vec<TokenStream2> = params.iter()
            .map(|(n, t)| quote! { #n: #t })
            .collect();
        out.push(quote! {
            pub fn #method_name(mut self, #(#param_decls),*) -> Self {
                self.#prefix #field_name = #enum_ty::#variant_ident(#(#param_names),*);
                self
            }
        });
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
                let after_keyword = tokens.strip_prefix("forward").unwrap_or(&tokens).trim_start();
                if let Some(inner) = after_keyword.strip_prefix('(').and_then(|s| s.strip_suffix(')')) {
                    let mut to = String::new();
                    let mut via = String::new();
                    let mut remaining = inner;
                    while !remaining.is_empty() {
                        remaining = remaining.trim_start();
                        if remaining.is_empty() { break; }
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
        for attr in &field.attrs {
            if !attr.path().is_ident("builder") {
                continue;
            }
            let generated = gen_method_for_field(field_name, &field.ty, attr, fields, &empty_prefix)?;
            methods.extend(generated);
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
            for attr in &field.attrs {
                if !attr.path().is_ident("builder") {
                    continue;
                }
                let generated = gen_method_for_field(field_name, &field.ty, attr, fields, &prefix)?;
                fwd_methods.extend(generated);
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
    let params_bare: Vec<&str> = params_full.split(',')
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
            if let Some(syn::GenericArgument::Type(Type::TraitObject(trait_obj))) = args.args.first() {
                // Find the Fn(...) bound
                for bound in &trait_obj.bounds {
                    if let syn::TypeParamBound::Trait(trait_bound) = bound {
                        let last_seg = trait_bound.path.segments.last()?;
                        if last_seg.ident == "Fn" || last_seg.ident == "FnMut" || last_seg.ident == "FnOnce" {
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
    let after_keyword = s_trimmed.strip_prefix("combo").unwrap_or(s_trimmed).trim_start();
    let inner = after_keyword.strip_prefix('(').and_then(|s| s.strip_suffix(')'))
        .ok_or_else(|| syn::Error::new(proc_macro2::Span::call_site(), "invalid combo attr"))?;
    let mut name = String::new();
    let mut fields = Vec::new();

    let mut remaining = inner;
    while !remaining.is_empty() {
        remaining = remaining.trim_start();
        if remaining.is_empty() { break; }
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
                        fields = val.split(',')
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

struct VariantAttr {
    name: String,
    variant: String,
    args: String,
}

fn parse_variant_attr(s: &str) -> syn::Result<VariantAttr> {
    // variant(name = "relative", variant = "Relative", args = "x: i32, y: i32")
    // proc_macro2 may insert space: "variant (...)"
    let s_trimmed = s.trim();
    let after_keyword = s_trimmed.strip_prefix("variant").unwrap_or(s_trimmed).trim_start();
    let inner = after_keyword.strip_prefix('(').and_then(|s| s.strip_suffix(')'))
        .ok_or_else(|| syn::Error::new(proc_macro2::Span::call_site(), "invalid variant attr"))?;
    let mut name = String::new();
    let mut variant = String::new();
    let mut args = String::new();

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
                    "args" => args = val.to_string(),
                    _ => {}
                }
            }
        } else {
            break;
        }
    }

    Ok(VariantAttr { name, variant, args })
}

fn parse_args_list(args: &str, span: proc_macro2::Span) -> syn::Result<Vec<(syn::Ident, syn::Type)>> {
    let mut result = Vec::new();
    for part in args.split(',') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        let colon_pos = part.find(':')
            .ok_or_else(|| syn::Error::new(span, format!("expected 'name: type' in args, got '{}'", part)))?;
        let name = part[..colon_pos].trim();
        let ty_str = part[colon_pos + 1..].trim();
        let ident = syn::Ident::new(name, span);
        let ty: syn::Type = syn::parse_str(ty_str)?;
        result.push((ident, ty));
    }
    Ok(result)
}
