use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::{
    Attribute, Ident, Result, Token, Type, braced, parenthesized,
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
};

/// Handler type for method thunks
#[derive(Clone)]
enum Handler {
    /// Simple passthrough - just forward the call
    Passthrough,
    /// Release handler - cleanup wrapper when refcount hits 0
    Release,
    /// Cast handler - cast typed pointers to void*
    Cast,
    /// Wrap handler - wrap return value with given function
    Wrap(Ident),
    /// Unwrap handler - unwrap wrapper arg before passing to inner
    /// (wrapper_type, arg_name) - the arg to unwrap
    Unwrap(Ident, Ident),
}

/// A single method in the COM interface
struct Method {
    name: Ident,
    args: Vec<(Ident, Type)>,
    ret: Type,
    handler: Handler,
}

/// The full com_wrapper input
struct ComWrapper {
    attrs: Vec<Attribute>,
    wrapper_name: Ident,
    inner_type: Ident,
    public_type: Ident,
    vtable_name: Ident,
    vtable_type: Ident,
    methods: Vec<Method>,
}

impl Parse for ComWrapper {
    fn parse(input: ParseStream) -> Result<Self> {
        // Parse attributes (doc comments)
        let attrs = input.call(Attribute::parse_outer)?;

        // WrapperName wraps InnerType as PublicType
        let wrapper_name: Ident = input.parse()?;
        let wraps: Ident = input.parse()?;
        if wraps != "wraps" {
            return Err(syn::Error::new(wraps.span(), "expected `wraps`"));
        }
        let inner_type: Ident = input.parse()?;
        input.parse::<Token![as]>()?;
        let public_type: Ident = input.parse()?;

        // { ... }
        let content;
        braced!(content in input);

        // vtable: VTABLE_NAME: VtableType,
        let vtable_kw: Ident = content.parse()?;
        if vtable_kw != "vtable" {
            return Err(syn::Error::new(vtable_kw.span(), "expected `vtable`"));
        }
        content.parse::<Token![:]>()?;
        let vtable_name: Ident = content.parse()?;
        content.parse::<Token![:]>()?;
        let vtable_type: Ident = content.parse()?;
        content.parse::<Token![,]>()?;

        // Parse methods
        let mut methods = Vec::new();
        while !content.is_empty() {
            // fn name(args) -> RetType [=> handler];
            content.parse::<Token![fn]>()?;
            let name: Ident = content.parse()?;

            // Parse args
            let args_content;
            parenthesized!(args_content in content);
            let args_parsed: Punctuated<(Ident, Type), Token![,]> = args_content.parse_terminated(
                |input| {
                    let name: Ident = input.parse()?;
                    input.parse::<Token![:]>()?;
                    let ty: Type = input.parse()?;
                    Ok((name, ty))
                },
                Token![,],
            )?;
            let args: Vec<_> = args_parsed.into_iter().collect();

            // -> RetType
            content.parse::<Token![->]>()?;
            let ret: Type = content.parse()?;

            // Optional => handler
            let handler = if content.peek(Token![=>]) {
                content.parse::<Token![=>]>()?;
                let handler_name: Ident = content.parse()?;
                match handler_name.to_string().as_str() {
                    "release" => Handler::Release,
                    "cast" => Handler::Cast,
                    "wrap" => {
                        // wrap(function_name)
                        let wrap_content;
                        parenthesized!(wrap_content in content);
                        let wrap_fn: Ident = wrap_content.parse()?;
                        Handler::Wrap(wrap_fn)
                    }
                    "unwrap" => {
                        // unwrap(WrapperType, arg_name)
                        let unwrap_content;
                        parenthesized!(unwrap_content in content);
                        let wrapper_ty: Ident = unwrap_content.parse()?;
                        unwrap_content.parse::<Token![,]>()?;
                        let arg_name: Ident = unwrap_content.parse()?;
                        Handler::Unwrap(wrapper_ty, arg_name)
                    }
                    other => {
                        return Err(syn::Error::new(
                            handler_name.span(),
                            format!("unknown handler: {}", other),
                        ));
                    }
                }
            } else {
                Handler::Passthrough
            };

            content.parse::<Token![;]>()?;

            methods.push(Method {
                name,
                args,
                ret,
                handler,
            });
        }

        Ok(ComWrapper {
            attrs,
            wrapper_name,
            inner_type,
            public_type,
            vtable_name,
            vtable_type,
            methods,
        })
    }
}

fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() {
            if i > 0 {
                result.push('_');
            }
            result.push(c.to_lowercase().next().unwrap());
        } else {
            result.push(c);
        }
    }
    result
}

fn generate_vtable_field(public_type: &Ident, method: &Method) -> TokenStream2 {
    let method_name = &method.name;
    let ret = &method.ret;
    let arg_types: Vec<_> = method.args.iter().map(|(_, ty)| ty).collect();

    quote! {
        pub #method_name: unsafe extern "C" fn(*mut #public_type, #(#arg_types),*) -> #ret
    }
}

fn generate_win64_vtable_field(inner_type: &Ident, method: &Method) -> TokenStream2 {
    let method_name = &method.name;
    let ret = &method.ret;
    let arg_types: Vec<_> = method.args.iter().map(|(_, ty)| ty).collect();

    quote! {
        pub #method_name: unsafe extern "win64" fn(*mut #inner_type, #(#arg_types),*) -> #ret
    }
}

fn generate_thunk(
    wrapper_name: &Ident,
    _inner_type: &Ident,
    public_type: &Ident,
    method: &Method,
) -> TokenStream2 {
    let method_name = &method.name;
    let fn_name = format_ident!(
        "{}_{}",
        to_snake_case(&wrapper_name.to_string()),
        to_snake_case(&method_name.to_string())
    );
    let ret = &method.ret;

    let arg_names: Vec<_> = method.args.iter().map(|(name, _)| name).collect();
    let arg_types: Vec<_> = method.args.iter().map(|(_, ty)| ty).collect();

    let args_def = if arg_names.is_empty() {
        quote! {}
    } else {
        quote! { , #(#arg_names: #arg_types),* }
    };

    match &method.handler {
        Handler::Passthrough => {
            let args_pass = if arg_names.is_empty() {
                quote! {}
            } else {
                quote! { , #(#arg_names),* }
            };
            quote! {
                unsafe extern "C" fn #fn_name(
                    this: *mut #public_type
                    #args_def
                ) -> #ret {
                    let wrapper = this as *mut #wrapper_name;
                    ((*(*(*wrapper).inner).vtable).#method_name)((*wrapper).inner #args_pass)
                }
            }
        }
        Handler::Release => {
            quote! {
                unsafe extern "C" fn #fn_name(this: *mut #public_type) -> #ret {
                    let wrapper = this as *mut #wrapper_name;
                    let inner = (*wrapper).inner;
                    let count = ((*(*inner).vtable).#method_name)(inner);
                    if count == 0 {
                        drop(Box::from_raw(wrapper));
                    }
                    count
                }
            }
        }
        Handler::Cast => {
            let args_cast = if arg_names.is_empty() {
                quote! {}
            } else {
                quote! { , #(#arg_names as _),* }
            };
            quote! {
                unsafe extern "C" fn #fn_name(
                    this: *mut #public_type
                    #args_def
                ) -> #ret {
                    let wrapper = this as *mut #wrapper_name;
                    ((*(*(*wrapper).inner).vtable).#method_name)((*wrapper).inner #args_cast)
                }
            }
        }
        Handler::Wrap(wrap_fn) => {
            let args_pass = if arg_names.is_empty() {
                quote! {}
            } else {
                quote! { , #(#arg_names),* }
            };
            quote! {
                unsafe extern "C" fn #fn_name(
                    this: *mut #public_type
                    #args_def
                ) -> #ret {
                    let wrapper = this as *mut #wrapper_name;
                    let result = ((*(*(*wrapper).inner).vtable).#method_name)((*wrapper).inner #args_pass);
                    #wrap_fn(result as _)
                }
            }
        }
        Handler::Unwrap(unwrap_wrapper_ty, unwrap_arg) => {
            // Generate args, replacing the unwrap_arg with unwrapped version
            let args_pass: Vec<_> = arg_names
                .iter()
                .map(|name| {
                    if *name == unwrap_arg {
                        quote! {
                            if #name.is_null() {
                                std::ptr::null_mut()
                            } else {
                                (*(#name as *mut #unwrap_wrapper_ty)).inner as _
                            }
                        }
                    } else {
                        quote! { #name }
                    }
                })
                .collect();
            let args_pass = if args_pass.is_empty() {
                quote! {}
            } else {
                quote! { , #(#args_pass),* }
            };
            quote! {
                unsafe extern "C" fn #fn_name(
                    this: *mut #public_type
                    #args_def
                ) -> #ret {
                    let wrapper = this as *mut #wrapper_name;
                    ((*(*(*wrapper).inner).vtable).#method_name)((*wrapper).inner #args_pass)
                }
            }
        }
    }
}

#[proc_macro]
pub fn com_wrapper(input: TokenStream) -> TokenStream {
    let wrapper = parse_macro_input!(input as ComWrapper);

    let attrs = &wrapper.attrs;
    let wrapper_name = &wrapper.wrapper_name;
    let inner_type = &wrapper.inner_type;
    let public_type = &wrapper.public_type;
    let vtable_name = &wrapper.vtable_name;
    let vtable_type = &wrapper.vtable_type;

    // Derive inner vtable type name (e.g., Win64Blob -> Win64BlobVtbl)
    let inner_vtable_type = format_ident!("{}Vtbl", inner_type);

    // Derive wrap function name (e.g., BlobWrapper -> wrap_blob)
    let wrapper_str = wrapper_name.to_string();
    let base_name = wrapper_str.strip_suffix("Wrapper").unwrap_or(&wrapper_str);
    let wrap_fn_name = format_ident!("wrap_{}", to_snake_case(base_name));

    // Generate thunk functions
    let thunks: Vec<_> = wrapper
        .methods
        .iter()
        .map(|m| generate_thunk(wrapper_name, inner_type, public_type, m))
        .collect();

    // Generate public vtable struct fields (C ABI)
    let vtable_struct_fields: Vec<_> = wrapper
        .methods
        .iter()
        .map(|m| generate_vtable_field(public_type, m))
        .collect();

    // Generate win64 vtable struct fields
    let win64_vtable_struct_fields: Vec<_> = wrapper
        .methods
        .iter()
        .map(|m| generate_win64_vtable_field(inner_type, m))
        .collect();

    // Generate vtable field assignments (instance initialization)
    let vtable_init_fields: Vec<_> = wrapper
        .methods
        .iter()
        .map(|m| {
            let method_name = &m.name;
            let fn_name = format_ident!(
                "{}_{}",
                to_snake_case(&wrapper_name.to_string()),
                to_snake_case(&method_name.to_string())
            );
            quote! { #method_name: #fn_name }
        })
        .collect();

    let expanded = quote! {
        // Win64 inner type (from Windows DLL)
        #[repr(C)]
        struct #inner_type {
            vtable: *const #inner_vtable_type,
        }

        #[repr(C)]
        struct #inner_vtable_type {
            #(#win64_vtable_struct_fields),*
        }

        // Public interface type with C ABI
        #[repr(C)]
        pub struct #public_type {
            pub vtable: *const #vtable_type,
        }

        #[repr(C)]
        pub struct #vtable_type {
            #(#vtable_struct_fields),*
        }

        // Wrapper struct
        #(#attrs)*
        #[repr(C)]
        struct #wrapper_name {
            vtable: *const #vtable_type,
            inner: *mut #inner_type,
        }

        #(#thunks)*

        static #vtable_name: #vtable_type = #vtable_type {
            #(#vtable_init_fields),*
        };

        // Wrap function to create a C ABI wrapper around a Win64 object
        unsafe fn #wrap_fn_name(inner: *mut #inner_type) -> *mut #public_type {
            if inner.is_null() {
                return std::ptr::null_mut();
            }
            let wrapper = Box::new(#wrapper_name {
                vtable: &#vtable_name,
                inner,
            });
            Box::into_raw(wrapper) as *mut #public_type
        }
    };

    TokenStream::from(expanded)
}
