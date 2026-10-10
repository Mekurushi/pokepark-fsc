use std::collections::{HashMap, HashSet};

use fsc_vm_ir::{
    AddressingMode, BasicBlock, BranchKind, BranchMode, Comparison, ConversionKind, DataId,
    ExitKind, FloatOperation, FunctionRef, Instruction, IntegerOperation, Linkage, MemoryWidth,
    Program, ReturnKind, ShiftOperation, StoreOperation, Terminator,
};

use crate::error::{AssemblerError, AssemblerResult};
use crate::{Assembler, AssemblyUnit};

pub fn assemble_program(program: &Program) -> AssemblerResult<AssemblyUnit> {
    let data_names = assign_data_names(program);
    let mut assembler = Assembler::new();

    for (_id, function) in program.functions() {
        assembler.define_function(function.name(), function.linkage() == Linkage::Exported)?;

        for (block_id, block) in function.blocks() {
            assembler.define_label(&block_id.to_string())?;
            emit_block(
                &mut assembler,
                program,
                &data_names,
                function.name(),
                block_id.to_string(),
                block,
            )?;
        }

        assembler.end_function()?;
    }

    for (id, data) in program.data_objects() {
        let name = &data_names[&id];
        assembler.define_data(name)?;
        for word in data.words() {
            assembler.emit_data_word(word.bits())?;
        }
    }

    Ok(assembler.into_unit())
}

fn assign_data_names(program: &Program) -> HashMap<DataId, String> {
    //TODO: adapt Assembler to not use the string as identifier
    let mut used_names: HashSet<String> = program
        .functions()
        .map(|(_id, function)| function.name().to_owned())
        .chain(
            program
                .data_objects()
                .filter_map(|(_id, data)| data.name().map(str::to_owned)),
        )
        .collect();
    let mut names = HashMap::new();

    for (id, data) in program.data_objects() {
        let name = data.name().map_or_else(
            || {
                let mut candidate = format!("__fsc_{id}");
                let mut discriminator = 0_u32;
                while used_names.contains(&candidate) {
                    candidate = format!("__fsc_{id}_{discriminator}");
                    discriminator += 1;
                }
                candidate
            },
            str::to_owned,
        );
        used_names.insert(name.clone());
        names.insert(id, name);
    }

    names
}

fn emit_block(
    assembler: &mut Assembler,
    program: &Program,
    data_names: &HashMap<DataId, String>,
    function_name: &str,
    block_name: String,
    block: &BasicBlock,
) -> AssemblerResult<()> {
    for instruction in block.instructions() {
        emit_instruction(assembler, program, data_names, instruction)?;
    }

    let terminator = block
        .terminator()
        .ok_or_else(|| AssemblerError::MissingTerminator {
            function: function_name.to_owned(),
            block: block_name,
        })?;
    emit_terminator(assembler, terminator)
}

fn emit_instruction(
    assembler: &mut Assembler,
    program: &Program,
    data_names: &HashMap<DataId, String>,
    instruction: &Instruction,
) -> AssemblerResult<()> {
    match instruction {
        Instruction::Syscall {
            argument_count,
            page,
            function,
        } => assembler.emit_syscall(*argument_count, *page, *function),
        Instruction::Delay(value) => assembler.emit_delay(*value),
        Instruction::DelayFromStack => assembler.emit_delay_load(),
        Instruction::DelayIfNonZero => assembler.emit_delay_neq0(),
        Instruction::Call(target) => {
            let name = match target {
                FunctionRef::Internal(id) => program.function(*id).name(),
                FunctionRef::External(function) => function.name(),
            };
            assembler.emit_call(name)?;
        }
        Instruction::GrowStack(slots) => assembler.emit_grow_stack(slots.cast_signed()),
        Instruction::ShrinkStack(slots) => assembler.emit_shrink_stack(slots.cast_signed()),
        Instruction::LoadFrameSlot(offset) => assembler.emit_load_arg(*offset),
        Instruction::LoadFrameSlotAddress(offset) => {
            assembler.emit_load_arg_ref();
            assembler.emit_load_arg(*offset);
        }
        Instruction::StoreFrameSlot(offset) => assembler.emit_store_arg(*offset),
        Instruction::AddToFrameSlot(offset) => assembler.emit_arg_addi(*offset),
        Instruction::SubtractFromFrameSlot(offset) => assembler.emit_arg_subi(*offset),
        Instruction::LoadRel(offset) => assembler.emit_load_rel(*offset),
        Instruction::PushConstant(word) => {
            let value = word.bits().cast_signed();
            if let Ok(short) = i16::try_from(value) {
                assembler.emit_push(short);
            } else {
                assembler.emit_push_imm(word.bits());
            }
        }
        Instruction::PushResult => assembler.emit_push_result(),
        Instruction::LoadString(value) => assembler.emit_lstr(value)?,
        Instruction::Integer(operation) => emit_integer(assembler, *operation),
        Instruction::Float(operation) => emit_float(assembler, *operation),
        Instruction::IntegerCompare(comparison) => emit_integer_comparison(assembler, *comparison),
        Instruction::FloatCompare(comparison) => emit_float_comparison(assembler, *comparison),
        Instruction::Shift(operation) => emit_shift(assembler, *operation),
        Instruction::LoadAddress(id) => {
            let name = data_names
                .get(id)
                .ok_or_else(|| AssemblerError::UndefinedSymbol(id.to_string()))?;
            assembler.emit_lea(name);
        }
        Instruction::Load { width, addressing } => emit_load(assembler, *width, *addressing),
        Instruction::Store {
            width,
            addressing,
            operation,
        } => emit_store(assembler, *width, *addressing, *operation),
        Instruction::Convert { kind, stack_offset } => match kind {
            ConversionKind::IntegerToFloat => assembler.emit_itof(*stack_offset),
            ConversionKind::FloatToInteger => assembler.emit_ftoi(*stack_offset),
        },
        Instruction::Debug { subtype, operand } => assembler.emit_debug(*subtype, *operand),
    }

    Ok(())
}

