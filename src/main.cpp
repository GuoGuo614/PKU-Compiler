#include <cassert>
#include <cstdio>
#include <string>
#include <memory>
#include "../include/ast.hpp"
#include "../include/ir.hpp"
#include "../include/riscv.hpp"

using namespace std;

extern FILE *yyin;
extern int yyparse(unique_ptr<BaseAST> &ast);

int main(int argc, const char *argv[]) {
  // 解析命令行参数. 测试脚本/评测平台要求你的编译器能接收如下参数:
  // compiler 模式 输入文件 -o 输出文件
  assert(argc == 5);
  string mode = argv[1];
  auto input = argv[2];
  auto output = argv[4];

  // 打开输入文件, 并且指定 lexer 在解析的时候读取这个文件
  yyin = fopen(input, "r");
  assert(yyin);

  // 调用 parser 函数, parser 函数会进一步调用 lexer 解析输入文件的
  unique_ptr<BaseAST> ast;
  auto ret = yyparse(ast);
  assert(!ret);

  // 根据模式处理 AST
  RawProgramGen program_gen;
  
  if (mode == "-koopa") {
    // 将 AST 转换为 Koopa IR 文件
    program_gen.generateKoopaIR(static_cast<CompUnitAST*>(ast.get()), output);
  } else if (mode == "-riscv") {
    // 先将 AST 转换为 raw program，再生成 RISC-V 汇编
    koopa_raw_program_t raw = program_gen.generateKoopaIR(static_cast<CompUnitAST*>(ast.get()), nullptr);
    
    // 将 raw program 转换为 riscv 汇编文件
    RiscVGen riscv_gen;
    riscv_gen.generateRiscV(raw, output);
  } else {
    cerr << "Unknown mode: " << mode << endl;
    return 1;
  }

  return 0;
}
