# Zinc

Zinc is a hobby programming language roughly inspired by a combination of Java and Rust.

The scope of this document is to plan for how the language should behave. It is not a goal to determine the syntax
of the language. It is also not a goal to delve into implementation-specific details, such as memory layout.

A common thread throughout the design is uniformity. An eventual compiler or interpreter should need to implement as few
edges cases or builtin methods as possible.

As a result, much of the language is extendable. One should be able to, in code, replicate most if not all language
features.

## Objects

Everything in Zinc is an object. This includes the original source code the intermediate steps during compilation.
This automatically provides powerful tools for reflection. If everything is an object, we can call methods on
everything, as long as we are able to retrieve the object.

An object has a type and a value for each field in its type.

## Types

A type is also an object of type `Type`.

Types are used to verify type safety. If type `V` is assignable to type `F` it means that a value of type `V` can be
assigned to a field of type `F`.

A type is by design not equivelent to a `Class`. A type can be thought of as an instance of a class, in the same
way an object is an instance of a type. An object stores a type along with values representing its fields. A type stores
a class along with values representing its constant fields.

A type based on a class verifies type safety using its class and its constant fields. If another type is of the same
class or of a superclass, and all constant types are themselves assignable, the type is assignable.

However, there exists a variant of a type without any reference to a class. An anonymous type. An anonymous type defines
constraints that must exist for another type to be assignable to it. This is similar to how an interface is defined,
however, it does not require that the type inherits any specific class, only that it somewhere defines a set of specific
methods.

Both objects and types must be created from something else, whether it be a type or a class.

## Class

A class is an object of type `Class`.

A class is declared in code. It can define inner elements: classes, methods, and fields.

A class can define constant fields (see generics in other languages). They are referred to as constant fields because
they are only required to be constant, that is known at compile-time. They can have any type. One could define a
constant field to determine the maximum capacity of an array.

An important question that arrises from constant fields being able to store any object: what happens if the object
is modified?

To resolve this problem, we would need to be able to store whether a field is mutable. A mutable reference to a constant
field can, if so, never be retrieved. Any modification of an object requires a mutable reference. This is similar to
Rust memory safety but without the requirement that only a single mutable reference exists at the same time.

One could also argue that one should only store immutable types in constant fields. In what situation would you store
a mutable type in a constant field?

## Method

A method is an object type `Method`.

A method is declared in code. It can define parameters and a code block.

A method can also abstain from having a code block, in which case it is considered an abstract method. Its parent class,
as a result, also becomes an abstract class.

A method can be static or an instance method. An instance method defines an argument `self` without any type.

An instance method can be called either with a qualifying object or as a static method with the qualifying object as the
first argument.

A method can be defined to be constant. If so, the method can only reference constant fields and methods. 

### Constructors

Constructors are special methods declared in a class. They should define an argument `self`.

* For a non-abstract class (a concrete class) the constructor is called as though a static method. A new object is
  created automatically and given to the constructor.
* For a concrete class the constructor can be called from a subclass as though an instance method on the `self` object
  cast to this class.
* For an abstract class the constructor is called as though an instance method on an object cast to this class. The
  object cannot be called as though a static method, it must be given a `self` argument.

## Syntax

A class, defined using the `class` keyword, consists of a name, an optional list of parents, and a list of child
elements.

A function, defined using the `function` keyword, consists of a name, a list of parameters, and a code block.

A class can define a constructor, a special function defined using the `constructor` keyword. The constructor must
define a `self` parameter with type assignable to the parent class.

A variable, defined using the `let` keyword, has a type and an initializer. The initializer must be assignable to the
variable type. The initializer can be omitted if the variable belongs to a class, in which case all constructors must
initialize the variable *before* it can be used.

A constant, defined using the `const` keyword, defines a constant variable. A constant must be assigned another
constant, either a literal, another constant variable, or the result of a constant function call.

A constant function, defined using the `const function` keyword, defines a constant function. A constant function
cannot reference any non-constant variable or function.

```zinc
class Person {
    let name: String;
    
    // A constructor is called just any static function.
    // The *self* argument is automatically created and passed to the constructor.
    // let person: Person = Person::new("John");
    constructor new(self: Person, name: String) -> Person {
        self.name = name;
    }
 
    // A function can be called on a specific object or as a static function.
    // let name: String = Person::get_name(person);
    // *or*
    // let name: String = person.get_name();  
    function get_name(self: Person) -> String {
        return self.name;
    }
}
```

