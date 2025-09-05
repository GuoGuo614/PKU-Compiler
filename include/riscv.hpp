#pragma once
#include <cstdio>
#include <string>
#include <cstring>
#include <memory>
#include "../include/ast.hpp"
#include "../include/koopa.h"

using namespace std;

class RiscVGen {
    public:
        // koopa_raw_program_t read_raw_from_file(const std::string& input_file);
        std::string generateRiscV(koopa_raw_program_t raw_program, const std::string& output_file);

    private:
        void Visit(const koopa_raw_program_t &program, std::ostringstream& out);
        void Visit(const koopa_raw_slice_t &slice, std::ostringstream& out);
        void Visit(const koopa_raw_function_t &func, std::ostringstream& out);
        void Visit(const koopa_raw_basic_block_t &bb, std::ostringstream& out);
        void Visit(const koopa_raw_value_t &value, std::ostringstream& out);
        void Visit(const koopa_raw_return_t &_return, std::ostringstream& out);
        void Visit(const koopa_raw_integer_t &_interger, std::ostringstream& out);
};