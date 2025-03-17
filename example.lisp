; example file, excute with:
; cargo run example.lisp

; ---- atoms ----
102910
3.1415 

; ---- variables ----
(let pi 3.1415)
(let hello_world "Hello World")
(let fac ("" 1 2 6 24 120 720 5040 40320))

(pi hello_world)
fac
((let empty_list ()))


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
