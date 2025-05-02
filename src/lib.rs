use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, format_ident};
use syn::{
    parse_macro_input, FnArg, Ident, ItemFn, Lit, Meta, PatType, Type,
    parse_quote, LitStr, Expr, ImplItemFn, Visibility,
};

/// Custom attribute for default values
#[proc_macro_attribute]
pub fn default(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // This simply passes through the input but makes it so we can use #[default = "..."]
    item
}

struct ArgInfo {
    name: Ident,
    ty: Type,
    default_expr: Option<String>,
}

#[proc_macro_attribute]
pub fn autoargs(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // First, try to parse as a method within an impl block
    if let Ok(impl_method) = syn::parse::<ImplItemFn>(item.clone()) {
        let _args_struct = generate_args_struct(&impl_method.sig.ident, &impl_method.sig.inputs, &impl_method.vis);
        
        // Extract the self param
        let self_param = impl_method.sig.inputs.iter().find_map(|arg| {
            if let FnArg::Receiver(receiver) = arg {
                Some(receiver)
            } else {
                None
            }
        });
        
        if let Some(_self_param) = self_param {
            // For methods, pull out the args and generate a modified method
            let fn_args = parse_fn_args(&impl_method.sig.inputs);
            let method_name = &impl_method.sig.ident;
            let method_vis = &impl_method.vis;
            let return_type = &impl_method.sig.output;
            let method_body = &impl_method.block;
            
            // Convert snake_case to CamelCase for the struct name
            let method_name_str = method_name.to_string();
            let camel_case_name = to_camel_case(&method_name_str);
            let args_struct_name = format_ident!("{}Args", camel_case_name);
            
            let arg_names: Vec<_> = fn_args.iter().map(|arg| &arg.name).collect();
            
            // Generate the modified method implementation
            let modified_method = quote! {
                #[allow(unused_parens)]
                #method_vis fn #method_name(#self_param, args: #args_struct_name) #return_type {
                    let (#(#arg_names),*) = (#(args.#arg_names),*);
                    #method_body
                }
            };
            
            // Return just the modified method implementation
            // The macro and struct definitions need to be outside the impl block
            return TokenStream::from(modified_method);
        }
    }
    
    // Otherwise, parse as a normal function
    let input_fn = parse_macro_input!(item as ItemFn);
    
    let fn_name = &input_fn.sig.ident;
    let fn_args = parse_fn_args(&input_fn.sig.inputs);
    let return_type = &input_fn.sig.output;
    let fn_visibility = &input_fn.vis;
    
    // Convert snake_case to CamelCase for the struct name
    let fn_name_str = fn_name.to_string();
    let camel_case_name = to_camel_case(&fn_name_str);
    
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
    let macro_arms = generate_function_macro_arms(&args_struct_name, fn_name);
    
    // Modify the original function to take the args struct
    let fn_body = &input_fn.block;
    let arg_names: Vec<_> = fn_args.iter().map(|arg| &arg.name).collect();
    
    let expanded = quote! {
        #fn_visibility struct #args_struct_name {
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
        #fn_visibility fn #fn_name(args: #args_struct_name) #return_type {
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

/// Companion attribute macro for implementing autoargs on methods
#[proc_macro_attribute]
pub fn impl_autoargs(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as syn::ItemImpl);
    
    let _self_ty = &input.self_ty;  // not used currently but might be useful in future
    let mut method_macros = vec![];
    let mut method_args_structs = vec![];
    
    for item in &input.items {
        if let syn::ImplItem::Fn(method) = item {
            let method_name = &method.sig.ident;
            
            // Check if this method has an #[autoargs] attribute
            let has_autoargs = method.attrs.iter().any(|attr| {
                attr.path().segments.iter().any(|seg| seg.ident == "autoargs")
            });
            
            if has_autoargs {
                // Generate the method macro
                let method_vis = &method.vis;
                let camel_case_name = to_camel_case(&method_name.to_string());
                let args_struct_name = format_ident!("{}Args", camel_case_name);
                
                // Generate the args struct
                let args_struct = generate_args_struct(&method_name, &method.sig.inputs, method_vis);
                method_args_structs.push(args_struct);
                
                // Determine if the method takes &self, &mut self, or self
                let self_param = method.sig.inputs.iter().find_map(|arg| {
                    if let FnArg::Receiver(receiver) = arg {
                        Some(receiver)
                    } else {
                        None
                    }
                });
                
                if let Some(_self_param) = self_param {
                    // Generate appropriate macro rules based on receiver type
                    let macro_rules = generate_method_macro_arms(&args_struct_name, method_name);
                    
                    let method_macro = quote! {
                        #[macro_export]
                        macro_rules! #method_name {
                            #(#macro_rules)*
                        }
                    };
                    
                    method_macros.push(method_macro);
                }
            }
        }
    }
    
    let result = quote! {
        #(#method_args_structs)*
        
        #(#method_macros)*
        
        #input
    };
    
    TokenStream::from(result)
}

