extern crate mime;
use self::mime::Mime;

impl StaticFile {
    /// Get the mime type of this static file.
    ///
    /// Currently, this method parses a (static) string every time.
    /// A future release of `mime` may support statically created
    /// `Mime` structs, which will make this nicer.
    #[allow(unused)]
    pub fn mime(&self) -> Mime {
        self._mime.parse().unwrap()
    }
}
