#pragma once
#include <memory>
#include <string>
#include <iostream>

enum class FuncType {
    _INT,
    _VOID,
    _LONG
};

inline std::string funcTypeToString(FuncType type) {
    switch (type) {
        case FuncType::_INT: return "int";
        case FuncType::_VOID: return "void";
        case FuncType::_LONG: return "long";
        default: return "unknown";
    }
}

class BaseAST {
    public:
        virtual ~BaseAST() = default;

        virtual void Dump() const = 0;
};

class CompUnitAST : public BaseAST {
    public:
        std::unique_ptr<BaseAST> func_def;

        void Dump() const override {
            std::cout << "CompUnitAST { ";
            func_def->Dump();
            std::cout << " }";
        }
};

class FuncDefAST : public BaseAST {
    public:
        std::unique_ptr<BaseAST> func_type;
        std::string ident;
        std::unique_ptr<BaseAST> block;

        void Dump() const override {
            std::cout << "FuncDefAST { ";
            func_type->Dump();
            std::cout << ", " << ident << ", ";
            block->Dump();
            std::cout << " }";
        }
};

class FuncTypeAST : public BaseAST {
    public:
        FuncType type;

        void Dump() const override {
            std::cout << "FuncTypeAST { ";
            std::cout << funcTypeToString(type) << " }";
        }
};

class BlockAST : public BaseAST {
    public:
        std::unique_ptr<BaseAST> stmt;

        void Dump() const override {
            std::cout << "BlockAST { ";
            stmt->Dump();
            std::cout << " }";
        }
};

class StmtAST : public BaseAST {
    public:
        std::string _return;
        std::unique_ptr<BaseAST> number;

        void Dump() const override {
            std::cout << "StmtAST { " << _return << " ";
            number->Dump();
            std::cout << " }";
        }
};

class NumberAST : public BaseAST {
    public:
        std::string number;

        void Dump() const override {
            std::cout << "NumberAST { " << number << " }";
        }
};
