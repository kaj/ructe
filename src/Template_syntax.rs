// This module is only a chapter of the documentation.
//! This module describes the template syntax used by ructe.
//!
//! The syntax is inspired by
//! [Twirl](https://github.com/playframework/twirl), the Scala-based
//! template engine in
//! [Play framework](https://www.playframework.com/),
//! but of course with rust types expressions instead of scala.
//!
//! A template consists of three basic parts:
//! First a preamble of `use` statements, each prepended by an @ sign.
//! Secondly a declaration of the parameters the template takes.
//! And third, the template body.
//!
//! ```text
//! @use any::rust::Type;
//!
//! @(name: &str, items: &[Type])
//!
//! <html>
//!    ...
//! </html>
//! ```
//!
//! The curly brackets, `{` and `}`, is used for blocks (see Loops,
//! Conditionals, and Calling other templates below).
//! To use them in the template body, they must be escaped as `@{` and
//! `@}`.
#![allow(non_snake_case)]

pub mod a_Value_expressions {
    //! A value expression can be as simple as `@name` to get the value of
    //! a parameter, but more complicated expressions, including function
    //! calls, are also allowed.
    //!
    //! # Value expressions
    //!
    //! A parameter can be used in an expression preceded by an @ sign.
    //!
    //! ```text
    //! <h1>@name</h1>
    //! ```
    //!
    //! If a parameter is a struct or a trait object, its fields or methods can
    //! be used, and if it is a callable, it can be called.
    //!
    //! ```text
    //! <p>The user @user.name has email @user.get_email().</p>
    //! <p>A function result is @function(with, three, arguments).</p>
    //! ```
    //!
    //! Standard function and macros can also be used, e.g. for specific
    //! formatting needs:
    //!
    //! ```text
    //! <p>The value is @format!("{:.1}", float_value).</p>
    //! ```
}

pub mod b_Loops {
    //! A ructe `@for` loop works works just as a rust `for` loop,
    //! iterating over anything that implements `std::iter::IntoIterator`,
    //! such as a `Vec` or a slice.
    //!
    //! # Loops
    //!
    //! Rust-like loops are supported like this:
    //!
    //! ```text
    //! <ul>@for item in items {
    //!   <li>@item</li>
    //! }</ul>
    //! ```
    //!
    //! Note that the thing to loop over (items, in the example) is a rust
    //! expression, while the contents of the block is template code.
    //!
    //! If items is a slice of tuples (or really, anything that is
    //! iterable yielding tuples), it is possible to deconstruct the
    //! tuples into separate values directly:
    //!
    //! ```text
    //! @for (n, item) in items.iter().enumerate() {
    //!     <p>@n: @item</p>
    //! }
    //! ```
}

pub mod c_Conditionals {
    //! Both `@if` statements with boolean expressions and match-like
    //! guard `@if let` statements are supported.
    //!
    //! # Conditionals
    //!
    //! Rust-like conditionals are supported in a style similar to the loops:
    //!
    //! ```text
    //! @if items.is_empty() {
    //!   <p>There are no items.</p>
    //! }
    //! ```
    //!
    //! Pattern matching let expressions are also supported, as well as an
    //! optional else part.
    //!
    //! ```text
    //! @if let Some(foo) = foo {
    //!   <p>Foo is @foo.</p>
    //! } else {
    //!   <p>There is no foo.</p>
    //! }
    //! ```
    //!
    //! General rust `match` statements are _not_ supported in ructe
    //! (at least not yet).
}

pub mod d_Calling_other_templates {
    //! The ability to call other templates for from a template makes
    //! both "tag libraries" and "base templates" possible with the
    //! same syntax.
    //!
    //! # Calling other templates
    //!
    //! While rust methods can be called as a simple expression, there is a
    //! special syntax for calling other templates:
    //! `@:template_name(template_arguments)`.
    //! Also, before calling a template, it has to be imported by a `use`
    //! statement.
    //! Templates are declared in a `templates` module.
    //!
    //! So, given something like this in `header.rs.html`:
    //!
    //! ```text
    //! @(title: &str)
    //!
    //! <head>
    //!   <title>@title</title>
    //!   <link rel="stylesheet" href="/my/style.css" type="text/css">
    //! </head>
    //! ```
    //!
    //! It can be used like this:
    //!
    //! ```text
    //! @use templates::header;
    //!
    //! @()
    //!
    //! <html>
    //!   @:header("Example")
    //!   <body>
    //!     <h1>Example</h1>
    //!     <p>page content ...</p>
    //!   </body>
    //! </html>
    //! ```
    //!
    //! It is also possible to send template blocks as parameters to templates.
    //! A structure similar to the above can be created by having something like
    //! this in `base_page.rs.html`:
    //!
    //! ```text
    //! @(title: &str, body: Content)
    //!
    //! <html>
    //!   <head>
    //!     <title>@title</title>
    //!     <link rel="stylesheet" href="/my/style.css" type="text/css">
    //!   </head>
    //!   <body>
    //!     <h1>@title</h1>
    //!     @:body()
    //!   </body>
    //! </html>
    //! ```
    //!
    //! And use it like this:
    //!
    //! ```text
    //! @use templates::base_page;
    //!
    //! @()
    //!
    //! @:base_page("Example", {
    //!     <p>page content ...</p>
    //! })
    //! ```
}
#[cfg(doc_test)]
pub fn test_template(code: &str) {
    // ok
}
