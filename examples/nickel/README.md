This is an example of using ructe with the nickel framework.

As for any rust project, the `Cargo.toml` file defines the project and
`src` contains the rust code.

The `templates` directory contains the ructe templates and the
`statics` code contains static resourses (images, in this example).
There is also a `style.scss` (which could be in a scss directory, but
since it's only one file I put it in the root).
These file and directory names are not magic, but defined in
`src/build.rs`.

The files in static is based on open clipart, modified by me and
optimized by [svgomg](https://jakearchibald.github.io/svgomg/).

* grass.svg is https://openclipart.org/detail/122113/grass
* cloud.svg is based on https://openclipart.org/detail/193560/cloud
* btfl.svg is based on https://openclipart.org/detail/149131/butterfly
