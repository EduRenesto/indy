# indy

## ATENÇÃO: ATUALIZE SEU TOOLCHAIN RUST!

Essa última versão do `indy` usa em peso Const Generics, disponível na
branch estável do `rustc` a partir da versão 1.51. Então, se sua versão do 
toolchain é `stable` e mais antiga que 1.51, atualize! (hint: `rustup update`)

---

`indy` é um emulador que implementa a ISA MIPS, escrito em Rust para a
disciplina Arquitetura de Computadores, na UFABC durante o quadrimeste 2021.1
pelo grande prof. dr. Emilio Francesquini.

Esse emulador (por enquanto) não implementa a ISA inteira -- as instruções
estão sendo implementadas conforme os programas que quero executar precisem
delas. Atualmente, todos os programas-entrada disponibilizados pelo professor
são decodificados e executados com sucesso pelo indy.

Como tenho enorme interesse por baixo nível, estou usando esse projeto para
explorações pessoais (vide seção *extras*).

## Relatório

O relatório está disponível em formato PDF na pasta `relatorio/relatorio.pdf`.
Para compilar, é necessário o `pandoc`, uma distribuição \LaTeX funcional e
basta rodar `make` na pasta `relatorio`.

## Requisitos

- Toolchain Rust: o projeto está sendo desenvolvido e testado utilizando a
  versão `1.53.0-nightly` (2021-03-24) do `rustc` e ferramentas. No entanto, o
  código principal não utiliza nenhuma feature instável, portanto um toolchain
  `stable` mais novo ou igual ao `1.51` é suficiente. Instalação e atualização do
  toolchain pode ser feita com facilidade com o
  [rustup.rs](https://rustup.rs).
- *(extra)* Toolchain GCC MIPS: para compilar alguns experimentos que fiz,
  será necessário um toolchain do GCC configurado para a arquitetura MIPS.
  No Artix Linux, uma distribuição baseada no Arch Linux, eu utilizo o pacote
  `cross-mipsel-linux-gnu-gcc` do AUR.

## Executando

Para desmontar um binário, mostrando seu código Assembly equivalente:

```sh
$ cargo run -- decode file
```

Onde `file` é o prefixo dos arquivos em questão (por ex, `02.hello`).

Para executar o binário, utilize:

```sh
$ cargo run --release -- run [conf] file
```

Para executar o binário, gravando os acessos à memória num arquivo, utilize:

```sh
$ cargo run --release -- trace [--outfile out] [conf] file
```

Para executar o binário, gravando os acessos à memória e mais informaçõs num
arquivo, utilize:

```sh
$ cargo run --release -- debug [--outfile out] [conf] file
```

Todos os comandos suportam a escrita de informações de debug na saída padrão.
Para tal, basta setar a variável de ambiente `RUST_LOG=debug`.

## Documentação

O código inteiro foi documentado utilizando o `rustdoc`, nativo da linguagem.
Para ler a documentação em formato HTML (recomendo!), execute o comando:

```sh
$ cargo doc
```

A documentação estará disponível no arquivo `target/doc/indy_rs/index.html`.

## Extras

Como comentei acima, estou utilizando o projeto para experimentos. Nessa
seção, que atualizarei conforme novos experimentos forem surgindo, irei
descrevê-los brevemente.

### Changelog

Além do relatório, existe um `relatorio/changelog.fase2.pdf`. Nele, descrevo
com mais detalhe do que no relatório as mudanças feitas no código entre a fase
I e a fase II.

### Suporte a arquivos ELF

Por meio da crate `goblin`, o emulador consegue carregar e executar programas
contidos em arquivos `elf`, respeitando os endereços das seções (TODO) e o
ponto de entrada da aplicação.

A principal motivação para isso foi rodar programas escritos em C no emulador.
Utilizando a flag `-c` no GCC, junto do linker script que escrevi
(`res/extra/linker.ld`), o emulador consegue executar o objeto resultante da
compilação. Veja o arquivo `res/extra/Makefile` para entender o processo.

Para executar um arquivo ELF, basta usar o subcomando `runelf`. Exemplo:

```sh
cargo run -- runelf [conf] file
```

Para desmontá-lo, use o subcomando `decodeelf`. Ele irá desconstruir todas as
seções que estão marcadas com a flag `ALLOC`, ou seja, as que serão carregadas
na memória. Exemplo:

```sh
cargo run -- decodeelf file
```

Outro TODO grande é rodar código Rust MIPS!

### Framebuffer (TODO)

Essa ideia é um pouco mais complexa e ainda não foi iniciada. Pretendo fazer
um framebuffer em memória para brincar com uns gráficos. 

### Eu (não) tenho muito tempo livre (TODO)

- Does it run Doom?
- Does it run \*\*\*\*ing Linux???
    - Minha intuição diz que não é tão difícil. Tenho experiencia escrevendo
      umas device trees, e já compilei uma vmlinux minimalista para MIPS. Só
      falta realmente implementar um driver no Kernel para simular serial
      usando syscalls, escrever a device tree que descreve o hardware emulado
      e carregar tudo direito na memória. Divertido!
    - **UPDATE**: o ELF loader do projeto consegue tranquilamente carregar o
      Kernel, e o `decodeelf` já desconstrói ele por inteiro. Para considerar
      conseguir rodar Linux, é preciso implementar no mínimo exceções (o
      coprocessador 0 em específico).
- Does it run *Spyro: Year Of The Dragon*?
    - Esse emulador, durante o próximo recesso, com certeza virará um emulador
      de PlayStation 1!