### Class

```bnf
class := "class" identifier constants? extends? body

constants := "<" constant_expression ("," constant_expression)* ">"

extends := ":" type ("," type)*

body := "{" statement* "}"
```

A class cannot extend itself, either directly or indirectly.

A class is considered abstract if it has at least one abstract method.

A class can extend the same class multiple times indirectly. Each such superclass is kept separate.

A class cannot directly extend the same class multiple times, if so, they must have different constant types.

### Function

```bnf
function := "const"? function" identifier arguments "->" type (body | ";")

arguments := "(" argument ("," argument)* ")"

argument := "const"? identifier ":" type

body := "{" statement* "}"
```

A function is considered abstract if it has a body.

A constant function cannot reference any non-constant variable or function.

### Field

```bnf
field := "const"? identifier ":" type "=" expression ";"
```

A constant field cannot reference any non-constant variable or function.

## Typing

Zinc is a statically typed language, as such all variables must be explicitly typed. It is a non-goal to support type
inference.

That said, Zinc supports generic variables:

```zinc
class List<T: Class> {
    // We can use constants in type expressions.
    // They must subclass Class.
    let elements: T[];
    
    constructor new() -> List<T> {
        // We must initialize the array *after* assigning T.
        self.elements = [];
    }
    
    function add(self: List<T>, value: T) -> List<T> {
        return List::new(T);
    }
}
```

### Inheritance

Zinc supports inheritance by inheriting from a parent class.

```zinc
class Animal {
    let name: String;
    
    // You can define constructors in abstract classes (classes with abstract methods).
    // However, they cannot be called directly.
    constructor new(name: String) -> Animal {
        self.name = name;
    }

    // This method is abstract: it doesn't define a body.
    // It must be implemented by a child class.
    function speak(self: Animal) -> String;
}

class Dog : Animal {
    constructor new(name: String) -> Dog {
        // We call the constructor of the parent class.
        // By passing this instance, Animal will instantiate this object and not create a new Animal. 
        Animal::new(self, name);
    }

    // This function is implemented by the Dog class.
    function speak(self: Dog) -> String {
        // If the superclass function was declared, we could call it using Animal::speak(self).
        // We can access fields from the superclass. 
        // If multiple superclasses have the same field, or if this class also declares a field with the same name,
        // we need to cast self to the desired supertype, for example (self as Animal).name.
        // If we extend the same superclass multiple times, we need to cast in multiple steps:
        // ((self as Animal) as Living).isLiving
        // *or*
        // ((self as Mammal) as Living).isLiving
        // If a class extends the same superclass multiple times, they are kept separated.
        return self.name + ": Woof!";
    }
}
```

Zinc supports operator overloading by inheriting specific classes. All operators in Zinc are overloaded.

Objects do not by default implement any operator.

For example, objects do not implement the equality operator. By implementing the `Identity` class, an object
receives a unique identity which lets us compare objects for equality - which will check if they are the exact same
object.

# Reflection

Zinc supports reflection at runtime. All classes, functions, and variables can be inspected as objects of type `Class`,
`Function`, or `Variable`. Likewise, these elements can be created at runtime and added to existing classes.

We want to be able to access elements as objects at runtime.

We can access a class or method by referencing the element. However, by referencing a field we are actually referencing
its value.

Fields can reference their field object, similarly to methods, and thereby have a method `Field::get()` which would
return the field's value.

An alternative is to add a special method or syntax to retrieve the field instead of its value. However, this would
require special support from the compiler.

An alternative is to find the field object from its parent. `Class::find_field(name: String)` would return the field
object with that name. However, this would use the name of the field when we might already have a reference to the
field. A method `Field::as_field(_)` could be used, it would need to be handled specially so that we don't give it the
value of the field but the field object itself.

Alternatively, we could cast the field value to `Field`, which would give the field. If the field's value is itself a
`Field`, we would still get the field's value. This would be akin to having the value of the field be a `Proxy`, it
would appear to be a value to everyone else, but is actually a `Field`.

To make this alternative more precise, we can add an interface `Dereference`, similar to Rust. A class which implements
this type can be used as both its own class and as the class it dereferences to. A class can only implement `Derefernce`
once.
