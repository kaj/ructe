# Changelog

All notable changes to this project will be documented in this file.

The format is based on
[Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this
project adheres to
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased

* Improve error reporting.  The debug output for `RucteError` is now the
  same as display, and the standard `Error::source` is implemented.
* Fix clippy lint clippy::get-first (PR #114).
* Update optional rsass to 0.25.0.

Thanks to @vbrandl for PR #114.


## Release 0.14.0 - 2022-02-06

* Breaking change: The generated template functions have a simpler
  signature.
* Allow litetimes in template argument types.  Issue #106, PR #110.
* Improve error handling in optional warp support, PR #109.
* Current stable rust is 1.57, MSRV is now 1.46.0.
* Update nom dependency to 7.1.0.
* Update optional rsass to 0.23.0.
* Update env_logger to 0.9 and gotham to 0.7.1 in examples
* Dropped support for warp 0.2 (the warp02 feature and example).

Thanks to @JojiiOfficial for reporting #106.


## Release 0.13.4 - 2021-06-25

* Allow `else if` after an `@if` block in templates. PR #104, fixes #81.
* Add a missing `}` in doc example.  PR #102.
* Update optional rsass to 0.22.0.
* Updated gotham example to 0.6.0.

Thanks @bearfrieze for #102 and @Aunmag for #81.

Tested with rustc 1.53.0, 1.48.0, 1.46.0, 1.44.1, 1.54.0-beta.1,
and 1.55.0-nightly (7c3872e6b 2021-06-24).


## Release 0.13.2 - 2021-03-14

* Improve formatting of README, PR #100.
* Update nom to 6.1.0, which raises the MSRV to 0.44
* Update base64 to 0.13 and itertools to 0.10.
* Update optional rsass to 0.19.0.
* Add warp 0.3 feature and example.
* Add tide 0.16 feaure and update example.
* Testing is now done with github actions rather than Travis CI.
* Minor clippy fixes, PR #99.

Thanks to @ibraheemdev for PR #100.

Tested with rustc 1.50.0 (cb75ad5db 2021-02-10),
1.48.0 (7eac88abb 2020-11-16),
1.46.0 (04488afe3 2020-08-24),
1.44.1 (c7087fe00 2020-06-17),
1.51.0-beta.6 (6a1835ad7 2021-03-12),
1.52.0-nightly (acca81892 2021-03-13)


## Release 0.13.0 - 2020-11-15

* Try to improve incremental compile times of projects using ructe by
  only writing fils if their contents actually changed. Also some code
  cleanup. PR #97.
* Update ructe itself to use edition 2018 (it is still useable for
  projects using both editios).  PR #98.
* Fix `StaticFiles::add_files_as` for empty `to` argument and add some
  more documentation for it.  Fixes issue #96.
* Update optional rsass dependency to 0.16.0.
* Add optional support for tide 0.14 and 0.15.
* Update gotham to 0.5 and axtix-web to 3.2 in examples.

Tested with rustc 1.47.0 (18bf6b4f0 2020-10-07),
1.42.0 (b8cedc004 2020-03-09), 1.40.0 (73528e339 2019-12-16),
1.48.0-beta.8 (121901459 2020-11-08), and
1.50.0-nightly (98d66340d 2020-11-14)


## Release 0.12.0 - 2020-08-14

* PR #80 and #94: Support Tide framework by a feature and an example.
* PR #91: Update basic examples to edition 2018.
* Issue #68, PR #90: Don't eat whitespace after a for loop.
* Issue #66, PR #89: Fix parse error for nested braces in expressions.
* PR #84: Use std::ascii::escape_default.
* PR #87: Provide ToHtml::to_buffer()
* Forbid unsafe and undocumented code.
* The build is on https://travis-ci.com/kaj/ructe now.
* Internal cleanup.

## Release 0.11.4 - 2020-04-25

* Improve `@match` parsing.

## Release 0.11.2 - 2020-04-22

* Bugfix: Allow space before laste brace in `@match`.

## Release 0.11.0 - 2020-04-21

* PR #73, Issue #38: Add support for `@match` statements.

Thanks to @vivlim for the issue.

## Release 0.10.0 - 2020-04-19

* Update rsass to 0.13.0 and improve sass error handling.
* Drop the warp01 feature.
* PR #72 from @kornelski: Avoid clobbering variable name.
* Update itertools to 0.9.0 and base64 to 0.12.0.

Thanks to @kornelski for suggestions and bug reports.


## Release 0.9.2 - 2020-01-25

* PR #70, Issue #63: Add feature warp02, supportig warp 0.2.x, and add
  a name alias warp01 for the old warp 0.1.x feature.  Same in
  examples.
* PR #69, Issue #67: Anyting that is allowed in a string in Rust
  should be allowed in a string in ructe.
* Fix clippy complaints re statics in generated code.
* Update actix-web example to 2.0.
* Fix doctest with mime03 feature.

Thanks to @nocduro and @Aunmag for suggestions and bug reports.


## Release 0.9.0 - 2019-12-25

* PR #65, Issue #64: An expression starting with paren ends on close.
  BREAKING: Before this change, calling a function on the result of
  some subexpression could be written as `@(a - b).abs()`.  After this
  change, that should be changed to `@((a - b).abs())` unless the
  intent is to have the result of (a - b) followed by the template
  string `.abs()`.
* RucteError now implements std::error::Error.
* Specify which references in examples are `dyn` or `impl`.
* Remove a useless string clone.
* Update rsass to 0.12.0.

Thanks to @Aunmag.


## Release 0.8.0 - 2019-11-06

* Issue #62: New version number due to a semver-breaking change,
  reported by @kornelski.

Otherwise same as 0.7.4:

* PR #55 from kornelski: Improve benchmarks.
* Part of issue #20: Allow template source files to be named *.rs.svg
  or *.rs.xml as well as *.rs.html.  The generated template functions
  will simlilarly be suffixed _svg, _xml or _html (any template_html
  will get a template alias, for backwards compatibility.
* PR #61 from Eroc33: Improve parsing for tuple and generic type
  expressions.
* Fix old doc link in readme.
* Update dependencies in ructe and examples.

Thaks to @kornelski and @Eroc33.


## Redacted: Relase 0.7.4 - 2019-11-02

* PR #55 from kornelski: Improve benchmarks.
* Part of issue #20: Allow template source files to be named
  `*.rs.svg` or `*.rs.xml` as well as `*.rs.html`.  The generated
  template functions will simlilarly be suffixed `_svg`, `_xml` or
  `_html` (any `template_html` will get a `template` alias, for
  backwards compatibility.
* PR #61 from @Eroc33: Improve parsing for tuple and generic type
  expressions.
* Fix old doc link in readme.
* Update dependencies in ructe and examples.

Thaks to @kornelski and @Eroc33.


## Release 0.7.2 - 2019-08-28

* Issue #53, PR #60: Allow empty strings everywhere quoted strings are
  allowed.
* Issue #57, PR #59: Accept explicit impl and dyn in types.
* Relax over-strict whitespace requirements, fix a regression in 0.7.0.
* PR #56: Require buf reference to implement Write, not buf itself
* PR #58: Fix warnings in generated code.
* Remove no-longer-used imports.

Thanks to @kornelski for multiple contributions.


## Release 0.7.0 - 2019-07-18

* PR #52: Upgrade nom to 5.0
* Update rsass to 0.11.0 (which also uses nom 5.0)
* Improve template declaration parsing and diagnostics.
* PR #50 and #51 from @dkotrada: Fix typos in actix example.
* Remove deprecated functions.


## Release 0.6.4 - 2019-06-23

* Added more modern rust compiler versions (and dropped 1.26).
* PR #49: Add an actix example.
* PR #48 from @Noughmad: Use `impl Write` or generic argument instead
  of dynamic traits. Fixes a warning for each template when using
  edition 2018 in nightly rust.
* Clearer doc about escaping special characters.
* PR #46 from @kornelski: Add missing crates keyword


## Release 0.6.2 - 2019-03-16

* Improved documentation and examples.
  All public items now have documentation.
* Improve build-time error handling.
  If there is an error involving an environment variable, include the
  variable name in the message.
* Take more Path build-time arguements AsRef.
  Make it possible to simpl use a string literal as a path in more places.


## Release 0.6.0 - 2019-03-14

* PR #45: Provide a warp feature.
  All my warp + ructe projects use the same RenderRucte extension trait
  to make calling the templates on generating responses a bit clearer.
  Provide that trait here as an optional feature.
* PR #43: Make the build scripts nicer.
  Provide a struct Ructe with methods to handle the red tape from build
  scripts.  Make the remaining parts of the build scripts shorter and
  more to the point.
* Use edition 2018 in warp example.
* Fix examples lang attribute.
  A whole bunch of examples had the html lang attibute set to sv when
  the content is actually in English.


## Release 0.5.10 - 2019-02-22

* Convert more file names to rust names (a file name might contain
  dashes and dots that needs to be converted to something else
  (underscore) to work in a rust name).
* Find new files in static dirs (add a cargo:rerun-if-changed line
  for the directory itself).


## Release 0.5.8 - 2019-02-16

* Adapt to rsass 0.9.8 (the sass feature now requires a compiler that
  supports edition 2018).
* More compact static data, using byte strings instead of numbers.
  (i.e. b"\xef\xbb\xbfabc" rather than [239, 187, 191, 65, 66, 67]).
* Minor internal cleanup.
* Update bytecount dependency.


## Release 0.5.6 - 2019-01-05

* PR #41: Benchmark and improve performance of html-escaping.
* PR #39: Silence a clippy warning about old syntax in silencing
  another warning.
* Update itertools to 0.8 (and env_logger in warp example)

Thanks to @kornelski for PRs #39 and #41.


## Release 0.5.4 - 2018-11-30

* Support struct unpacking in `@if` and `@for` expressions.


## Release 0.5.2 - 2018-11-04

* Special case for empty sub-templates, mainly to avoid a warning when
  compiling generated code.
* Update md5 to 0.6.
* Update gotham in example to 0.3.0.
* Use mime 0.3 in static example, and remove mime03 example.


## Release 0.5.0 - 2018-11-03

* Support multiple Content arguments.
  Impl Trait is used to make sub-templates as arguments less magic.
  This way we can also support more than one Content argument to the
  same template.
* PR #36 / Issue #35: Test and fix support for edition=2018.
  Module paths used by generated code are now compatible with the 2018
  edition.  Also, some code in examples and documentation use more
  2018-friendly module paths.
* PR 34: Use bytecount rather than simple counting, elide one lifetime.
* Update nom to 4.1.1, base64 to 0.10.0, bytecount to 0.4, and md5 to 0.5.
* Update iron to 0.6 and warp to 0.1.9 in examples.
* Minor cleanup in nickel example.

Thanks to @KlossPeter for PR #34 and @matthewpflueger for issue #35.


## Release 0.4.6 - 2018-10-07

* Lock nom version at 4.0, since it seems the 4.1 release is
  incompatible with the error handling in ructe.


## Release 0.4.4 - 2018-09-06

* Test and fix #33, unduplicate curly brackets.
* Add `@@` escape, producing a single `@` sign.  Suggested in #33.
* Some more mime types for static files.
* Update dependencies: nom 4.0, rsass 0.9.0
* Add a warp example, and link to kaj/warp-diesel-ructe-sample

Thanks to @dermetfan for reporting issue #33.


## Release 0.4.2 - 2018-08-01

* Test and fix issue #31, comments in body.

Thanks to @jo-so for reporting the issue, and for the test


## Release 0.4.0 - 2018-07-05

* Template syntax:
  - Allow local ranges (i.e. `2..7`) in loop expressions.
  - Allow underscore rust names.  There is use for unused variables in
    templates, so allow names starting with underscore.
  - Issue #24 / PR #28: Allow logic operators in `@if ...` expressions.
  - Issue #25 / PR #27: Allow much more in parentehsis expressions.

* Improved examples:
  - A new design for the framework examples web page, using svg graphics.
  - Improve code and inline documentation of iron and nickel examples.
  - Add a similar example with the Gotham framework.

* Recognize `.svg` static files.
* Allocate much fewer strings when parsing expressions in templates.
* PR #26: use `write_all` rather than the `write!` macro in generated
  code, contributed by @kornelski
* Fix `application/octet-stream` MIME type.  Contributed by @kornelski.
* Use `write_str`/`write_all` when generating output.  Contributed by
  @kornelski.


## Release 0.3.16 - 2018-04-08

Changes since 0.3.14 is mainly some internal cleanup, a link fix in
README and the optional rsass dependency is updated to 0.8.0.


## Release 0.3.14 - 2018-03-11

* Make the space after a comma in list expressions optional.
* Allow enum variants (and module names) in expressions.
* Some cleanup in parser code.


## Release 0.3.12 - 2018-02-10

* Add a way to add static files without hashnames.
  A static file can be added and mapped as an arbitrary name, or a
  directory can be recursively added with an arbitrary prefix.


## Release 0.3.10 - 2017-12-30

* Allow `*` at start of expressions (and subexpressions).
* Updated (optional) rsass to ^0.7.0.
* Updated base64 to ^0.9.0.


## Release 0.3.8 - 2017-12-07

* Make clippy happy with the code genarated for templates.
* Updated lazy_static to 1.0.
* Updated base64 to 0.8.
* Updated (optional) rsass to 0.6.


## Relese 0.3.6 - 2017-11-05

* Update nom dependency to version 3.2.
* Update optional rsass dependency to version 0.5.0.
* Update base64 dependency to 0.7.0.
* A documentation typo fixed, by @jo-so.
* Minor internal cleanup.


## Release 0.3.4 - 2017-07-10

* PR #15, issue #14: Allow destructure in loops, thanks to @nubis.
* PR #16 Allow complex argument types to templates
* PR #17 Write a doc chapter about static content.


## Release 0.3.2 - 2017-06-23

* Fix a bug in ordering (and therefor findability) of static files.
* Improved documentation.
* PR #13: Provide mime type for static file data.
* Fix file paths for `@import` in scss (using sass feature).
* Code cleanup (use `?` operator rather than `try!` macro).
* Use include_bytes for static files to improve compile times.


## Release 0.3.0 - 2017-05-07

* Issue #10: Watch template directories for changes, to build new
  templates when they are created.
* PR #12: Integrate sass, including a function to reference static
  files from a scss document.
* Some documentation improvements and internal code cleanup.


## Release 0.2.6 - 2017-03-23

* #8 / PR #11: Much improved error reporting.
* #9 allow comparison operators in if statements.
* Improved documentation, including a chapter on template structure in
  the docs (based on what was in the README.md).

This release is tested with rust versions 1.14.0, 1.15.1, 1.16.0
(stable), 1.17.0-beta.2 (b7c276653 2017-03-20), and 1.17.0-nightly
(8c4f2c64c 2017-03-22).


## Release 0.2.4 - 2017-02-05

* Test: expression may be in string, such as `<a href="@foo">...</a>`.
  This should work for all expressions.
* Handle escaped quotes in strings.
* PR #7: Allow slices in templates.
* Stop using the nom macro chain!, which is deprecated.

This release is tested with rust versions 1.14.0, 1.15.0 (stable),
1.16.0-beta.1 (beta), and 1.17.0-nightly (0648517fa 2017-02-03).


## Release 0.2.2 - 2017-01-29

* PR #3: Add convenient handling of static files
* Add documentation for utilities.


## Release 0.2.0 - 2017-01-28

* PR #6, Issue #5: Template directory structure.  Finds templates in
  all subdirectories of the template dir rather than only the template
  dir itself.  Template functions are created in a module structure
  that mirrors the directory structure.
  Thanks to @mrLSD for suggestion.
* Update `nom` to 2.0
* Use `base64` instead of entire `rustc_serialize` for just base64 coding.
* Issue #4: Mention curly brackets escaping in docs.  Thanks to @dermetfan.
* Cleanup, more tests and documentation.


## Release 0.1.2 - 2016-11-20

* Allow expressions to start with boolean not.
* DRYer test code.
* Write the generated code for each template into a separate file.
  As suggested by Jethro Beekman in
  [a comment on my blog](https://rasmus.krats.se/2016/ructe.en#c2427).
* More calling templates from templates doc.


## Release 0.1.1 - 2016-10-03

* Support calling templates with body arguments. Usefull for
  "base-page" templates.
* Provide `Html` trait to template code.
* Some testing and cleanup.


## Version 0.1.0 - 2016-09-24

* First version published on crates.io.
* Support for `@if` and `@for` blocks.
* Improved expression parsing with chaining, square and curly brakets,
  and string literals.
* Compile all found templates.
* More tests and documentation.


## Initial commit - 2016-09-14

Very siple templates, with arguments, worked, and text was
html-escaped as needed.
