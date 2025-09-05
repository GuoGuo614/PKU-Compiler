#include "../include/riscv.hpp"
#include <fstream>
#include <sstream>
#include <cassert>

using namespace std;

/* koopa_raw_program_t RiscVGen::read_raw_from_file(const std::string& input_file) {
    koopa_program_t program;

    koopa_parse_from_file(input_file.c_str(), &program);
    koopa_raw_program_builder_t builder = koopa_new_raw_program_builder();

    return koopa_build_raw_program(builder, program);
} */

std::string RiscVGen::generateRiscV(koopa_raw_program_t raw, const std::string& output_file) {
    std::ostringstream asm_code;
    
    this->Visit(raw, asm_code);

    // 写入文件
    std::ofstream file(output_file);
    if (!file.is_open()) {
        throw std::runtime_error("Cannot open output file: " + output_file);
    }
    
    file << asm_code.str();
    file.close(); 
    
    return asm_code.str();
}

void gen_global_ident(const koopa_raw_slice_t &funcs, std::ostringstream& out) {
    out << "\t.globl ";
    // 遍历函数，为每个函数生成 .globl 声明
    for (size_t i = 0; i < funcs.len; i++) {
        auto func = static_cast<koopa_raw_function_t>(funcs.buffer[i]);
        std::string func_name = func->name;
        if (!func_name.empty() && (func_name[0] == '@' || func_name[0] == '%')) {
            func_name = func_name.substr(1);
        }
        out << func_name << " ";
    }
    out << "\n";
}

void RiscVGen::Visit(const koopa_raw_program_t &program, std::ostringstream& out) {
    // 生成 .text 段
    out << "\t.text\n";
    // 生成 .global 段
    gen_global_ident(program.funcs, out);

    this->Visit(program.values, out);
    this->Visit(program.funcs, out);
}

void RiscVGen::Visit(const koopa_raw_slice_t &slice, std::ostringstream& out) {
    // 处理切片，根据类型分发
    for (size_t i = 0; i < slice.len; i++) {
        auto ptr = slice.buffer[i];
        switch (slice.kind)
        {
        case KOOPA_RSIK_FUNCTION:
            Visit(static_cast<koopa_raw_function_t>(ptr), out);
            break;
        case KOOPA_RSIK_BASIC_BLOCK:
            Visit(static_cast<koopa_raw_basic_block_t>(ptr), out);
            break;
        case KOOPA_RSIK_VALUE:
            Visit(static_cast<koopa_raw_value_t>(ptr), out);
            break;
        default:
            assert(0);
            break;
        }
    }
}

void RiscVGen::Visit(const koopa_raw_function_t &func, std::ostringstream& out) {
    // 处理函数名，去掉前缀 @ 或 %
    std::string func_name = func->name;
    if (!func_name.empty() && (func_name[0] == '@' || func_name[0] == '%')) {
        func_name = func_name.substr(1);
    }
    out << func_name << ":\n";
    Visit(func->bbs, out);
}

void RiscVGen::Visit(const koopa_raw_basic_block_t &bb, std::ostringstream& out) {
    Visit(bb->insts, out);
}

void RiscVGen::Visit(const koopa_raw_value_t &value, std::ostringstream& out) {
    // 根据指令类型判断后续需要如何访问
    const auto &kind = value->kind;
    switch (kind.tag) {
        case KOOPA_RVT_RETURN:
        // 访问 return 指令
        Visit(kind.data.ret, out);
        break;
        case KOOPA_RVT_INTEGER:
        // 访问 integer 指令
        Visit(kind.data.integer, out);
        break;
        default:
        // 其他类型暂时遇不到
        assert(false);
    }
}

void RiscVGen::Visit(const koopa_raw_return_t &_return, std::ostringstream& out) {
    out << "\tli a0, ";
    Visit(_return.value, out);
    out << "\n";
    out << "\tret\n";
}

void RiscVGen::Visit(const koopa_raw_integer_t &_interger, std::ostringstream& out) {
    out << _interger.value;
}