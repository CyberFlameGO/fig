use std::{
    fs::OpenOptions,
    process::{Command, Output},
};

use fig::ir::Block;

fn run_command(cmd: &str) -> std::io::Result<Output> {
    println!("+ {}", cmd);
    Command::new("sh").arg("-c").arg(cmd).output()
}

fn main() -> std::io::Result<()> {
    let mut start = Block::new("_start".into());

    let left = start.build_load(3);
    let right = start.build_load(5);
    let res = start.build_multiply(left, right);

    let left = start.build_load(10);
    let right = res;
    let res = start.build_add(left, right);

    let left = res;
    let right = start.build_load(4);
    let res = start.build_subtract(left, right);

    let left = res;
    let right = start.build_load(3);
    let res = start.build_divide(left, right);

    start.build_exit(res);

    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .open("output.s")?;
    start.generate_code(&mut file)?;

    run_command("nasm -f elf64 -o output.o output.s")?;
    run_command("ld -o output output.o")?;

    Ok(())
}
