// Teste simples da cache.
// A ideia é escrever em todas as linhas valores sequenciais. 
// Assim, fica mais fácil de debuggar.
addi $v0, $zero, 500
syscall
addi $t0, $zero, 0
sw $t0, 0($gp)
addi $t0, $zero, 1
sw $t0, 4($gp)
addi $t0, $zero, 2
sw $t0, 8($gp)
addi $t0, $zero, 3
sw $t0, 12($gp)
addi $t0, $zero, 4
sw $t0, 16($gp)
addi $t0, $zero, 5
sw $t0, 20($gp)
addi $t0, $zero, 6
sw $t0, 24($gp)
addi $t0, $zero, 7
sw $t0, 28($gp)
addi $v0, $zero, 500
syscall
addi $v0, $zero, 10
syscall
