use std::{
    fs::OpenOptions,
    process::{Command, Output},
};

use fig::ir::{Block, Module};

fn run_command(cmd: &str) -> std::io::Result<Output> {
    println!("+ {}", cmd);
    Command::new("sh").arg("-c").arg(cmd).output()
}

fn main() -> std::io::Result<()> {
    let mut if_zero = Block::new("if_zero".into());
    let ten = if_zero.build_load(10);
    if_zero.build_exit(ten);

    let mut start = Block::new("_start".into());
    let value = start.build_load(0);
    start.build_jump_if_zero(value, &if_zero);

    let zero = start.build_load(0);
    start.build_exit(zero);

    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .open("output.s")?;
    let mut module = Module::default();
    module.append_block(&start);
    module.append_block(&if_zero);
    module.generate_code(&mut file)?;

    run_command("nasm -f elf64 -o output.o output.s")?;
    run_command("ld -o output output.o")?;

    Ok(())
}
