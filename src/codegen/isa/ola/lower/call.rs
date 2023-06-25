use std::vec;

use super::{
    get_operand_for_val, get_operands_for_val, get_vreg_for_val, new_empty_inst_output,
    new_empty_str_inst_output,
};
use crate::codegen::core::ir::{
    function::instruction::InstructionId,
    module::name::Name,
    types::{CompoundType, FunctionType, Type},
    value::{ConstantValue, Value, ValueId},
};
use crate::codegen::{
    function::instruction::Instruction as MachInstruction,
    isa::ola::{
        instruction::{InstructionData, Opcode, Operand as MO, OperandData},
        register::{RegClass, RegInfo, GR},
        Ola,
    },
    isa::TargetIsa,
    lower::{LoweringContext, LoweringError},
    register::{Reg, RegisterClass, RegisterInfo},
};
use anyhow::{Ok, Result};

pub fn lower_call(
    ctx: &mut LoweringContext<Ola>,
    id: InstructionId,
    tys: &[Type],
    args: &[ValueId],
) -> Result<()> {
    let callee = args[0];
    let name = match &ctx.ir_data.values[callee] {
        Value::Constant(ConstantValue::GlobalRef(Name::Name(name), _)) => name.clone(),
        callee => {
            return Err(LoweringError::Todo(format!("Unsupported callee: {:?}", callee)).into())
        }
    };
    log::debug!("call name: {}", name);

    // frist lower builtin call into isa inst directly
    if name.starts_with("builtin") {
        match name.as_str() {
            "builtin_assert" => {
                let lhs = get_operand_for_val(ctx, tys[1], args[1])?;
                let rhs = get_operand_for_val(ctx, tys[2], args[2])?;
                ctx.inst_seq.push(MachInstruction::new(
                    InstructionData {
                        opcode: Opcode::ASSERTri,
                        operands: vec![MO::input(lhs), MO::input(rhs)],
                    },
                    ctx.block_map[&ctx.cur_block],
                ));
            }
            "builtin_range_check" => {
                let arg = get_operand_for_val(ctx, tys[1], args[1])?;
                ctx.inst_seq.push(MachInstruction::new(
                    InstructionData {
                        opcode: Opcode::RANGECHECK,
                        operands: vec![MO::input(arg)],
                    },
                    ctx.block_map[&ctx.cur_block],
                ));
            }
            e => todo!("{:?}", e),
        }
        return Ok(());
    }

    if name.as_str() == "set_storage" {
        pass_str_args_to_regs(ctx, &tys[1..], &args[1..])?;
        ctx.inst_seq.push(MachInstruction::new(
            InstructionData {
                opcode: Opcode::SSTORE,
                operands: vec![],
            },
            ctx.block_map[&ctx.cur_block],
        ));
        return Ok(());
    }

    if name.as_str() == "get_storage" || name.as_str() == "poseidon_hash" {
        pass_str_args_to_regs(ctx, &tys[1..], &args[1..])?;

        let ret_reg0: Reg = GR::R1.into();
        let ret_reg1: Reg = GR::R2.into();
        let ret_reg2: Reg = GR::R3.into();
        let ret_reg3: Reg = GR::R4.into();

        let opcode = if name.as_str() == "get_storage" {
            Opcode::SLOAD
        } else {
            Opcode::POSEIDON
        };
        ctx.inst_seq.push(MachInstruction::new(
            InstructionData {
                opcode,
                operands: vec![
                    MO::implicit_output(ret_reg0.into()),
                    MO::implicit_output(ret_reg1.into()),
                    MO::implicit_output(ret_reg2.into()),
                    MO::implicit_output(ret_reg3.into()),
                ],
            },
            ctx.block_map[&ctx.cur_block],
        ));

        let ret_ty = ctx.types.base().element(tys[0]).unwrap();
        let res_reg: [Reg; 4] = [ret_reg0, ret_reg1, ret_reg2, ret_reg3];
        let mut output = vec![];
        let opcode = Opcode::MOVrr;
        if !ctx.ir_data.users_of(id).is_empty() {
            for _ in 0..4 {
                output = new_empty_str_inst_output(ctx, ret_ty, id);
            }
            for idx in 0..4 {
                ctx.inst_seq.push(MachInstruction::new(
                    InstructionData {
                        opcode,
                        operands: vec![
                            MO::output(output[idx].into()),
                            MO::input(res_reg[idx].into()),
                        ],
                    },
                    ctx.block_map[&ctx.cur_block],
                ));
            }
        }
        return Ok(());
    }

    // then lower general function calls and prophet separately:
    // for gfc insts pattern as args + call_inst + return
    // for prophet insts pattern as args + prophet_pseudo + return_prophet
    let mut opcode = Opcode::CALL;
    if name.starts_with("prophet") {
        opcode = Opcode::PROPHET;
    }

    let result_ty = if let Some(ty) = ctx.types.get(tys[0])
                    && let CompoundType::Function(FunctionType { ret, .. }) = &*ty {
        *ret
    } else {
        tys[0]
    };
    let output = new_empty_inst_output(ctx, result_ty, id);

    pass_args_to_regs(ctx, &tys[1..], &args[1..])?;

    let result_reg: Reg = GR::R0.into();

    ctx.inst_seq.push(MachInstruction::new(
        InstructionData {
            opcode,
            operands: vec![
                MO::implicit_output(result_reg.into()),
                MO::new(OperandData::Label(name)),
            ],
        },
        ctx.block_map[&ctx.cur_block],
    ));

    if !ctx.ir_data.users_of(id).is_empty() {
        let opcode = Opcode::MOVrr;
        ctx.inst_seq.push(MachInstruction::new(
            InstructionData {
                opcode,
                operands: vec![MO::output(output[0].into()), MO::input(result_reg.into())],
            },
            ctx.block_map[&ctx.cur_block],
        ));
    }

    Ok(())
}

