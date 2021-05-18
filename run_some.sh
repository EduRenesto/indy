#!/bin/bash

# Executa todas as entradas-teste.

tests=("1 res/09.contador"
    "2 res/09.contador"
    "3 res/09.contador"
    "4 res/09.contador"
    "5 res/09.contador"
    "6 res/09.contador"
    "2 res/17.rng"
    "3 res/18.naive_dgemm"
    "4 res/19.regular_dgemm"
    "5 res/20.blocking_dgemm"
    "6 res/21.mandelbrot")

for i in "${tests[@]}"; do
    echo -e "\e[31m=> Rodando $i...\e[0m"
    target/release/indy run $i
    echo ""
    sleep 2
done
