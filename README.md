# Rust lsp example

Exemplary lsp implementation for a small lisp like language.

```lisp
; example file, excute with:
; cargo run example.lisp

; ---- atoms ----
102910
3.1415 

; ---- variables ----
(let pi 3.1415)
(let hello_world "Hello World")
(let fac ("" 1 2 6 24 120 720 5040 40320))
(let empty_list ())

(pi hello_world)
fac
empty_list


; ---- lists ----
(1 85 1201 (128 ""))
(5 10)
(pi 12345 hello_world)
(25 
   (25 
      (25 
         (25 25))))

; --- lambdas ---
; * lambdas are pure, no variables out of the lambdas scope can be mutated
; * or accessed.

; single argument vs multiple arguments
(lambda (num) num) (lambda (num num2) (num num2))

; immediate invocation
((lambda (num) num) "hello")

(
 ; assign lambda to a variable -> create a function
 (let return_n_n (lambda (n n1) (n n1))) 
 ; call lambda
 5 12
)
```

Evaluates to:

```text
[000]: 102910
[001]: 3.1415
[002]: 3.1415
[003]: `Hello World`
[004]: 9#(``, 1, 2, 6, 24, 120, 720, 5040, 40320)
[005]: 0#()
[006]: 2#(3.1415, `Hello World`)
[007]: 9#(``, 1, 2, 6, 24, 120, 720, 5040, 40320)
[008]: 0#()
[009]: 4#(1, 85, 1201, 2#(128, ``))
[010]: 2#(5, 10)
[011]: 3#(3.1415, 12345, `Hello World`)
[012]: 2#(25, 2#(25, 2#(25, 2#(25, 25))))
[013]: λ(α)
[014]: λ(α,α)
[015]: `hello`
[016]: 2#(5, 12)
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
