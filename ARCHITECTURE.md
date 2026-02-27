# Architecture

The Zinc interpreter is divided into a frontend and a backend.
The frontend and backend are decoupled. The backend can be swapped out, for example to a compiler to machine code.

# Frontend

The frontend is responsible for parsing, validating and lowering source code into an intermediate representation used by
the backend.

## Lexer

Converts source code into a stream of tokens.

## Parser

Converts a stream of tokens into a concrete syntax tree.

The concrete syntax tree is a node tree. It is a one-to-one representation of the source code.

During parsing, the source code is checked for syntax errors.
The parser also collects symbols declared in the source code.

## Lowerer

Converts the concrete syntax tree into an abstract syntax tree.

The abstract syntax tree is represented by a structure for each element in the source code.
The abstract syntax tree is a representation of source code's semantics.

During lowering, the source code is checked for semantic errors.

# Backend

The backend is responsible for interpreting the code.