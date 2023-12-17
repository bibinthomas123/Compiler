#![allow(clippy::needless_return)]
#![allow(dead_code)]
#![allow(unused_assignments)]
#![allow(unused_imports)]


use std::fs::File;
use std::io::Read;

use termion::input;

use crate::codegen::CTranspiler;
use crate::compilation_unit::CompilationUnit;
mod ast;
mod diagnostics;
mod text;
mod compilation_unit;
mod typings;
mod codegen;

fn main() -> Result<(), ()> {

    let args:Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        eprintln!("No file found");
        std::process::exit(1);
    }

    let input_file = &args[1];
    // Split the input file into file name and extension
    let (_file_name, file_extension) = match input_file.rsplit_once('.') {
        Some((name, extension)) => (name, extension),
        None => {
            eprintln!("Error: File has no extension");
            std::process::exit(1);
        }
    };

    if file_extension != "cx"{
        println!("the file cannot be compiled: Unknown file extension {}" , file_extension);
        std::process::exit(1);
    }
    // println!("{}",file_extension);


    //reading the contents from the file 
    let mut input = String::new();
    match File::open(input_file).and_then(|mut file|file.read_to_string(&mut input)){
        Ok(_)=>{}
        Err(_)=>{
            eprintln!("Error: failed to read the file {}" , input_file);
            std::process::exit(1);
        }
    } 
    let mut compilation_unit = CompilationUnit::compile(&input).map_err(|_| ())?;
    compilation_unit.run();
    let c_transpiler = CTranspiler::new(&compilation_unit.global_scope);
    let _transpiled_code = c_transpiler.transpile(&mut compilation_unit.ast);
    // println!("{}", transpiled_code);
    // let mut c_file = File::create("out.c").unwrap();
    // c_file.write_all(transpiled_code.as_bytes()).unwrap();
    // // compile with clang using rust
    // Command::new("clang")
    //     .arg("out.c")
    //     .arg("-o")
    //     .arg("out")
    //     .status()
    //     .unwrap();
    // // run the compiled binary
    // Command::new("./out")
    //     .status()
    //     .unwrap();
    Ok(())
}
