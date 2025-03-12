# Rust lsp example

Exemplary lsp implementation for a small lisp like language.

```lisp
; example file, excute with:
; cargo run example.lisp

; ---- atoms ----
102910
3.1415
"Hello World"

; ---- variables ----
(let pi 3.1415)
(let hello_world "Hello World")
(let fac (0 1 2 6 24 120 720 5040 40320))

pi
hello_world
fac

; ---- lists ----
(1 85 1201 (128 2))
(5 10)
(pi 12345 hello_world)
(25
   (25
      (25
         (25 25))))
```

Evaluates to:

```text
[000]: 102910
[001]: 3.1415
[002]: 3.1415
[003]: `Hello World`
[004]: (0, 1, 2, 6, 24, 120, 720, 5040, 40320)
[005]: (1, 85, 1201, (128, ``))
[006]: (5, 10)
[007]: (3.1415, 12345, `Hello World`)
[008]: (25, (25, (25, (25, 25))))
```

## Installation

```shell
cargo build --release
mv target/release/rust-lsp-example /usr/local/bin/rust-lsp-example
```

## Attaching language server to neovim

```lua
vim.lsp.config['rust-lsp-example'] = {
    cmd = { '/usr/local/bin/rust-lsp-example', '--lsp' },
    filetypes = { "lisp" },
}
vim.lsp.enable('rust-lsp-example')
```

## Project structure

The project contains the following modules:

| module | description                                               |
| ------ | --------------------------------------------------------- |
| lexer  | convert byte stream to token stream                       |
| parser | create abstract syntax tree from token stream             |
| cli    | invoke lexer and parser from the command line             |
| lsp    | provides diagnostics and hover for the lisp like language |