// Helper function to generate an args struct for a method
fn generate_args_struct(method_name: &Ident, inputs: &syn::punctuated::Punctuated<FnArg, syn::token::Comma>, visibility: &Visibility) -> TokenStream2 {
    let fn_args = parse_fn_args(inputs);
    
    // Convert snake_case to CamelCase for the struct name
    let method_name_str = method_name.to_string();
    let camel_case_name = to_camel_case(&method_name_str);
    
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
    
    quote! {
        #visibility struct #args_struct_name {
            #(#struct_fields),*
        }
        
        impl Default for #args_struct_name {
            fn default() -> Self {
                Self {
                    #(#default_fields),*
                }
            }
        }
    }
}

// Parse function arguments into our ArgInfo struct
fn parse_fn_args(inputs: &syn::punctuated::Punctuated<FnArg, syn::token::Comma>) -> Vec<ArgInfo> {
    inputs
        .iter()
        .filter_map(|arg| {
            match arg {
                // Skip self parameters
                FnArg::Receiver(_) => None,
                FnArg::Typed(PatType { pat, ty, attrs, .. }) => {
                    if let syn::Pat::Ident(pat_ident) = &**pat {
                        let name = pat_ident.ident.clone();
                        
                        // Skip "self" even if it's typed
                        if name.to_string() == "self" {
                            return None;
                        }
                        
                        // Extract default value from attributes
                        let default_expr = attrs.iter().find_map(|attr| {
                            if attr.path().segments.iter().any(|seg| seg.ident == "default") {
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
                    None
                }
            }
        })
        .collect()
}

// Generate macro arms for regular functions
fn generate_function_macro_arms(
    args_struct_name: &Ident,
    fn_name: &Ident,
) -> Vec<TokenStream2> {
    // Base case: empty macro call, all defaults
    let base_case = quote! {
        () => {
            #fn_name(#args_struct_name::default())
        };
    };
    
    // Case with named args
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

// Generate macro arms for methods
fn generate_method_macro_arms(
    args_struct_name: &Ident,
    method_name: &Ident,
) -> Vec<TokenStream2> {
    // Base case: empty macro call, all defaults
    let base_case = quote! {
        ($self:expr) => {
            $self.#method_name(#args_struct_name::default())
        };
    };
    
    // Case with named args
    let named_args_case = quote! {
        ($self:expr, $( $name:ident = $value:expr ),* $(,)? ) => {
            {
                let mut args = #args_struct_name::default();
                $(
                    args.$name = $value;
                )*
                $self.#method_name(args)
            }
        };
    };
    
    // Case for directly passing the struct
    let struct_case = quote! {
        ($self:expr, $args:expr) => {
            $self.#method_name($args)
        };
    };
    
    vec![base_case, named_args_case, struct_case]
}

// Convert snake_case to CamelCase
fn to_camel_case(input: &str) -> String {
    input.split('_')
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                None => String::new(),
                Some(c) => c.to_uppercase().chain(chars).collect(),
            }
        })
        .collect()
}

// Helper function for TokenStream2 parsing
fn tokenstream_from_str(s: &str) -> Result<TokenStream2, proc_macro2::LexError> {
    s.parse()
}