fn pass_args_to_regs(ctx: &mut LoweringContext<Ola>, tys: &[Type], args: &[ValueId]) -> Result<()> {
    let gpru = RegInfo::arg_reg_list(&ctx.call_conv);

    for (gpr_used, (&ty, &arg0)) in tys.iter().zip(args.iter()).enumerate() {
        println!(
            "type pointer {:?},{:?},arg {:?}",
            ty.is_pointer(ctx.types),
            ty,
            arg0
        );
        let arg = get_operand_for_val(ctx, ty, arg0)?;
        println!("1 type {:?},arg {:?}", ty, arg0);
        let out = gpru[gpr_used].apply(&RegClass::for_type(ctx.types, ty));

        // TODO: pointer with ref passing
        /* if ty.is_pointer(ctx.types) {
            match &ctx.ir_data.values[arg0] {
                Value::Instruction(addr_id) => {
                    let opcode = ctx.ir_data.instructions[*addr_id].opcode;
                    if opcode == IrOpcode::GetElementPtr {
                        // ctx.mark_as_merged(*addr_id);
                    }
                }
                _ => return Err(LoweringError::Todo("Unsupported load pattern 1".into()).into()),
            }

            let fp: Reg = GR::R9.into();
            ctx.inst_seq.push(MachInstruction::new(
                InstructionData {
                    opcode: Opcode::ADDri,
                    operands: vec![
                        MO::output(out.into()),
                        MO::input(fp.into()),
                        MO::input((-3).into()),
                    ],
                },
                ctx.block_map[&ctx.cur_block],
            ));
            continue;
        } */

        let opcode = match &arg {
            OperandData::Int32(_) => Opcode::MOVri,
            OperandData::Reg(_) => Opcode::MOVrr, // TODO: FIXME
            OperandData::VReg(vreg) => {
                let ty = ctx.mach_data.vregs.type_for(*vreg);
                let sz = ctx.isa.data_layout().get_size_of(ctx.types, ty);
                match sz {
                    4 | 8 => Opcode::MOVrr,
                    e => {
                        return Err(LoweringError::Todo(format!(
                            "Unsupported argument size: {:?}",
                            e
                        ))
                        .into())
                    }
                }
            }
            e => return Err(LoweringError::Todo(format!("Unsupported argument: {:?}", e)).into()),
        };
        ctx.inst_seq.push(MachInstruction::new(
            InstructionData {
                opcode,
                operands: vec![MO::output(out.into()), MO::input(arg)],
            },
            ctx.block_map[&ctx.cur_block],
        ));
    }

    Ok(())
}

fn pass_str_args_to_regs(
    ctx: &mut LoweringContext<Ola>,
    tys: &[Type],
    args: &[ValueId],
) -> Result<()> {
    let gpru = RegInfo::str_arg_reg_list(&ctx.call_conv);
    let cur_ty = ctx.types.base().element(tys[0]).unwrap();

    let mut arg_str = get_operands_for_val(ctx, tys[0], args[0])?;
    // sstore: [key,value]
    // sload: [key]
    // poseidion: [params]
    if args.len() > 1 {
        let v = get_operands_for_val(ctx, tys[0], args[1])?;
        arg_str.extend(v);
    }

    for (gpr_used, arg) in arg_str.iter().enumerate() {
        let cur_ty = ctx.types.base().element(tys[0]).unwrap();
        let out = gpru[gpr_used].apply(&RegClass::for_type(ctx.types, cur_ty));

        let opcode = match &arg {
            OperandData::Int32(_) => Opcode::MOVri,
            OperandData::Reg(_) => Opcode::MOVrr,
            OperandData::VReg(_) => Opcode::MOVrr,
            e => {
                return Err(
                    LoweringError::Todo(format!("Unsupported storage argument: {:?}", e)).into(),
                )
            }
        };
        ctx.inst_seq.push(MachInstruction::new(
            InstructionData {
                opcode,
                operands: vec![MO::output(out.into()), MO::input(arg.clone())],
            },
            ctx.block_map[&ctx.cur_block],
        ));
    }

    Ok(())
}

pub fn lower_return(ctx: &mut LoweringContext<Ola>, arg: Option<(Type, ValueId)>) -> Result<()> {
    if let Some((ty, value)) = arg {
        let vreg = get_vreg_for_val(ctx, ty, value)?;
        let sz = ctx.isa.data_layout().get_size_of(ctx.types, ty);
        assert!(ty.is_integer() || ty.is_pointer(ctx.types));
        let (reg, opcode) = match sz {
            4 | 8 => (GR::R0.into(), Opcode::MOVrr),
            _ => todo!(),
        };
        ctx.inst_seq.push(MachInstruction::new(
            InstructionData {
                opcode,
                operands: vec![MO::output(OperandData::Reg(reg)), MO::input(vreg.into())],
            },
            ctx.block_map[&ctx.cur_block],
        ));
    }
    ctx.inst_seq.push(MachInstruction::new(
        InstructionData {
            opcode: Opcode::RET,
            operands: vec![],
        },
        ctx.block_map[&ctx.cur_block],
    ));
    Ok(())
}
