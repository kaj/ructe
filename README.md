# Rust Compiled Templates — ructe

This is my attempt at writing a HTML template system for Rust.
Some inspiration comes from the scala template system used in play 2,
as well as plain old jsp.

[![Build Status](https://travis-ci.org/kaj/ructe.svg?branch=master)](https://travis-ci.org/kaj/ructe)
[![Crate](https://meritbadge.herokuapp.com/ructe)](https://crates.io/crates/ructe)
[![docs](https://docs.rs/ructe/badge.svg)](https://docs.rs/ructe)

## Design criteria

* As many errors as possible should be caught at compile-time.
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

Ructes is in a rather early stage, but does work;
templates can be transpiled to rust functions, which are then compiled
and can be called from rust code.

### Template format

A template consists of three basic parts:
First a preamble of `use` statements, each prepended by an @ sign.
Secondly a declaration of the parameters the template takes.
And third, the template body.

The full syntax is described [in the
documentation](https://docs.rs/ructe/~0.3/ructe/Template_syntax/index.html).
Some examples can be seen in
[examples/simple/templates](examples/simple/templates).
A template may look something like this:

```
@use any::rust::Type;
@use templates::statics::style_css;

@(name: &str, items: &[Type])

<html>
   <head>
     <title>@name</title>
     <link rel="stylesheet" href="/static/@style_css.name" type="text/css"/>
   </head>
   <body>
     <h1>@name</h1>
     <dl>
     @for item in items {
       <dt>@item.title()
       <dd>@item.description()
     }
     </dl>
   <body>
</html>
```

## How to use ructe

Ructe compiles your templates to rust code that should be compiled with
your other rust code, so it needs to be called before compiling,
as described [in "How to use ructe", in the
documentation](https://docs.rs/ructe/~0.3/ructe/How_to_use_ructe/index.html).
There are also [examples](examples).

When I use ructe with [nickel](https://crates.io/crates/nickel), I use a
rendering function that looks like this:

```rust
fn render<'mw, F>(res: Response<'mw>, do_render: F)
                  ->MiddlewareResult<'mw>
    where F: FnOnce(&mut Write) -> io::Result<()>
{
    let mut stream = res.start()?;
    match do_render(&mut stream) {
        Ok(()) => Ok(Halt(stream)),
        Err(e) => stream.bail(format!("Problem rendering template: {:?}", e))
    }
}
```

Which I call like this:

```rust
render(res, |o| templates::foo(o, other, arguments))
```
