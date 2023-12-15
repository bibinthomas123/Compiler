#![allow(clippy::needless_return)]
#![allow(dead_code)]
#![allow(unused_assignments)]


use crate::codegen::CTranspiler;
use crate::compilation_unit::CompilationUnit;
mod ast;
mod diagnostics;
mod text;
mod compilation_unit;
mod typings;
mod codegen;

fn main() -> Result<(), ()> {
    let input = "\
     func test() -> int {
        let a = 1;
        let b = 2;
        let c = a + b;
        return c;
     }
     let d = test();
     d
    ";
    let mut compilation_unit = CompilationUnit::compile(input).map_err(|_| ())?;
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
