#include "indy_api.h"

struct foo_t {
    int m_a;
    int m_b;
};

void modify_foo(struct foo_t *foo) {
    foo->m_a *= 2;
    foo->m_b *= 2;
}

void __start() {
    indy_print_str("Digite um numero: ");
    int a = indy_read_int();
    indy_print_str("Voce digitou: ");
    indy_print_int(a);
    indy_print_str("\n");

    struct foo_t foo = {
        .m_a = 10,
        .m_b = 20,
    };

    indy_print_str("foo.m_a = ");
    indy_print_int(foo.m_a);
    indy_print_str(", foo.m_b = ");
    indy_print_int(foo.m_b);
    indy_print_str("\n");

    modify_foo(&foo);

    indy_print_str("foo.m_a = ");
    indy_print_int(foo.m_a);
    indy_print_str(", foo.m_b = ");
    indy_print_int(foo.m_b);
    indy_print_str("\n");

    indy_halt();
}
