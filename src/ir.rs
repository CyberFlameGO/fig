use std::io::Write;

/// A value.
type Value = i64;

/// Reference to a value created by an instruction.
type ValueRef = usize;

/// Instructions of the IR to be compiled into native code.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum Instruction {
    /// Introduce a new value to the code to be used by other instructions.
    Load { storage: ValueRef, value: Value },
    /// Add two values.
    Add { left: ValueRef, right: ValueRef },
    /// Subtract two values.
    Subtract { left: ValueRef, right: ValueRef },
    /// Multiply two values.
    Multiply { left: ValueRef, right: ValueRef },
    /// Divide two values.
    Divide { left: ValueRef, right: ValueRef },
    /// Exit the process with the given exit code.
    Exit { exit_code: ValueRef },
}

/// Enumeration of general-purpose registers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Register {
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
    pub fn alloc(&mut self) -> ValueRef {
        let reg = self.free_regs.pop().expect("no free register");
        let id = self.used_regs.len();
        self.used_regs.push(reg);
        id
    }

    /// Get an allocated register from its id.
    pub fn get(&self, id: ValueRef) -> Register {
        self.used_regs[id]
    }
}

/// A block is a set of named set of instructions.
#[derive(Debug)]
pub struct Block {
    /// The name will be used as a label in the resulting native code.
    name: String,
    /// List of instructions belonging to this block.
    instructions: Vec<Instruction>,
    /// Register allocator for code generation.
    registers: RegisterAlloc,
}

impl Block {
    /// Create a new empty block with the given name.
    pub fn new(name: String) -> Self {
        Self {
            name,
            instructions: vec![],
            registers: RegisterAlloc::new(),
        }
    }

    /// Generate the native code for this block and write it to the given Writer.
    pub fn generate_code(&self, w: &mut impl Write) -> std::io::Result<()> {
        use Instruction::*;

        writeln!(w, "{}", self.name)?;
        for instruction in &self.instructions {
            match instruction {
                Load { storage, value } => {
                    let storage_reg = self.registers.get(*storage);
                    writeln!(w, "\tmov {}, {}", storage_reg.name(), value)?;
                }
                Add { left, right } => {
                    let left_reg = self.registers.get(*left);
                    let right_reg = self.registers.get(*right);
                    writeln!(w, "\tadd {}, {}", left_reg.name(), right_reg.name())?;
                }
                Subtract { left, right } => {
                    let left_reg = self.registers.get(*left);
                    let right_reg = self.registers.get(*right);
                    writeln!(w, "\tsub {}, {}", left_reg.name(), right_reg.name())?;
                }
                Multiply { left, right } => {
                    let left_reg = self.registers.get(*left);
                    let right_reg = self.registers.get(*right);
                    writeln!(w, "\timul {}, {}", left_reg.name(), right_reg.name())?;
                }
                Divide { left, right } => {
                    let left_reg = self.registers.get(*left);
                    let right_reg = self.registers.get(*right);
                    writeln!(w, "\tpush rdx")?;
                    writeln!(w, "\tmov rdx, 0")?;
                    if left_reg != Register::Rax {
                        writeln!(w, "\tpush rax")?;
                        writeln!(w, "\tmov rax, {}", left_reg.name())?;
                    }
                    writeln!(w, "\tidiv {}", right_reg.name())?;
                    if left_reg != Register::Rax {
                        writeln!(w, "\tmov {}, rax", left_reg.name())?;
                        writeln!(w, "\tpop rax ")?;
                    }
                    writeln!(w, "\tpop rdx")?;
                }
                Exit { exit_code } => {
                    let exit_code_reg = self.registers.get(*exit_code);
                    // We can savely overwrite RAX here because the process is about to be
                    // terminated anyway.
                    writeln!(w, "\tmov rax, 60")?;
                    // If the exit code is not already stored in RDI move it there.
                    if exit_code_reg != Register::Rdi {
                        writeln!(w, "\tmov rdi, {}", exit_code_reg.name())?;
                    }
                    writeln!(w, "\tsyscall")?;
                }
            }
        }

        Ok(())
    }

    /// Append a `Load` instruction to the end of this block.
    /// Returns a reference to the value to be used in other instructions.
    pub fn build_load(&mut self, value: Value) -> ValueRef {
        let storage = self.registers.alloc();
        self.instructions.push(Instruction::Load { storage, value });
        storage
    }

    /// Append a `Add` instruction to the end of this block.
    /// Returns a reference to the result to be used in other instructions.
    pub fn build_add(&mut self, left: ValueRef, right: ValueRef) -> ValueRef {
        self.instructions.push(Instruction::Add { left, right });
        left
    }

    /// Append a `Subtract` instruction to the end of this block.
    /// Returns a reference to the result to be used in other instructions.
    pub fn build_subtract(&mut self, left: ValueRef, right: ValueRef) -> ValueRef {
        self.instructions
            .push(Instruction::Subtract { left, right });
        left
    }

    /// Append a `Multiply` instruction to the end of this block.
    /// Returns a reference to the result to be used in other instructions.
    pub fn build_multiply(&mut self, left: ValueRef, right: ValueRef) -> ValueRef {
        self.instructions
            .push(Instruction::Multiply { left, right });
        left
    }

    /// Append a `Divide` instruction to the end of this block.
    /// Returns a reference to the result to be used in other instructions.
    pub fn build_divide(&mut self, left: ValueRef, right: ValueRef) -> ValueRef {
        self.instructions.push(Instruction::Divide { left, right });
        left
    }

    /// Append an `Exit` instruction to the end of this block.
    pub fn build_exit(&mut self, exit_code: ValueRef) {
        self.instructions.push(Instruction::Exit { exit_code });
    }
}
