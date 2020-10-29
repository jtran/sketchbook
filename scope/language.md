# The Scope Lanugage

In the same way that JSON is the lowest common denominator across languages for
representing a data structure, at a point in time, the scope language uses
simple operations that are common across languages for representing state change
over time.

Here are some of the operations.  This list is experimental and subject to change.

- set x = \<constant\>
- set x = \<constant\> from complex_function(y, z)
- copy x into y
- copy x into array[2]
- copy x into map["color"]
- copy array[3] into map["name"]
- push array \<constant\>
- pop array
- delete key map \<key_name\>

From this, and a basic understanding of common data structures like arrays,
maps, trees, and references, many runtime states can be displayed.  Morever,
there is a straightforward visual transition between the representation before
the operation and the representation after the operation.

### What about Computation?

You may be wondering, where are all the computation operations, like plus and
minus.  Surely a language must at least be able to do arithmetic.

In the current formulation of the scope language, no computation is done, only
state mutation.  The computation is done in the host language which produces the
state change operations in the scope language.

In the same way that you can represent any values in JSON, but you cannot
compute those values in JSON, the same is true of the scope language.

The result is that the scope language can represent state and state changes of
any runtime system, regardless of its runtime semantics.  To give a concrete
example, the scope doesn't need to know the subtle rules for adding numbers with
a fixed precision and overflow in your language.  Presumably, you are computing
the result anyway in your runtime, so instead of the scope trying to replicate
that those rules, the runtime just embeds the result into the operations in the
scope language.

This is the reason that most of the time, arbitrary expressions aren't needed in
scope operations.  Any complex computation will result in a `set x = <constant>`
operation.  To track what values were input into the computation, you can use
the more advanced `set x = <constant> from complex_function(y, z)`.  In this
case, `complex_function` is just a label.  The scope isn't aware of its
implementation.
