use std::{borrow::Cow, io::Write};

/// A value.
type Value = i64;

/// Reference to a value created by an instruction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueRef {
    Register(Register),
    Memory(usize),
}

impl ValueRef {
    pub fn code(self) -> Cow<'static, str> {
        use ValueRef::*;
        match self {
            Register(reg) => Cow::Borrowed(reg.name()),
            Memory(off) => Cow::Owned(format!("[rbp-{}]", off)),
        }
    }
}

/// Instructions of the IR to be compiled into native code.
#[derive(Debug)]
enum Instruction {
    /// Introduce a new value to the code to be used by other instructions.
    Constant { storage: ValueRef, value: Value },
    /// Allocate memory on the stack.
    Alloc { size: usize },
    /// Store a value in memory.
    Store { value: ValueRef, storage: ValueRef },
    /// Add two values.
    Add { left: ValueRef, right: ValueRef },
    /// Subtract two values.
    Subtract { left: ValueRef, right: ValueRef },
    /// Multiply two values.
    Multiply { left: ValueRef, right: ValueRef },
    /// Divide two values.
    Divide { left: ValueRef, right: ValueRef },
    /// Jump to the given block.
    Jump { dest: String },
    /// Jump to the given block if the value is 0.
    JumpIfZero { value: ValueRef, dest: String },
    /// Call a function by its name with a single argument.
    Call { func: String, arg: Option<ValueRef> },
    /// Exit the process with the given exit code.
    Exit { exit_code: ValueRef },
}

/// Enumeration of general-purpose registers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Register {
    Rax,
    Rbx,
    Rcx,
    Rdx,
    Rsi,
    Rdi,
    R8,
    R9,
    R10,
    R11,
    R12,
    R13,
    R14,
    R15,
}

impl Register {
    fn name(self) -> &'static str {
        use Register::*;
        match self {
            Rax => "rax",
            Rbx => "rbx",
            Rcx => "rcx",
            Rdx => "rdx",
            Rsi => "rsi",
            Rdi => "rdi",
            R8 => "r8",
            R9 => "r9",
            R10 => "r10",
            R11 => "r11",
            R12 => "r12",
            R13 => "r13",
            R14 => "r14",
            R15 => "r15",
        }
    }
}

/// Register allocator for code generation.
#[derive(Debug)]
struct RegisterAlloc {
    /// List of free registers left for this block.
    free_regs: Vec<Register>,
    /// List of used registers in this block.
    used_regs: Vec<Register>,
}

impl RegisterAlloc {
    /// Create a new, clean register allocator.
    pub fn new() -> Self {
        use Register::*;
        Self {
            free_regs: vec![
                Rdi, Rsi, Rdx, Rcx, Rbx, Rax, R8, R9, R10, R11, R12, R13, R14, R15,
            ],
            used_regs: vec![],
        }
    }

    /// Allocate a new register and return its identifier.
    pub fn alloc(&mut self) -> Register {
        self.free_regs.pop().expect("no free register")
    }

    /// Free an allocated register so it can be allocated for something else again later.
    pub fn free(&mut self, reg: Register) {
        self.free_regs.push(reg);
    }
}

/// Stack memory allocator for code generation.
#[derive(Debug, Default)]
struct StackAlloc {
    /// The current size of the stack allocated memory.
    current_size: usize,
}

impl StackAlloc {
    /// Allocate memory on the stack with the given size.
    pub fn alloc(&mut self, size: usize) -> usize {
        self.current_size += size;
        self.current_size
    }
}

/// A module is a collection of functions.
#[derive(Debug, Default)]
pub struct Module<'a> {
    funcs: Vec<&'a Function<'a>>,
}

impl<'a> Module<'a> {
    pub fn append_func(&mut self, func: &'a Function) {
        self.funcs.push(func);
    }

