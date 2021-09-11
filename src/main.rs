use std::{fs::OpenOptions, io::Write, process::Command};

use fig::ir::{Block, Function, Module};

fn run_command(cmd: &str) -> std::io::Result<()> {
    println!("+ {}", cmd);
    let output = Command::new("sh").arg("-c").arg(cmd).output()?;
    std::io::stdout().write_all(&output.stdout)?;
    std::io::stderr().write_all(&output.stderr)?;
    Ok(())
}

fn main() -> std::io::Result<()> {
    let mut entry = Block::new(".entry".into());

    let mut end = Block::new(".end".into());
    let exit_code = end.build_constant(0);
    end.build_exit(exit_code);

    let var = entry.build_alloc(8);
    let val = entry.build_constant(10);
    entry.build_store(val, var);

    let mut r#loop = Block::new(".loop".into());
    r#loop.build_call("put_int".into(), Some(var));
    let one = r#loop.build_constant(1);
    r#loop.build_subtract(var, one);
    r#loop.build_jump_if_zero(var, end.name.clone());
    r#loop.build_jump(r#loop.name.clone());

    let mut func = Function::new("_start".into());
    func.append_block(&entry);
    func.append_block(&r#loop);
    func.append_block(&end);

    let mut module = Module::default();
    module.append_func(&func);

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
