# Rust Compiled Templates — ructe

This is my attempt at writing a HTML template system for Rust.
Some inspiration comes from the scala template system used in play 2,
as well as plain old jsp.

[![Build Status](https://travis-ci.org/kaj/ructe.svg?branch=master)](https://travis-ci.org/kaj/ructe)

## Design criteria

* As many errors as possible should be caught in compile-time.
* A compiled binary should include all the template code it needs,
  no need to read template files at runtime.
* Compilation may take time, running should be fast.
* Writing templates should be almost as easy as writing html.
* The template language should be as expressive as possible.
* It should be possible to write templates for any text-like format,
  not only html.
* Any value that implements the `Display` trait should be outputable.
* By default, all values should be html-escaped.  There should be an
  easy but explicit way to output preformatted html.

## Current status

This is currently more of a proof of concept that anyting ready for
actual production use.
That said, it actually does work; templates can be transpiled to rust
functions, which are then compiled and can be called from rust code.
The template syntax is not stable yet, but some examples in the current
format can be seen below, and in
[examples/simple/templates](examples/simple/templates).

### Template format

A template consists of three basic parts:
First a preamle of `use` statements, each prepended by an @ sign.
Secondly a declaration of the parameters the template takes.
And third, the template body.

```
@use any::rust::Type;

@(name: &str, items: Vec<Type>)

<html>
   ...
</html>
```

### Value expressions

A parameter can be used in an expression preceded by an @ sign.

```
<h1>@name</h1>
```

If a parameter is a struct or a trait object, its fields or methods can
be used, and if it is a callable, it can be called.

```
<p>The user @user.name has email @user.email.</p>
```

Standard function and macros can also be used, e.g. for specific
formatting needs:

```
<p>The value is @format!("{:.1}", float_value).</p>
```

### Loops

Rust-like loops are supported like this:

```
<ul>@for item in items {
  <li>@item</li>
}</ul>
```

Note that the thing to loop over (items, in the example) is a rust
expression, while the contents of the block is template code.

### Conditionals

Rust-like conditionals are supported in a style similar to the loops:

```
@if items.is_empty() {
  <p>There are no items.</p>
}
```

Pattern matching let expressions are also supported, as well as an
optional else part.

```
@if let Some(foo) = foo {
  <p>Foo is @foo.</p>
} else {
  <p>There is no foo.</p>
}
```

### Calling other templates

While rust methods can be called as a simple expression, there is a
special syntax for calling other templates:
`@:template_name(template_arguments)`.
Also, before calling a template, it has to be imported by a `use`
statement.
Templates are declared in a `templates` module.

So, given something like this in `header.rs.html`:

```
@(title: &str)

<head>
  <title>@title</title>
  <link rel="stylesheet" href="/my/style.css" type="text/css">
</head>
```

It can be used like this:

```
@use templates::header`

@()

<html>
  @:header("Example")
  <body>
    <h1>Example</h1>
    ...
  </body>
</html>
```

## How to use ructe

Ructe compiles your templates to rust code that should be compiled with
your other rust code, so it needs to be called before compiling.
Assuming you use [cargo](http://doc.crates.io/), it can be done like
this:
First, specify a build script and ructe as a build dependency in
`Cargo.toml`:

```toml
build = "src/build.rs"

[build-dependencies]
ructe = { git = "https://github.com/kaj/ructe" }
```

Then, in the build script, compile all templates found in the templates
directory and put the output where cargo tells it to:

```rust
extern crate ructe;

use ructe::compile_templates;
use std::env;
use std::path::PathBuf;

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let in_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap())
        .join("templates");
    compile_templates(&in_dir, &out_dir).expect("foo");
}
```

And finally, include and use the generated code in your code.
The file `templates.rs` will contain `mod templates { ... }`,
so I just include it in my `main.rs`:

```rust
include!(concat!(env!("OUT_DIR"), "/templates.rs"));
```

When calling a template, the arguments declared in the template will be
prepended by an argument that is the `std::io::Write` to write the
output to.
It can be a `Vec<u8>` as a buffer or for testing, or an actual output
destination.
The return value of a template is `std::io::Result<()>`, which should be
`Ok(())` unless writing to the destination fails.

```
#[test]
fn test_hello() {
    let mut buf = Vec::new();
    templates::hello(&mut buf, "World").unwrap();
    assert_eq!(from_utf8(&buf).unwrap(), "<h1>Hello World!</h1>\n");
}
```

When I use ructe with [nickel](https://crates.io/crates/nickel), I use a
rendering function that looks like this:

```
fn render<'mw, F>(res: Response<'mw>, do_render: F)
                  ->MiddlewareResult<'mw>
    where F: FnOnce(&mut Write) -> io::Result<()>
{
    let mut stream = try!(res.start());
    match do_render(&mut stream) {
        Ok(()) => Ok(Halt(stream)),
        Err(e) => stream.bail(format!("Problem rendering template: {:?}", e))
    }
}
```

Which I call like this:

```
render(res, |o| templates::foo(o, other, arguments))
```
