#include "minips_api.h"

struct foo_t {
    int m_a;
    int m_b;
};

void modify_foo(struct foo_t *foo) {
    foo->m_a *= 2;
    foo->m_b *= 2;
}

void __start() {
    minips_print_str("Digite um numero: ");
    int a = minips_read_int();
    minips_print_str("Voce digitou: ");
    minips_print_int(a);
    minips_print_str("\n");

    struct foo_t foo = {
        .m_a = 10,
        .m_b = 20,
    };

    minips_print_str("foo.m_a = ");
    minips_print_int(foo.m_a);
    minips_print_str(", foo.m_b = ");
    minips_print_int(foo.m_b);
    minips_print_str("\n");

    modify_foo(&foo);

    minips_print_str("foo.m_a = ");
    minips_print_int(foo.m_a);
    minips_print_str(", foo.m_b = ");
    minips_print_int(foo.m_b);
    minips_print_str("\n");

    minips_halt();
}
