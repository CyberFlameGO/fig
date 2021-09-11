use std::{fs::OpenOptions, io::Write, process::Command};

use fig::ir::{Block, Module};

fn run_command(cmd: &str) -> std::io::Result<()> {
    println!("+ {}", cmd);
    let output = Command::new("sh").arg("-c").arg(cmd).output()?;
    std::io::stdout().write_all(&output.stdout)?;
    std::io::stderr().write_all(&output.stderr)?;
    Ok(())
}

fn main() -> std::io::Result<()> {
    let mut start = Block::new("_start".into());

    let left = start.build_constant(5);
    let right = start.build_constant(7);
    let res = start.build_multiply(left, right);
    start.build_call("put_int".into(), Some(res));

    let exit = start.build_constant(0);
    start.build_exit(exit);

    let mut module = Module::default();
    module.append_block(&start);

    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open("output.s")?;
    module.generate_code(&mut file)?;

    run_command("nasm -f elf64 -o output.o output.s")?;
    run_command("nasm -f elf64 -o lib/lib.o lib/lib.s")?;
    run_command("ld -o output output.o lib/lib.o")?;

    Ok(())
}
