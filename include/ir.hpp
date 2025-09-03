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
        std::string generateKoopaIR(CompUnitAST *ast, const char* output);

    private:
        koopa_raw_program_t raw_program_parse_from_ast(CompUnitAST *ast);
        koopa_raw_slice_t values_parse_from_ast();
        koopa_raw_slice_t funcs_parse_from_ast(FuncDefAST *ast);
        koopa_raw_slice_t bbs_parse_from_ast(BlockAST *ast);
        koopa_raw_slice_t insts_parse_from_ast(StmtAST *stmt);
};