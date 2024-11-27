# Lexpr

## Features

1. Allow defining functions that resembles the natural language (most commonly English).
2. Rules decrease the usage of parenthesis.

## Rules (sorted by precedence descendingly):

1. Any sequence of alphanumeric identifiers NOT separated by delimiters (dot, colon, comma, brackets) becomes ONE identifier with words joined by hyphens
2. Symbolic identifiers (that are not dot, colon, comma or brackets) are also separated by space or atomic expressions
3. Dot is similar to dot operator in Javascript, it's used for left-associative chaining
4. Colon has lower precedence than dot, it's used for right-associative chaining, similar to Haskell `$`
5. Comma is similar to semicolon in Javascript, it has the lowest precedence, used for separating expressions
6. Parentheses create atomic expressions that prevent identifier merging

## Example translation (left Lexpr, right Sexpr):

- `hello world` = `(hello-world)` # hello and world merge into one identifier
- `f 123` = `(f 123)` # 123 is atomic, prevents merging
- `123 f` = `(f 123)`
- `plus 2 3` = `(plus 2 3)` # numbers are atomic
- `2 plus 3` = `(plus 2 3)`
- `f x y` = `(f-x-y)` # f, x, and y merge into one identifier
- `(f) x y` = `(f x-y)` # parentheses prevent f from merging, but x y still merge
- `(f) (x) (y)` = `(f x y)` # all merging prevented by parentheses
- `x. f y` = `(f-y x)` # dot operator prevents x from merging, but f y merge
- `x. f (y)` = `(f x y)` # dot operator and parentheses prevent all merging
- `f: x y` = `(f x-y)` # colon prevents merging with f, but x y still merge
- `f: (x) (y)` = `(f x y)` # colon and parentheses prevent all merging

## Example of a function with two arguments (all producing `(greater-than x y)`):

- `(x) greater than (y)` # using parentheses for both args
- `x. greater than (y)` # using dot for first arg, parentheses for second
- `(x) greater than: y` # using parentheses for first arg, colon for second
- `x. greater than: y` # using dot for first arg, colon for second

## Example code (remember Lexpr is only a data-format):

```
def (n. fib):
  if (n = 0)
  then 0
  else: if (n = 1)
    then 1
    else: n - 1. fib. +: n - 2. fib
```

```
def (n !):
  if (n = 0)
  then 1
  else: n *: (n - 1) !
```
