extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{
    parse, parse_macro_input, spanned::Spanned, Ident, ItemFn, ReturnType, Type, Visibility,
};

#[derive(Debug, PartialEq)]
enum Exception {
    Reset,
    UndefinedInstruction,
    SoftwareInterrupt,
    PrefetchAbort,
    DataAbort,
    Interrupt,
}

#[proc_macro_attribute]
pub fn exception(args: TokenStream, input: TokenStream) -> TokenStream {
    let mut f = parse_macro_input!(input as ItemFn);

    if !args.is_empty() {
        return parse::Error::new(Span::call_site(), "This attribute accepts no arguments")
            .to_compile_error()
            .into();
    }

    let fspan = f.span();
    let ident = f.sig.ident.clone();

    let ident_s = ident.to_string();
    let exn = match &*ident_s {
        "Reset" => Exception::Reset,
        "SoftwareInterrupt" => Exception::SoftwareInterrupt,
        "UndefinedInstruction" => Exception::UndefinedInstruction,
        "PrefetchAbort" => Exception::PrefetchAbort,
        "DataAbort" => Exception::DataAbort,
        "SystemInterrupt" => Exception::Interrupt,
        _ => {
            return parse::Error::new(ident.span(), "This is not a valid exception name")
                .to_compile_error()
                .into();
        }
    };

    let mut valid_signature = f.sig.constness.is_none()
        && f.vis == Visibility::Inherited
        && f.sig.abi.is_none()
        && f.sig.inputs.is_empty()
        && f.sig.generics.params.is_empty()
        && f.sig.generics.where_clause.is_none()
        && f.sig.variadic.is_none();
    &&match f.sig.output {
        ReturnType::Default => true,
        ReturnType::Type(_, ref ty) => match **ty {
            Type::Tuple(ref tuple) => tuple.elems.is_empty(),
            Type::Never(..) => true,
            _ => false,
        },
    };

    if exn == Exception::SoftwareInterrupt {
        valid_signature = f.sig.constness.is_none()
            && f.vis == Visibility::Inherited
            //&& f.sig.abi.is_none()
            //&& f.sig.inputs.is_empty()
            && f.sig.generics.params.is_empty()
            && f.sig.generics.where_clause.is_none()
            && f.sig.variadic.is_none();
        // &&match f.sig.output {
        //     ReturnType::Default => true,
        //     ReturnType::Type(_, ref ty) => match **ty {
        //         Type::Tuple(ref tuple) => tuple.elems.is_empty(),
        //         Type::Never(..) => true,
        //         _ => false,
        //     },
        // };
    }

    if !valid_signature {
        return parse::Error::new(
            fspan,
            "`#[exception]` handlers must have signature `[unsafe] fn() [-> !]`",
        )
        .to_compile_error()
        .into();
    }

    f.sig.ident = Ident::new(&format!("__portux_{}", f.sig.ident), Span::call_site());

    let tramp_ident = Ident::new(&format!("{}_trampoline", f.sig.ident), Span::call_site());
    let ident = &f.sig.ident;

    let use_nested_interrupt;
    let dont_restore_registers;
    let lr_size;

    match exn {
        Exception::Interrupt => {
            use_nested_interrupt = true;
            dont_restore_registers = false;
            lr_size = 4;
        }
        Exception::SoftwareInterrupt => {
            use_nested_interrupt = false;
            dont_restore_registers = true;
            lr_size = 0;
        }
        Exception::UndefinedInstruction | Exception::Reset => {
            use_nested_interrupt = false;
            dont_restore_registers = false;
            lr_size = 0;
        }
        Exception::PrefetchAbort | Exception::DataAbort => {
            use_nested_interrupt = false;
            dont_restore_registers = true;
            lr_size = 4;
        }
    }

    quote!(
        #[naked]
        #[no_mangle]
        #[doc(hidden)]
        #[export_name = #ident_s]
        pub unsafe extern "C" fn #tramp_ident() {
            processor::exception_routine!(subroutine=#ident, lr_size=#lr_size, nested_interrupt=#use_nested_interrupt, dont_restore_registers=#dont_restore_registers);
        }

        #f
    )
    .into()
}

#[proc_macro_attribute]
pub fn interrupt(args: TokenStream, input: TokenStream) -> TokenStream {
    let f = parse_macro_input!(input as ItemFn);

    if !args.is_empty() {
        return parse::Error::new(Span::call_site(), "This attribute accepts no arguments")
            .to_compile_error()
            .into();
    }

    let fspan = f.span();

    let valid_signature = f.sig.constness.is_none()
        && f.vis == Visibility::Inherited
        && f.sig.abi.is_none()
        && f.sig.inputs.is_empty()
        && f.sig.generics.params.is_empty()
        && f.sig.generics.where_clause.is_none()
        && f.sig.variadic.is_none();
    &&match f.sig.output {
        ReturnType::Default => true,
        ReturnType::Type(_, ref ty) => match **ty {
            Type::Tuple(ref tuple) => tuple.elems.is_empty(),
            Type::Never(..) => true,
            _ => false,
        },
    };

    if !valid_signature {
        return parse::Error::new(
            fspan,
            "`#[interrupt]` handlers must have signature `[unsafe] fn() [-> !]`",
        )
        .to_compile_error()
        .into();
    }

    let tramp_ident = Ident::new(&format!("{}_trampoline", f.sig.ident), Span::call_site());
    let ident = &f.sig.ident;

    let use_nested_interrupt = true;
    let dont_restore_registers = false;
    let lr_size = 4;

    quote!(
        #[naked]
        #[doc(hidden)]
        pub unsafe extern "C" fn #tramp_ident() {
            processor::exception_routine!(subroutine=#ident, lr_size=#lr_size, nested_interrupt=#use_nested_interrupt, dont_restore_registers=#dont_restore_registers);
        }

        #f
    )
    .into()
}
