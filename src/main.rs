use std::fs::File;
use std::io::Read;

use ctl::ir_first::generate_insts;
use ctl::parser::parse;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        println!("Compiler needs at least 1 argument - source file name");
        return ();
    }

    // Work with file, get the file's contents to a String
    let file_res = File::open(&args[1]);
    let file: File;
    match file_res {
        Ok(f) => file = f,
        Err(mes) => {
            println!("{}", mes);
            return;
        }
    };
    let mut buf_reader = std::io::BufReader::new(file);
    let mut contents = String::new();
    match buf_reader.read_to_string(&mut contents) {
        Ok(_s) => (),
        Err(mes) => {
            println!("{}", mes);
            return;
        }
    };

    // Parse the contents
    let funcs_res = parse(contents);
    let funcs;
    match funcs_res {
        Ok(f) => funcs = f,
        Err(mes) => {
            println!("Parse error: {}", mes);
            std::process::exit(1);
        }
    };

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
}
