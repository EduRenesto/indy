# minips-rs

minips-rs é um emulador de um subset da ISA MIPS, batizado carinhosamente de
MINIPS, escrito em Rust.

Esse README é um WIP e será melhorado o mais cedo possível.

## Requisitos

- Toolchain Rust: provavelmente stable

## Executando

Para desmontar um binário, mostrando seu código Assembly equivalente:

```sh
$ cargo run -- decode file
```

Onde `file` é o prefixo dos arquivos em questão (por ex, `02.hello`).

Para executar o binário, utilize:

```sh
$ cargo run -- run file
```
