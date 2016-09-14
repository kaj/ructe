# Rust Compile Templates -- ructe

This is my attempt at writing a HTML template system for Rust.
Some inspiration comes from the scala template system used in play 2.

Design criteria:

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
