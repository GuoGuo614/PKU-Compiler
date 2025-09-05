#pragma once
#include <cstdio>
#include <string>
#include <cstring>
#include <memory>
#include "../include/ast.hpp"
#include "../include/koopa.h"

using namespace std;

class RawProgramGen {
    public:
        koopa_raw_program_t generateKoopaIR(CompUnitAST *ast, const char* output);

    private:
        koopa_raw_program_t raw_program_parse_from_ast(CompUnitAST *ast);
        koopa_raw_slice_t values_parse_from_ast();
        koopa_raw_slice_t funcs_parse_from_ast(FuncDefAST *ast);
        koopa_raw_slice_t bbs_parse_from_ast(BlockAST *ast);
        koopa_raw_slice_t insts_parse_from_ast(StmtAST *stmt);
        koopa_raw_value_t expr_parse_from_ast(ExprAST* expr);
        koopa_raw_value_t unary_expr_parse_from_ast(UnaryExpAST* unary_expr);
        koopa_raw_value_t primary_expr_parse_from_ast(PrimaryExpAST* primary);
};

koopa_raw_value_t create_integer_value(int value);