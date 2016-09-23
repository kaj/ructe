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
format can be seen in
[examples/simple/templates](examples/simple/templates).
