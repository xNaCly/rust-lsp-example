# Rust lsp example

Exemplary lsp implementation for a small lisp like language.

```lisp
;; atoms
102910 ;; number
3.1415 ;; also number
"Hello World" ;; string

;; artihmetics operations
(+ 3.1415 12345)
(+ "Hello" "World")
(- 1 85)
(/ 128 2)
(* 5 10)

;; nested
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
