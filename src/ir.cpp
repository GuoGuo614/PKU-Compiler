#include "../include/ir.hpp"

using namespace std;

koopa_raw_program_t RawProgramGen::generateKoopaIR(CompUnitAST *ast, const char* output) {
    koopa_raw_program_t raw_program = this->raw_program_parse_from_ast(ast);
    koopa_program_t ir_program;
    koopa_generate_raw_to_koopa(&raw_program, &ir_program);

    size_t len = 0;
    koopa_dump_to_string(ir_program, nullptr, &len);
    
    char* buffer = new char[len + 1];
    koopa_dump_to_string(ir_program, buffer, &len);
    // 仅在需要输出 Koopa 文件时才写文件，-riscv 模式会传入 nullptr
    if (output != nullptr) {
        koopa_dump_to_file(ir_program, output);
    }
    
    delete[] buffer;
    koopa_delete_program(ir_program);
    
    return raw_program;
}

koopa_raw_program_t RawProgramGen::raw_program_parse_from_ast(CompUnitAST *ast) {
    koopa_raw_program_t raw_program;

    raw_program.funcs = this->funcs_parse_from_ast(static_cast<FuncDefAST*>(ast->func_def.get()));
    raw_program.values = this->values_parse_from_ast();

    return raw_program;
}

koopa_raw_slice_t RawProgramGen::values_parse_from_ast() {
    koopa_raw_slice_t values_slices;
    values_slices.buffer = nullptr;
    values_slices.len = 0;
    values_slices.kind = KOOPA_RSIK_VALUE;
    return values_slices;
}

koopa_raw_slice_t RawProgramGen::funcs_parse_from_ast(FuncDefAST *ast) {
    // 创建函数数据结构
    koopa_raw_function_data_t *func_data = new koopa_raw_function_data_t();
    
    // 设置函数名
    std::string func_name = "@" + ast->ident;
    char* name_copy = new char[func_name.length() + 1];
    strcpy(name_copy, func_name.c_str());
    func_data->name = name_copy;
    
    // 设置函数类型 - 需要从 FuncTypeAST 获取
    FuncTypeAST *func_type = static_cast<FuncTypeAST*>(ast->func_type.get());
    koopa_raw_type_kind_t *type_kind = new koopa_raw_type_kind_t();
    type_kind->tag = KOOPA_RTT_FUNCTION;
    
    // 设置返回类型
    koopa_raw_type_kind_t *ret_type = new koopa_raw_type_kind_t();
    if (func_type->type == FuncType::_INT) {
        ret_type->tag = KOOPA_RTT_INT32;
    } else if (func_type->type == FuncType::_VOID) {
        ret_type->tag = KOOPA_RTT_UNIT;
    }
    
    // 设置函数类型的参数和返回类型
    type_kind->data.function.ret = ret_type;
    type_kind->data.function.params.buffer = nullptr;
    type_kind->data.function.params.len = 0;
    type_kind->data.function.params.kind = KOOPA_RSIK_TYPE;
    
    func_data->ty = type_kind;
    
    // 设置参数列表（当前为空）
    func_data->params.buffer = nullptr;
    func_data->params.len = 0;
    func_data->params.kind = KOOPA_RSIK_VALUE;
    
    // 设置基本块
    func_data->bbs = this->bbs_parse_from_ast(static_cast<BlockAST*>(ast->block.get()));
    
    // 创建包含单个函数的 slice
    koopa_raw_slice_t funcs_slice;
    funcs_slice.buffer = new const void*[1];
    funcs_slice.buffer[0] = func_data;
    funcs_slice.len = 1;
    funcs_slice.kind = KOOPA_RSIK_FUNCTION;
    
    return funcs_slice;
}

