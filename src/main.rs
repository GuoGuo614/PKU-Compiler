use lalrpop_util::lalrpop_mod;
use std::env::args;
use std::fs::read_to_string;
use std::fs::File;
use std::io::{self, Result};

pub mod assm;
pub mod ast;
pub mod kira;
pub mod ctx;
pub mod const_eval;
pub mod symbol;
pub mod alloc;

use assm::GenerateAsm;
use kira::GenerateIR;

// 引用 lalrpop 生成的解析器
// 因为我们刚刚创建了 sysy.lalrpop, 所以模块名是 sysy
lalrpop_mod!(sysy);

fn main() -> Result<()> {
    // 解析命令行参数
    let mut args = args();
    args.next();
    let mode = args.next().unwrap();
    let input = args.next().unwrap();
    args.next();
    let output = args.next().unwrap();

    // 读取输入文件
    let input = read_to_string(input)?;

    // 调用 lalrpop 生成的 parser 解析输入文件
    let ast = sysy::CompUnitParser::new().parse(&input).unwrap();

    match mode.as_str() {
        "-riscv" => {
            let program = ast.generate_ir();
            let mut f = File::create(output)?;
            program.write_assm_to(&mut f);
            Ok(())
        }
        "-koopa" => {
            // 调用写函数，由 AST 生成 IR
            ast.write_ir(output);
            Ok(())
        }
        _ => Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("unsupported mode: {}", mode),
        )),
    }
}
