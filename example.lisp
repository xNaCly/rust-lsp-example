; example file, excute with:
; cargo run example.lisp

; ---- atoms ----
102910
3.1415 
"Hello World"
"
    Hello World
    Hello World
    Hello World
    Hello World
"

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

; --- lambdas ---
; no op
; (lambda (num) num) 
; ; immediate invocation
; ((lambda (num) num) "hello") 
; 
; ; assign lambda to a variable
;  (let return_name 
;     (lambda (name) (name)))
; ; call lambda
; (return_name 5) 
