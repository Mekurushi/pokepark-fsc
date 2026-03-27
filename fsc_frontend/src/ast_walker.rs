use fsc_core::Assembler;
use fsc_core::error::AssemblerResult;
use crate::ast;

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
        self.core.define_function(&function.name, function.private)?;

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
            ast::Instruction::SysCall{argc, page, func} => Ok(self.core.emit_syscall(*argc,
                                                                                     *page, *func)),
            ast::Instruction::GrowStack(n) => Ok(self.core.emit_grow_stack(*n)),
            ast::Instruction::LoadArg(n)   => Ok(self.core.emit_load_arg(*n)),
            ast::Instruction::StoreArg(n)   => Ok(self.core.emit_store_arg(*n)),
            ast::Instruction::Push(n)      => Ok(self.core.emit_push(*n)),
            ast::Instruction::PushImm(n)      => Ok(self.core.emit_push_imm(*n)),
            ast::Instruction::PushResult      => Ok(self.core.emit_push_result()),
            ast::Instruction::Add          => Ok(self.core.emit_add()),
            ast::Instruction::Sub          => Ok(self.core.emit_sub()),
            ast::Instruction::LStr(s)      => self.core.emit_lstr(s),
            ast::Instruction::Call(sym)    => self.core.emit_call(sym),
            ast::Instruction::Jmp(label)   => self.core.emit_jmp(label),
            ast::Instruction::Jz(label)   => self.core.emit_jz(label),
            ast::Instruction::Eq0   => Ok(self.core.emit_eq0()),
            ast::Instruction::Eq   => Ok(self.core.emit_eq()),
            ast::Instruction::Ret(n)       => Ok(self.core.emit_ret(*n)),
            ast::Instruction::Retv(n)      => Ok(self.core.emit_retv(*n)),
        }
    }
}