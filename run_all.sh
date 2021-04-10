#!/bin/bash

# Executa todas as entradas-teste.

tests=("01.soma"
    "02.hello"
    "03.input"
    "04.branches"
    "05.fibo"
    "06.collatz"
    "07.loadstore"
    "08.sort"
    "09.contador"
    "10.hello++"
    "11.surpresinha"
    "12.branch_delay_slot"
    "13.arit"
    "14.flutuantes"
    "15.pi"
    "16.automato")

for i in ${tests[@]}; do
    echo -e "\e[31m=> Rodando $i...\e[0m"
    target/release/minips-rs run res/$i
    echo ""
    sleep 2
done
