// SPDX-License-Identifier: Apache-2.0

use crate::sema::ast::{ArrayLength, Contract, Namespace, Type};
use std::path::Path;
use std::str;

use num_bigint::BigInt;
use num_traits::ToPrimitive;

use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::{ Module};
use inkwell::targets::{RelocMode};
use inkwell::types::{BasicMetadataTypeEnum, BasicType, BasicTypeEnum, FunctionType, IntType, StringRadix,
};
use inkwell::values::{BasicValueEnum, FunctionValue, GlobalValue, IntValue, PointerValue};
use inkwell::AddressSpace;
use crate::irgen::functions::gen_functions;


pub struct Binary<'a> {
    pub name: String,
    pub module: Module<'a>,
    pub builder: Builder<'a>,
    pub(crate) context: &'a Context,
}

impl<'a> Binary<'a> {
    /// Build the LLVM IR for a single contract
    pub fn build(
        context: &'a Context,
        contract: &'a Contract,
        ns: &'a Namespace,
        filename: &'a str
    ) -> Self {

        let mut binary = Binary::new(
            context,
            &contract.name,
            filename,
        );

        gen_functions(&mut binary, contract, ns);
        binary
    }

    pub fn dump_llvm(&self, path: &Path) -> Result<(), String> {
        if let Err(s) = self.module.print_to_file(path) {
            return Err(s.to_string());
        }

        Ok(())
    }

    pub fn new(
        context: &'a Context,
        name: &str,
        filename: &str,
    ) -> Self {
        let module = context.create_module(name);


        let builder = context.create_builder();

        module.set_source_file_name(filename);

        Binary {
            name: name.to_owned(),
            module,
            builder,
            context,
        }
    }

    /// Default empty value
    pub(crate) fn default_value(&self, ty: &Type, ns: &Namespace) -> BasicValueEnum<'a> {
        let llvm_ty = self.llvm_var_ty(ty, ns);

        // const_zero() on BasicTypeEnum yet. Should be coming to inkwell soon
        if llvm_ty.is_pointer_type() {
            llvm_ty.into_pointer_type().const_null().into()
        } else {
            llvm_ty.into_int_type().const_zero().into()
        }
    }

    /// Convert a BigInt number to llvm const value
    pub(crate) fn number_literal(&self, bits: u32, n: &BigInt, _ns: &Namespace) -> IntValue<'a> {
        let ty = self.context.custom_width_int_type(bits);
        let s = n.to_string();

        ty.const_int_from_string(&s, StringRadix::Decimal).unwrap()
    }

    /// Emit function prototype
    pub(crate) fn function_type(
        &self,
        params: &[Type],
        returns: &[Type],
        ns: &Namespace,
    ) -> FunctionType<'a> {
        // function parameters
        let mut args = params
            .iter()
            .map(|ty| self.llvm_var_ty(ty, ns).into())
            .collect::<Vec<BasicMetadataTypeEnum>>();

        // add return values
        for ty in returns {
            args.push(if ty.is_reference_type(ns) && !ty.is_contract_storage() {
                self.llvm_type(ty, ns)
                    .ptr_type(AddressSpace::Generic)
                    .ptr_type(AddressSpace::Generic)
                    .into()
            } else {
                self.llvm_type(ty, ns)
                    .ptr_type(AddressSpace::Generic)
                    .into()
            });
        }

        let i64_type = self.context.i64_type();
        i64_type.fn_type(&args, false)

    }



    /// Return the llvm type for a variable holding the type, not the type itself
    pub(crate) fn llvm_var_ty(&self, ty: &Type, ns: &Namespace) -> BasicTypeEnum<'a> {
        let llvm_ty = self.llvm_type(ty, ns);
        match ty.deref_memory() {
            Type::Struct(_) | Type::Array(..) => {
                llvm_ty.ptr_type(AddressSpace::Generic).as_basic_type_enum()
            }
            _ => llvm_ty,
        }
    }

    /// Return the llvm type for field in struct or array
    pub(crate) fn llvm_field_ty(&self, ty: &Type, ns: &Namespace) -> BasicTypeEnum<'a> {
        let llvm_ty = self.llvm_type(ty, ns);
        match ty.deref_memory() {
            Type::Array(_, dim) if dim.last() == Some(&ArrayLength::Dynamic) => {
                llvm_ty.ptr_type(AddressSpace::Generic).as_basic_type_enum()
            }
            _ => llvm_ty,
        }
    }

    /// Return the llvm type for the resolved type.
    pub(crate) fn llvm_type(&self, ty: &Type, ns: &Namespace) -> BasicTypeEnum<'a> {
        match ty {
            Type::Bool => BasicTypeEnum::IntType(self.context.bool_type()),

            Type::Uint(n) => {
                BasicTypeEnum::IntType(self.context.custom_width_int_type(*n as u32))
            }
            Type::Enum(n) => self.llvm_type(&ns.enums[*n].ty, ns),

            Type::Array(base_ty, dims) => {
                let ty = self.llvm_field_ty(base_ty, ns);

                let mut dims = dims.iter();

                let mut aty = match dims.next().unwrap() {
                    ArrayLength::Fixed(d) => ty.array_type(d.to_u32().unwrap()),
                    ArrayLength::Dynamic => {
                        return self.module.get_struct_type("struct.vector").unwrap().into()
                    }
                    ArrayLength::AnyFixed => {
                        unreachable!()
                    }
                };

                for dim in dims {
                    match dim {
                        ArrayLength::Fixed(d) => aty = aty.array_type(d.to_u32().unwrap()),
                        ArrayLength::Dynamic => {
                            return self.module.get_struct_type("struct.vector").unwrap().into()
                        }
                        ArrayLength::AnyFixed => {
                            unreachable!()
                        }
                    }
                }

                BasicTypeEnum::ArrayType(aty)
            }
            Type::Struct(n) => self
                .context
                .struct_type(
                    &ns.structs[*n]
                        .fields
                        .iter()
                        .map(|f| self.llvm_field_ty(&f.ty, ns))
                        .collect::<Vec<BasicTypeEnum>>(),
                    false,
                )
                .as_basic_type_enum(),
            Type::Function {
                params, returns, ..
            } => {
                let ftype = self.function_type(params, returns, ns);

                BasicTypeEnum::PointerType(ftype.ptr_type(AddressSpace::Generic))
            }
            Type::UserType(no) => self.llvm_type(&ns.user_types[*no].ty, ns),
            Type::BufferPointer => self
                .context
                .i8_type()
                .ptr_type(AddressSpace::Generic)
                .as_basic_type_enum(),
            _ => unreachable!(),
        }
    }

}



