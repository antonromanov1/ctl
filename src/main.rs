use std::fs::File;
use std::io::Read;

use ctl::ir_first::generate_insts;
use ctl::parser::parse;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        println!("Compiler needs at least 1 argument - source file name");
        return Ok(());
    }

    // Work with file, get the file's contents to a String
    let file = File::open(&args[1])?;
    let mut buf_reader = std::io::BufReader::new(file);
    let mut contents = String::new();
    buf_reader.read_to_string(&mut contents)?;

    // Parse the contents
    let funcs = parse(contents)?;

    // Generate IR first for each function and dump it to the stdout
    for func in funcs {
        let insts = generate_insts(&func);

        println!(
            "Function {}, {} instructions:",
            func.get_name(),
            insts.len()
        );
        for (i, inst) in insts.iter().enumerate() {
            println!("{}. {}", i, inst.to_string());
        }
        println!("");
    }

    Ok(())
}
