# Resources

Zaplib was designed to be familiar to web programmers. 

We avoid many advanced Rust concepts (Traits, Generics, Macros, Async), so the core syntax is easy to pick up. This is particularly true if you're coming from TypeScript.

## Ownership & Borrowing

One major syntactic difference you'll notice is `&mut` type annotations in function parameters. It indicates a *mutable reference*. In JavaScript, all parameters are mutable references, so these semantics should feel familiar: all changes to mutable references are made on the original. 

Unlike in JavaScript, Rust has an ownership model that allows it to:

* not have a garbage collector
* automatically allocate and free memory
* guarantee that references are valid (no null pointers, no 'use after free)
* guarantee that all references are thread-safe

In order to pull this off, Rust keeps careful track of which piece of your code has *ownership* of a particular value. 
When you pass a mutable reference to a function, we say that function is *borrowing* that value, and Rust's *borrow checker* guarantees that no other piece of code can access it until the borrower passes the ownership.

If you are new to Rust, you can get through the initial tutorials while ignoring the `& mut` type annotations. However, you will eventually run into the borrow checker, and at that point we recommend reading at least these articles on Rust's [ownership model](https://doc.rust-lang.org/book/ch04-01-what-is-ownership.html) and [borrowing](https://doc.rust-lang.org/book/ch04-02-references-and-borrowing.html).


## Resources

- [The Rust Book](https://doc.rust-lang.org/book)
- [Rustlings](https://github.com/rust-lang/rustlings)
- [How to get into Rust as a TypeScript Developer](https://www.thisdot.co/blog/how-to-get-into-rust-as-a-typescript-developer)

