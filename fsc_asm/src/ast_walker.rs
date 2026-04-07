use crate::ast;
use fsc_assembler::Assembler;
use fsc_assembler::error::AssemblerResult;

pub struct AstWalker<'a> {
    core: &'a mut Assembler,
}

impl<'a> AstWalker<'a> {
    pub fn new(core: &'a mut Assembler) -> Self {
        Self { core }
    }

    pub fn walk(&mut self, program: &ast::Program) -> AssemblerResult<()> {
        for function in &program.functions {
            self.walk_function(function)?;
        }
        Ok(())
    }

    fn walk_function(&mut self, function: &ast::Function) -> AssemblerResult<()> {
        self.core
            .define_function(&function.name, function.private)?;

        for stmt in &function.body {
            self.walk_statement(stmt)?;
        }

        Ok(())
    }

    fn walk_statement(&mut self, stmt: &ast::Statement) -> AssemblerResult<()> {
        match stmt {
            ast::Statement::Label(name) => {
                self.core.define_label(name)?;
            }
            ast::Statement::Instruction(ins) => {
                self.walk_instruction(ins)?;
            }
        }
        Ok(())
    }

    fn walk_instruction(&mut self, ins: &ast::Instruction) -> AssemblerResult<()> {
        match ins {
            ast::Instruction::SysCall { argc, page, func } => {
                self.core.emit_syscall(*argc, *page, *func);
                Ok(())
            }
            ast::Instruction::GrowStack(n) => {
                self.core.emit_grow_stack(*n);
                Ok(())
            }
            ast::Instruction::ShrinkStack(n) => {
                self.core.emit_shrink_stack(*n);
                Ok(())
            }
            ast::Instruction::LoadArg(n) => {
                self.core.emit_load_arg(*n);
                Ok(())
            }
            ast::Instruction::StoreArg(n) => {
                self.core.emit_store_arg(*n);
                Ok(())
            }
            ast::Instruction::ArgAddi(n) => {
                self.core.emit_arg_addi(*n);
                Ok(())
            }
            ast::Instruction::ArgSubi(n) => {
                self.core.emit_arg_subi(*n);
                Ok(())
            }
            ast::Instruction::Push(n) => {
                self.core.emit_push(*n);
                Ok(())
            }
            ast::Instruction::PushImm(n) => {
                self.core.emit_push_imm(*n);
                Ok(())
            }
            ast::Instruction::PushResult => {
                self.core.emit_push_result();
                Ok(())
            }
            ast::Instruction::Add => {
                self.core.emit_add();
                Ok(())
            }
            ast::Instruction::Sub => {
                self.core.emit_sub();
                Ok(())
            }
            ast::Instruction::Mul => {
                self.core.emit_mul();
                Ok(())
            }
            ast::Instruction::Div => {
                self.core.emit_div();
                Ok(())
            }
            ast::Instruction::Mod => {
                self.core.emit_mod();
                Ok(())
            }
            ast::Instruction::And => {
                self.core.emit_and();
                Ok(())
            }
            ast::Instruction::Or => {
                self.core.emit_or();
                Ok(())
            }
            ast::Instruction::Xor => {
                self.core.emit_xor();
                Ok(())
            }
            ast::Instruction::Not => {
                self.core.emit_not();
                Ok(())
            }
            ast::Instruction::Neg => {
                self.core.emit_neg();
                Ok(())
            }
            ast::Instruction::Fadd => {
                self.core.emit_fadd();
                Ok(())
            }
            ast::Instruction::Fsub => {
                self.core.emit_fsub();
                Ok(())
            }
            ast::Instruction::Fmul => {
                self.core.emit_fmul();
                Ok(())
            }
            ast::Instruction::Fdiv => {
                self.core.emit_fdiv();
                Ok(())
            }
            ast::Instruction::Feq0 => {
                self.core.emit_feq0();
                Ok(())
            }
            ast::Instruction::Fneg => {
                self.core.emit_fneg();
                Ok(())
            }
            ast::Instruction::Feq => {
                self.core.emit_feq();
                Ok(())
            }
            ast::Instruction::Fneq => {
                self.core.emit_fneq();
                Ok(())
            }
            ast::Instruction::Flt => {
                self.core.emit_flt();
                Ok(())
            }
            ast::Instruction::Fgt => {
                self.core.emit_fgt();
                Ok(())
            }
            ast::Instruction::Fle => {
                self.core.emit_fle();
                Ok(())
            }
            ast::Instruction::Fge => {
                self.core.emit_fge();
                Ok(())
            }
            ast::Instruction::LStr(s) => self.core.emit_lstr(s),
            ast::Instruction::Delay(operand) => {
                self.core.emit_delay(*operand);
                Ok(())
            }
            ast::Instruction::DelayLoad => {
                self.core.emit_delay_load();
                Ok(())
            }
            ast::Instruction::DelayNeq0 => {
                self.core.emit_delay_neq0();
                Ok(())
            }
            ast::Instruction::Exit1 => {
                self.core.emit_exit_1();
                Ok(())
            }
            ast::Instruction::Exit2 => {
                self.core.emit_exit_2();
                Ok(())
            }
            ast::Instruction::SetArgMode => {
                self.core.emit_set_arg_mode();
                Ok(())
            }
            ast::Instruction::Call(sym) => self.core.emit_call(sym),
            ast::Instruction::Jmp(label) => self.core.emit_jmp(label),
            ast::Instruction::Jnz(label) => self.core.emit_jnz(label),
            ast::Instruction::JnzPause(label) => self.core.emit_jnz_pause(label),
            ast::Instruction::JnzSet(label) => self.core.emit_jnz_set(label),
            ast::Instruction::JzSet(label) => self.core.emit_jz_set(label),
            ast::Instruction::JzPause(label) => self.core.emit_jz_pause(label),
            ast::Instruction::Jz(label) => self.core.emit_jz(label),
            ast::Instruction::Jeq(label) => self.core.emit_jeq(label),
            ast::Instruction::JeqImm { imm, label } => self.core.emit_jeq_imm(*imm, label),
            ast::Instruction::Eq0 => {
                self.core.emit_eq0();
                Ok(())
            }
            ast::Instruction::Eq => {
                self.core.emit_eq();
                Ok(())
            }
            ast::Instruction::Neq => {
                self.core.emit_neq();
                Ok(())
            }
            ast::Instruction::Lt => {
                self.core.emit_lt();
                Ok(())
            }
            ast::Instruction::Gt => {
                self.core.emit_gt();
                Ok(())
            }
            ast::Instruction::Le => {
                self.core.emit_le();
                Ok(())
            }
            ast::Instruction::Ge => {
                self.core.emit_ge();
                Ok(())
            }
            ast::Instruction::Sl => {
                self.core.emit_sl();
                Ok(())
            }
            ast::Instruction::Srm => {
                self.core.emit_srm();
                Ok(())
            }
            ast::Instruction::Sr => {
                self.core.emit_sr();
                Ok(())
            }
            ast::Instruction::Lea(symbol) => {
                self.core.emit_lea(symbol);
                Ok(())
            }
            ast::Instruction::Ret(n) => {
                self.core.emit_ret(*n);
                Ok(())
            }
            ast::Instruction::Retv(n) => {
                self.core.emit_retv(*n);
                Ok(())
            }
            ast::Instruction::Lb => {
                self.core.emit_lb();
                Ok(())
            }
            ast::Instruction::Ls => {
                self.core.emit_ls();
                Ok(())
            }
            ast::Instruction::Lw => {
                self.core.emit_lw();
                Ok(())
            }
            ast::Instruction::Lbi => {
                self.core.emit_lbi();
                Ok(())
            }
            ast::Instruction::Lsi => {
                self.core.emit_lsi();
                Ok(())
            }
            ast::Instruction::Lwi => {
                self.core.emit_lwi();
                Ok(())
            }
            ast::Instruction::Sb => {
                self.core.emit_sb();
                Ok(())
            }
            ast::Instruction::Ss => {
                self.core.emit_ss();
                Ok(())
            }
            ast::Instruction::Sw => {
                self.core.emit_sw();
                Ok(())
            }
            ast::Instruction::SbAdd => {
                self.core.emit_sbadd();
                Ok(())
            }
            ast::Instruction::SbiAdd => {
                self.core.emit_sbiadd();
                Ok(())
            }
            ast::Instruction::SbSub => {
                self.core.emit_sbsub();
                Ok(())
            }
            ast::Instruction::SbiSub => {
                self.core.emit_sbisub();
                Ok(())
            }
            ast::Instruction::SsAdd => {
                self.core.emit_ssadd();
                Ok(())
            }
            ast::Instruction::SsiAdd => {
                self.core.emit_ssiadd();
                Ok(())
            }
            ast::Instruction::SsSub => {
                self.core.emit_sssub();
                Ok(())
            }
            ast::Instruction::SsiSub => {
                self.core.emit_ssisub();
                Ok(())
            }

            ast::Instruction::SwAdd => {
                self.core.emit_swadd();
                Ok(())
            }
            ast::Instruction::SwiAdd => {
                self.core.emit_swiadd();
                Ok(())
            }
            ast::Instruction::SwSub => {
                self.core.emit_swsub();
                Ok(())
            }
            ast::Instruction::SwiSub => {
                self.core.emit_swisub();
                Ok(())
            }

            ast::Instruction::ItoF(operand) => {
                self.core.emit_itof(*operand);
                Ok(())
            }
            ast::Instruction::FtoI(operand) => {
                self.core.emit_ftoi(*operand);
                Ok(())
            }
        }
    }
}
