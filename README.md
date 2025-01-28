# Rust lsp example

Exemplary lsp implementation for a small lisp like language.

```lisp
;; ---- atoms ----
102910
3.1415 
"Hello World"
"
Hello World
Hello World
Hello World
Hello World
"

;; ---- variables ----
(:pi 3.1415)
(:tau 
    (* 2 pi))
(:hello_world "Hello World")

;; ---- operations ----
(+ pi 12345)
(+ hello_world hello_world)
(- 1 85)
(/ 128 2)
(* 5 10)
(+ 25 
   (- 25 
      (/ 25 
         (* 25 25))))
```

The project contains the following modules:

| module | description                                               |
| ------ | --------------------------------------------------------- |
| lexer  | convert byte stream to token stream                       |
| parser | create abstract syntax tree from token stream             |
| cli    | invoke lexer and parser from the command line             |
| lsp    | provides diagnostics and hover for the lisp like language |

## Installation

```shell
cargo build --release
mv target/release/rust-lsp-example /usr/local/bin/rust-lsp-example
```

## Attaching language server to neovim

```lua
vim.lsp.config['rust-lsp-example'] = {
    cmd = { '/usr/local/bin/rust-lsp-example' },
    filetypes = { "lisp" },
}
vim.lsp.enable('rust-lsp-example')
```
