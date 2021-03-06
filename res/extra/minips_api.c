#include "indy_api.h"

void indy_print_int(int i) {
    asm volatile(
        "addi $v0, $zero, 1;"
        "add $a0, $zero, %[i];"
        "syscall" 
        : 
        : [i] "r" (i)
        : "$a0", "$v0");
}

void indy_print_char(char c) {
    asm volatile(
        "addi $v0, $zero, 11;"
        "add $a0, $zero, %[c];"
        "syscall" 
        : 
        : [c] "r" (c)
        : "$a0", "$v0");
}

void indy_print_str(const char* str) {
    asm volatile(
        "addi $v0, $zero, 4;"
        "add $a0, $zero, %[str];"
        "syscall" 
        : 
        : [str] "r" (str)
        : "$a0", "$v0");
}

int indy_read_int() {
    int a;
    asm volatile(
        "addi $v0, $zero, 5;"
        "syscall;"
        "add %[a], $v0, $zero;"
        : [a] "=r" (a)
        :
        : "$v0");
    
    return a;
}

void indy_halt() {
    asm volatile(
        "addi $v0, $zero, 10;"
        "syscall"
        :
        :
        : "$v0");
}
