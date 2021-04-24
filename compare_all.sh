#!/bin/bash

# Compara o decode de todos as entradas com o decode de referência.

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
    "16.automato"
    "17.rng"
    "18.naive_dgemm"
    "19.regular_dgemm"
    "20.blocking_dgemm"
    "21.mandelbrot")

for i in ${tests[@]}; do
    echo -en "=> Rodando $i..."

     diff -wB <(target/release/minips-rs decode res/$i) <(res/minips decode res/$i) && echo -e " \e[32mok\e[0m" || echo -e " \e[31mFAIL\e[0m"
done