fn emit_terminator(assembler: &mut Assembler, terminator: &Terminator) -> AssemblerResult<()> {
    match terminator {
        Terminator::Jump(target) => assembler.emit_jmp(&target.to_string())?,
        Terminator::Branch {
            kind,
            taken,
            not_taken,
        } => {
            let taken = taken.to_string();
            match kind {
                BranchKind::IfZero(BranchMode::Normal) => assembler.emit_jz(&taken)?,
                BranchKind::IfZero(BranchMode::Pause) => assembler.emit_jz_pause(&taken)?,
                BranchKind::IfZero(BranchMode::Set) => assembler.emit_jz_set(&taken)?,
                BranchKind::IfNonZero(BranchMode::Normal) => assembler.emit_jnz(&taken)?,
                BranchKind::IfNonZero(BranchMode::Pause) => assembler.emit_jnz_pause(&taken)?,
                BranchKind::IfNonZero(BranchMode::Set) => assembler.emit_jnz_set(&taken)?,
                BranchKind::IfEqual => assembler.emit_jeq(&taken)?,
                BranchKind::IfEqualImmediate(immediate) => {
                    assembler.emit_jeq_imm(*immediate, &taken)?;
                }
            }
            assembler.emit_jmp(&not_taken.to_string())?;
        }
        Terminator::Return { kind, frame_slots } => match kind {
            ReturnKind::None => assembler.emit_ret(frame_slots.cast_signed()),
            ReturnKind::Value => assembler.emit_retv(frame_slots.cast_signed()),
        },
        Terminator::Exit(ExitKind::Abort) => assembler.emit_exit_1(),
        Terminator::Exit(ExitKind::Reset) => assembler.emit_exit_2(),
    }

    Ok(())
}

fn emit_integer(assembler: &mut Assembler, operation: IntegerOperation) {
    match operation {
        IntegerOperation::Add => assembler.emit_add(),
        IntegerOperation::Subtract => assembler.emit_sub(),
        IntegerOperation::Multiply => assembler.emit_mul(),
        IntegerOperation::Divide => assembler.emit_div(),
        IntegerOperation::Remainder => assembler.emit_mod(),
        IntegerOperation::And => assembler.emit_and(),
        IntegerOperation::Or => assembler.emit_or(),
        IntegerOperation::Xor => assembler.emit_xor(),
        IntegerOperation::Not => assembler.emit_not(),
        IntegerOperation::IsZero => assembler.emit_eq0(),
        IntegerOperation::Negate => assembler.emit_neg(),
    }
}

fn emit_float(assembler: &mut Assembler, operation: FloatOperation) {
    match operation {
        FloatOperation::Add => assembler.emit_fadd(),
        FloatOperation::Subtract => assembler.emit_fsub(),
        FloatOperation::Multiply => assembler.emit_fmul(),
        FloatOperation::Divide => assembler.emit_fdiv(),
        FloatOperation::IsZero => assembler.emit_feq0(),
        FloatOperation::Negate => assembler.emit_fneg(),
    }
}

