void print_str(const char *str) {
    /*asm volatile("add $a0, $zero, %[str]" : : [str] "r" (str) : "$a0");*/
    /*asm volatile("addi $v0, $zero, 4" : : : "$v0");*/
    /*asm("syscall");*/

    asm volatile(
        "add $a0, $zero, %[str];"
        "addi $v0, $zero, 4;"
        "syscall;"
        :
        : [str] "r" (str)
        : "$a0", "$v0");
}

void finish() __attribute__ ((naked));
void finish() {
    asm volatile("addi $v0, $zero, 10" ::: "$v0");
    asm volatile("syscall");
}

const char *hello = "Hello World from MIPS C!";

void __start() {
    print_str(hello);
    finish();
}
