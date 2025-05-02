use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, format_ident};
use syn::{
    parse_macro_input, FnArg, Ident, ItemFn, Lit, Meta, PatType, Type,
    parse_quote, LitStr, Expr,
};

struct ArgInfo {
    name: Ident,
    ty: Type,
    default_expr: Option<String>,
}

#[proc_macro_attribute]
pub fn autoargs(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(item as ItemFn);
    
    let fn_name = &input_fn.sig.ident;
    let fn_args = parse_fn_args(&input_fn);
    let return_type = &input_fn.sig.output;
    
    // Convert snake_case to CamelCase for the struct name
    let fn_name_str = fn_name.to_string();
    let camel_case_name = fn_name_str.split('_')
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                None => String::new(),
                Some(c) => c.to_uppercase().chain(chars).collect(),
            }
        })
        .collect::<String>();
    
    let args_struct_name = format_ident!("{}Args", camel_case_name);
    
    // Generate the struct fields
    let struct_fields = fn_args.iter().map(|arg| {
        let name = &arg.name;
        let ty = &arg.ty;
        quote! { pub #name: #ty }
    });
    
    // Generate the Default implementation
    let default_fields = fn_args.iter().map(|arg| {
        let name = &arg.name;
        let default_expr = match &arg.default_expr {
            Some(expr) => tokenstream_from_str(expr).unwrap_or_else(|_| {
                let expr_lit = LitStr::new(expr, proc_macro2::Span::call_site());
                parse_quote! { #expr_lit.parse().unwrap() }
            }),
            None => parse_quote! { Default::default() },
        };
        quote! { #name: #default_expr }
    });
    
    // Generate the macro
    let macro_arms = generate_macro_arms(&fn_args, &args_struct_name, fn_name);
    
    // Modify the original function to take the args struct
    let fn_body = &input_fn.block;
    let arg_names: Vec<_> = fn_args.iter().map(|arg| &arg.name).collect();
    
    let expanded = quote! {
        pub struct #args_struct_name {
            #(#struct_fields),*
        }
        
        impl Default for #args_struct_name {
            fn default() -> Self {
                Self {
                    #(#default_fields),*
                }
            }
        }
        
        #[allow(unused_parens)]
        fn #fn_name(args: #args_struct_name) #return_type {
            let (#(#arg_names),*) = (#(args.#arg_names),*);
            #fn_body
        }
        
        #[macro_export]
        macro_rules! #fn_name {
            #(#macro_arms)*
        }
    };
    
    TokenStream::from(expanded)
}

fn parse_fn_args(input_fn: &ItemFn) -> Vec<ArgInfo> {
    input_fn
        .sig
        .inputs
        .iter()
        .filter_map(|arg| {
            if let FnArg::Typed(PatType { pat, ty, attrs, .. }) = arg {
                if let syn::Pat::Ident(pat_ident) = &**pat {
                    let name = pat_ident.ident.clone();
                    
                    // Extract default value from attributes
                    let default_expr = attrs.iter().find_map(|attr| {
                        if attr.path().is_ident("default") {
                            match &attr.meta {
                                Meta::NameValue(meta) => {
                                    // Handle Expr::Lit case for the value
                                    if let Expr::Lit(expr_lit) = &meta.value {
                                        if let Lit::Str(lit) = &expr_lit.lit {
                                            return Some(lit.value());
                                        }
                                    }
                                },
                                _ => {}
                            }
                        }
                        None
                    });
                    
                    return Some(ArgInfo {
                        name,
                        ty: (**ty).clone(),
                        default_expr,
                    });
                }
            }
            None
        })
        .collect()
}

fn generate_macro_arms(
    _args: &[ArgInfo],
    args_struct_name: &Ident,
    fn_name: &Ident,
) -> Vec<TokenStream2> {
    // Base case: empty macro call, all defaults
    let base_case = quote! {
        () => {
            #fn_name(#args_struct_name::default())
        };
    };
    
    // Case with named args - note the different pattern to avoid conflicts
    let named_args_case = quote! {
        ( $( $name:ident = $value:expr ),* $(,)? ) => {
            {
                let mut args = #args_struct_name::default();
                $(
                    args.$name = $value;
                )*
                #fn_name(args)
            }
        };
    };
    
    // Case for directly passing the struct
    let struct_case = quote! {
        ( $args:expr ) => {
            #fn_name($args)
        };
    };
    
    vec![base_case, named_args_case, struct_case]
}

// Helper function for TokenStream2 parsing
fn tokenstream_from_str(s: &str) -> Result<TokenStream2, proc_macro2::LexError> {
    s.parse()
}