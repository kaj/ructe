extern crate mime;
use self::mime::Mime;

impl StaticFile {
    pub fn mime(&self) -> Mime {
        (&self._mime).into()
    }
}

impl<'a> Into<Mime> for &'a StaticMime {
    fn into(self) -> Mime {
        // TODO Should not need to parse all the time!
        self.0.parse().unwrap()
    }
}
