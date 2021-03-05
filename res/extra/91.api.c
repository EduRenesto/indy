#include "minips_api.h"

void __start() {
    print_str("Digite um número: ");
    int a = read_int();
    print_str("Você digitou: ");
    print_int(a);
    print_char('\n');

    halt();
}
