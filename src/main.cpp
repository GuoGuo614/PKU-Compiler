#include <cassert>
#include <cstdio>
#include <string>
#include <memory>
#include "../include/ast.hpp"
#include "../include/ir.hpp"

using namespace std;

extern FILE *yyin;
extern int yyparse(unique_ptr<BaseAST> &ast);

int main(int argc, const char *argv[]) {
  // 解析命令行参数. 测试脚本/评测平台要求你的编译器能接收如下参数:
  // compiler 模式 输入文件 -o 输出文件
  assert(argc == 5);
  auto mode = argv[1];
  auto input = argv[2];
  auto output = argv[4];

  // 打开输入文件, 并且指定 lexer 在解析的时候读取这个文件
  yyin = fopen(input, "r");
  assert(yyin);

  // 调用 parser 函数, parser 函数会进一步调用 lexer 解析输入文件的
  unique_ptr<BaseAST> ast;
  auto ret = yyparse(ast);
  assert(!ret);

  // 打印 AST，现需要将 AST 转换为 Koopa IR 程序
  RawProgramGen program_gen;
  string koopa_ir = program_gen.generateKoopaIR(static_cast<CompUnitAST*>(ast.get()), output);

  return 0;
}