fn emit_integer_comparison(assembler: &mut Assembler, comparison: Comparison) {
    match comparison {
        Comparison::Equal => assembler.emit_eq(),
        Comparison::NotEqual => assembler.emit_neq(),
        Comparison::LessThan => assembler.emit_lt(),
        Comparison::GreaterThan => assembler.emit_gt(),
        Comparison::LessOrEqual => assembler.emit_le(),
        Comparison::GreaterOrEqual => assembler.emit_ge(),
    }
}

fn emit_float_comparison(assembler: &mut Assembler, comparison: Comparison) {
    match comparison {
        Comparison::Equal => assembler.emit_feq(),
        Comparison::NotEqual => assembler.emit_fneq(),
        Comparison::LessThan => assembler.emit_flt(),
        Comparison::GreaterThan => assembler.emit_fgt(),
        Comparison::LessOrEqual => assembler.emit_fle(),
        Comparison::GreaterOrEqual => assembler.emit_fge(),
    }
}

fn emit_shift(assembler: &mut Assembler, operation: ShiftOperation) {
    match operation {
        ShiftOperation::Left => assembler.emit_sl(),
        ShiftOperation::RightArithmetic => assembler.emit_srm(),
        ShiftOperation::RightLogical => assembler.emit_sr(),
    }
}

fn emit_load(assembler: &mut Assembler, width: MemoryWidth, addressing: AddressingMode) {
    match (width, addressing) {
        (MemoryWidth::Byte, AddressingMode::Direct) => assembler.emit_lb(),
        (MemoryWidth::Halfword, AddressingMode::Direct) => assembler.emit_ls(),
        (MemoryWidth::Word, AddressingMode::Direct) => assembler.emit_lw(),
        (MemoryWidth::Byte, AddressingMode::Indexed) => assembler.emit_lbi(),
        (MemoryWidth::Halfword, AddressingMode::Indexed) => assembler.emit_lsi(),
        (MemoryWidth::Word, AddressingMode::Indexed) => assembler.emit_lwi(),
    }
}

fn emit_store(
    assembler: &mut Assembler,
    width: MemoryWidth,
    addressing: AddressingMode,
    operation: StoreOperation,
) {
    match (width, addressing, operation) {
        (MemoryWidth::Byte, AddressingMode::Direct, StoreOperation::Assign) => assembler.emit_sb(),
        (MemoryWidth::Halfword, AddressingMode::Direct, StoreOperation::Assign) => {
            assembler.emit_ss();
        }
        (MemoryWidth::Word, AddressingMode::Direct, StoreOperation::Assign) => assembler.emit_sw(),
        (MemoryWidth::Byte, AddressingMode::Indexed, StoreOperation::Assign) => {
            assembler.emit_sbi();
        }
        (MemoryWidth::Halfword, AddressingMode::Indexed, StoreOperation::Assign) => {
            assembler.emit_ssi();
        }
        (MemoryWidth::Word, AddressingMode::Indexed, StoreOperation::Assign) => {
            assembler.emit_swi();
        }
        (MemoryWidth::Byte, AddressingMode::Direct, StoreOperation::Add) => assembler.emit_sbadd(),
        (MemoryWidth::Halfword, AddressingMode::Direct, StoreOperation::Add) => {
            assembler.emit_ssadd();
        }
        (MemoryWidth::Word, AddressingMode::Direct, StoreOperation::Add) => assembler.emit_swadd(),
        (MemoryWidth::Byte, AddressingMode::Indexed, StoreOperation::Add) => {
            assembler.emit_sbiadd();
        }
        (MemoryWidth::Halfword, AddressingMode::Indexed, StoreOperation::Add) => {
            assembler.emit_ssiadd();
        }
        (MemoryWidth::Word, AddressingMode::Indexed, StoreOperation::Add) => {
            assembler.emit_swiadd();
        }
        (MemoryWidth::Byte, AddressingMode::Direct, StoreOperation::Subtract) => {
            assembler.emit_sbsub();
        }
        (MemoryWidth::Halfword, AddressingMode::Direct, StoreOperation::Subtract) => {
            assembler.emit_sssub();
        }
        (MemoryWidth::Word, AddressingMode::Direct, StoreOperation::Subtract) => {
            assembler.emit_swsub();
        }
        (MemoryWidth::Byte, AddressingMode::Indexed, StoreOperation::Subtract) => {
            assembler.emit_sbisub();
        }
        (MemoryWidth::Halfword, AddressingMode::Indexed, StoreOperation::Subtract) => {
            assembler.emit_ssisub();
        }
        (MemoryWidth::Word, AddressingMode::Indexed, StoreOperation::Subtract) => {
            assembler.emit_swisub();
        }
    }
}
