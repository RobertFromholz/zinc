# Zinc

Zinc is aimed to compile a subset of Rust.

This specification aims to clarify what is supported and what is unsupported.

Type inference, generics and lifetimes are not supported.

# Modules

Modules in separate files are supported. This includes both `foo.rs` and `foo/mod.rs`.

Nested modules (`mod foo { ... }`) are not supported.

# Functions

Functions are supported.

Generic arguments are not supported. Lifetime arguments are not supported – all arguments take ownership of the
variable.

# Structs

Structs are supported.

Generic arguments or lifetime arguments are not supported.

Struct implementations are not supported.

# Traits

Traits are not supported.