    pub fn generate_code(&self, w: &mut impl Write) -> std::io::Result<()> {
        writeln!(w, "segment .text")?;
        writeln!(w, "extern put_int")?;
        for func in &self.funcs {
            func.generate_code(w)?;
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct Function<'a> {
    /// The name of the function which will be used as a label in native code.
    name: String,
    /// The blocks that belong to this function.
    blocks: Vec<&'a Block>,
}

impl<'a> Function<'a> {
    /// Create a new function with the given name.
    pub fn new(name: String) -> Self {
        Self {
            name,
            blocks: vec![],
        }
    }

    /// Append a block to this function.
    pub fn append_block(&mut self, block: &'a Block) {
        self.blocks.push(block);
    }

    /// Generate native code for this function.
    pub fn generate_code(&self, w: &mut impl Write) -> std::io::Result<()> {
        writeln!(w, "global {}", self.name)?;
        writeln!(w, "{}:", self.name)?;
        writeln!(w, "\tpush rbp")?;
        writeln!(w, "\tmov rbp, rsp")?;
        for block in &self.blocks {
            block.generate_code(w)?;
        }
        writeln!(w, "\tleave")?;
        Ok(())
    }
}

/// A block is a set of named set of instructions.
#[derive(Debug)]
pub struct Block {
    /// The name will be used as a label in the resulting native code.
    pub name: String,
    /// List of instructions belonging to this block.
    instructions: Vec<Instruction>,
    /// Register allocator for code generation.
    registers: RegisterAlloc,
    /// Stack memory allocator for code generation.
    stack: StackAlloc,
}

impl Block {
    /// Create a new empty block with the given name.
    pub fn new(name: String) -> Self {
        Self {
            name,
            instructions: vec![],
            registers: RegisterAlloc::new(),
            stack: StackAlloc::default(),
        }
    }

    /// Generate the native code for this block and write it to the given Writer.
    fn generate_code(&self, w: &mut impl Write) -> std::io::Result<()> {
        use Instruction::*;

        writeln!(w, "{}:", self.name)?;
        for instruction in &self.instructions {
            match *instruction {
                Constant { storage, value } => {
                    writeln!(w, "\tmov {}, {}", storage.code(), value)?;
                }
                Alloc { size } => {
                    writeln!(w, "\tsub rsp, {}", size)?;
                }
                Store { value, storage } => {
                    writeln!(w, "\tmov {}, {}", storage.code(), value.code())?;
                }
                Add { left, right } => {
                    writeln!(w, "\tadd {}, {}", left.code(), right.code())?;
                }
                Subtract { left, right } => {
                    writeln!(w, "\tsub {}, {}", left.code(), right.code())?;
                }
                Multiply { left, right } => {
                    writeln!(w, "\timul {}, {}", left.code(), right.code())?;
                }
                Divide { left, right } => {
                    writeln!(w, "\tpush rdx")?;
                    writeln!(w, "\tmov rdx, 0")?;
                    if left != ValueRef::Register(Register::Rax) {
                        writeln!(w, "\tpush rax")?;
                        writeln!(w, "\tmov rax, {}", left.code())?;
                    }
                    writeln!(w, "\tidiv {}", right.code())?;
                    if left != ValueRef::Register(Register::Rax) {
                        writeln!(w, "\tmov {}, rax", left.code())?;
                        writeln!(w, "\tpop rax ")?;
                    }
                    writeln!(w, "\tpop rdx")?;
                }
                Jump { ref dest } => {
                    writeln!(w, "\tjmp {}", dest)?;
                }
                JumpIfZero { value, ref dest } => {
                    writeln!(w, "\tcmp QWORD {}, 0", value.code())?;
                    writeln!(w, "\tje {}", dest)?;
                }
                Call { ref func, arg } => {
                    if let Some(arg) = arg {
                        if arg != ValueRef::Register(Register::Rdi) {
                            writeln!(w, "\tmov rdi, {}", arg.code())?;
                        }
                    }
                    writeln!(w, "\tcall {}", func)?;
                }
                Exit { exit_code } => {
                    // We can savely overwrite RAX here because the process is about to be
                    // terminated anyway.
                    writeln!(w, "\tmov rax, 60")?;
                    // If the exit code is not already stored in RDI move it there.
                    if exit_code != ValueRef::Register(Register::Rdi) {
                        writeln!(w, "\tmov rdi, {}", exit_code.code())?;
                    }
                    writeln!(w, "\tsyscall")?;
                }
            }
        }
        Ok(())
    }

    /// Append a `Constant` instruction to the end of this block.
    /// Returns a reference to the value to be used in other instructions.
    pub fn build_constant(&mut self, value: Value) -> ValueRef {
        let storage = ValueRef::Register(self.registers.alloc());
        self.instructions
            .push(Instruction::Constant { storage, value });
        storage
    }

    /// Append an `Alloc` instruction to the end of this block.
    /// Returns a reference to the memory allocated to be used in other instructions.
    pub fn build_alloc(&mut self, size: usize) -> ValueRef {
        self.instructions.push(Instruction::Alloc { size });
        ValueRef::Memory(self.stack.alloc(size))
    }

    /// Append a `Store` instruction to the end of this block.
    pub fn build_store(&mut self, value: ValueRef, storage: ValueRef) {
        self.instructions
            .push(Instruction::Store { value, storage });
        if let ValueRef::Register(reg) = value {
            self.registers.free(reg);
        }
    }

    /// Append a `Add` instruction to the end of this block.
    /// Returns a reference to the result to be used in other instructions.
    pub fn build_add(&mut self, left: ValueRef, right: ValueRef) -> ValueRef {
        self.instructions.push(Instruction::Add { left, right });
        if let ValueRef::Register(reg) = right {
            self.registers.free(reg);
        }
        left
    }

    /// Append a `Subtract` instruction to the end of this block.
    /// Returns a reference to the result to be used in other instructions.
    pub fn build_subtract(&mut self, left: ValueRef, right: ValueRef) -> ValueRef {
        self.instructions
            .push(Instruction::Subtract { left, right });
        if let ValueRef::Register(reg) = right {
            self.registers.free(reg);
        }
        left
    }

    /// Append a `Multiply` instruction to the end of this block.
    /// Returns a reference to the result to be used in other instructions.
    pub fn build_multiply(&mut self, left: ValueRef, right: ValueRef) -> ValueRef {
        self.instructions
            .push(Instruction::Multiply { left, right });
        if let ValueRef::Register(reg) = right {
            self.registers.free(reg);
        }
        left
    }

    /// Append a `Divide` instruction to the end of this block.
    /// Returns a reference to the result to be used in other instructions.
    pub fn build_divide(&mut self, left: ValueRef, right: ValueRef) -> ValueRef {
        self.instructions.push(Instruction::Divide { left, right });
        if let ValueRef::Register(reg) = right {
            self.registers.free(reg);
        }
        left
    }

    /// Append a `Jump` instruction to the end of this block.
    pub fn build_jump(&mut self, dest: String) {
        self.instructions.push(Instruction::Jump { dest });
    }

    /// Append a `JumpIfZero` instruction to the end of this block.
    pub fn build_jump_if_zero(&mut self, value: ValueRef, dest: String) {
        self.instructions
            .push(Instruction::JumpIfZero { value, dest });
        if let ValueRef::Register(reg) = value {
            self.registers.free(reg);
        }
    }

    /// Append a `Call` instruction to the end of this block.
    pub fn build_call(&mut self, func: String, arg: Option<ValueRef>) {
        self.instructions.push(Instruction::Call { func, arg });
    }

    /// Append an `Exit` instruction to the end of this block.
    pub fn build_exit(&mut self, exit_code: ValueRef) {
        self.instructions.push(Instruction::Exit { exit_code });
    }
}
