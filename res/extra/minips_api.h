/*
 * Funcoes para chamar as syscalls do minips
 */

/// Printa um inteiro.
extern void print_int(int i);
/// Printa um caractere.
extern void print_char(char c);
/// Printa uma string.
extern void print_str(const char* str);

/// Lê um int da entrada padrão.
extern int read_int();

/// Encerra a execução do programa.
extern void halt();
