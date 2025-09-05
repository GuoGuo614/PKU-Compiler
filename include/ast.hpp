#pragma once
#include <memory>
#include <string>
#include <iostream>

enum class FuncType {
    _INT, _VOID, _LONG
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
        std::unique_ptr<BaseAST> exp;

        void Dump() const override {
            std::cout << "StmtAST { " << _return << " ";
            exp->Dump();
            std::cout << " }";
        }
};

class PrimaryExpAST : public BaseAST {
    public:
        enum PrimaryType {
            EXPR,
            NUMBER
        };
        
        PrimaryType type;
        struct {
            std::unique_ptr<BaseAST> exp;
            std::unique_ptr<BaseAST> number;
        } data;

        void Dump() const override {
            std::cout << "PrimaryExpAST { ";
            if (type == EXPR) {
                data.exp->Dump();
            } else {
                data.number->Dump();
            }
            std::cout << " }";
        }
};

class ExprAST : public BaseAST {
    public:
        std::unique_ptr<BaseAST> unExp;

        void Dump() const override {
            std::cout << "ExprAST { ";
            unExp->Dump();
            std::cout << " }";
        }
};

class UnaryOpAST : public BaseAST {
    public:
        std::string op;  // 存储运算符字符串："+", "-", "!"
        
        void Dump() const override {
            std::cout << "UnaryOpAST { " << op << " }";
        }
};

class UnaryExpAST : public BaseAST {
    public:
        enum UnaryType {
            PRIMARY, UNARY
        };

        UnaryType type;
        struct {
            std::unique_ptr<BaseAST> primary;
            struct {
                std::unique_ptr<BaseAST> unOp;
                std::unique_ptr<BaseAST> unExp;
            } unary;
        } data;

        void Dump() const override {
            std::cout << "UnaryExpAST { ";
            if (type == PRIMARY) {
                data.primary->Dump();
            } else {
                data.unary.unOp->Dump();
                std::cout << " ";
                data.unary.unExp->Dump();
            }
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
