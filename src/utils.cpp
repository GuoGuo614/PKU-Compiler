#include "../include/ir.hpp"

koopa_raw_value_t create_integer_value(int value) {
    koopa_raw_value_data_t *int_value = new koopa_raw_value_data_t();
    koopa_raw_type_kind_t *int_type = new koopa_raw_type_kind_t();
    int_type->tag = KOOPA_RTT_INT32;
    int_value->ty = int_type;
    int_value->name = nullptr;
    int_value->used_by.buffer = nullptr;
    int_value->used_by.len = 0;
    int_value->used_by.kind = KOOPA_RSIK_VALUE;
    int_value->kind.tag = KOOPA_RVT_INTEGER;
    int_value->kind.data.integer.value = value;
    return int_value;
}