koopa_raw_slice_t RawProgramGen::bbs_parse_from_ast(BlockAST *ast) {
    // 创建基本块数据结构
    koopa_raw_basic_block_data_t *bb_data = new koopa_raw_basic_block_data_t();
    
    // 设置基本块名称
    std::string bb_name = "%entry";
    char* name_copy = new char[bb_name.length() + 1];
    strcpy(name_copy, bb_name.c_str());
    bb_data->name = name_copy;
    
    // 设置参数列表（当前为空）
    bb_data->params.buffer = nullptr;
    bb_data->params.len = 0;
    bb_data->params.kind = KOOPA_RSIK_VALUE;
    
    // 设置使用者列表（当前为空）
    bb_data->used_by.buffer = nullptr;
    bb_data->used_by.len = 0;
    bb_data->used_by.kind = KOOPA_RSIK_VALUE;
    
    // 设置指令列表
    bb_data->insts = this->insts_parse_from_ast(static_cast<StmtAST*>(ast->stmt.get()));
    
    // 创建包含单个基本块的 slice
    koopa_raw_slice_t bbs_slice;
    bbs_slice.buffer = new const void*[1];
    bbs_slice.buffer[0] = bb_data;
    bbs_slice.len = 1;
    bbs_slice.kind = KOOPA_RSIK_BASIC_BLOCK;
    
    return bbs_slice;
};

koopa_raw_slice_t RawProgramGen::insts_parse_from_ast(StmtAST *stmt) {
    // 创建返回指令的值数据结构
    koopa_raw_value_data_t *ret_value = new koopa_raw_value_data_t();
    
    // 设置返回指令的类型
    koopa_raw_type_kind_t *ret_type = new koopa_raw_type_kind_t();
    ret_type->tag = KOOPA_RTT_UNIT;  // return 指令本身的类型是 unit
    ret_value->ty = ret_type;
    
    // 设置名称（返回指令通常没有名称）
    ret_value->name = nullptr;
    
    // 设置使用者列表（当前为空）
    ret_value->used_by.buffer = nullptr;
    ret_value->used_by.len = 0;
    ret_value->used_by.kind = KOOPA_RSIK_VALUE;
    
    // 设置返回指令的类型和数据
    ret_value->kind.tag = KOOPA_RVT_RETURN;
    
    // 解析返回值（从 ExprAST 获取）
    ExprAST *expr = static_cast<ExprAST*>(stmt->exp.get());
    koopa_raw_value_t int_value = this->expr_parse_from_ast(expr);
    
    // 设置返回指令的返回值
    ret_value->kind.data.ret.value = int_value;
    
    // 创建包含指令的 slice
    koopa_raw_slice_t insts_slice;
    insts_slice.buffer = new const void*[1];
    insts_slice.buffer[0] = ret_value;
    insts_slice.len = 1;
    insts_slice.kind = KOOPA_RSIK_VALUE;
    
    return insts_slice;
}

koopa_raw_value_t RawProgramGen::expr_parse_from_ast(ExprAST* expr) {
    auto ue = static_cast<UnaryExpAST*>(expr->unExp.get());
    return this->unary_expr_parse_from_ast(ue);
}

koopa_raw_value_t RawProgramGen::unary_expr_parse_from_ast(UnaryExpAST* unary_expr) {
    if (unary_expr->type == UnaryExpAST::PRIMARY) {
        return this->primary_expr_parse_from_ast(static_cast<PrimaryExpAST*>(unary_expr->data.primary.get()));
    } else {
        koopa_raw_value_t value = this->unary_expr_parse_from_ast(static_cast<UnaryExpAST*>(unary_expr->data.unary.unExp.get()));
        auto unop = static_cast<UnaryOpAST*>(unary_expr->data.unary.unOp.get());
        switch (unop->op[0]) {
        case '+':
            return value; break;
        case '-':
            return create_integer_value(-value->kind.data.integer.value);
        case '!':
            return create_integer_value(!value->kind.data.integer.value);
        default:
            break;
        }
    }
    return nullptr;
}

koopa_raw_value_t RawProgramGen::primary_expr_parse_from_ast(PrimaryExpAST* primary) {
    if (primary->type == PrimaryExpAST::NUMBER) {
        // 创建整数常量
        NumberAST* number = static_cast<NumberAST*>(primary->data.number.get());
        int _int_ = std::stoi(number->number);
        koopa_raw_value_t int_value = create_integer_value(_int_);
        
        return int_value;
    } else if (primary->type == PrimaryExpAST::EXPR) {
        // 去括号，递归处理表达式
        return this->expr_parse_from_ast(static_cast<ExprAST*>(primary->data.exp.get()));
    }
    return nullptr;
}
