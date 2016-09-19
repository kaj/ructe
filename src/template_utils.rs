pub trait ToHtml {
    fn to_html(&self, out: &mut Write) -> io::Result<()>;
}

#[allow(dead_code)]
pub struct Html<T> (pub T);

impl<T: Display> ToHtml for Html<T> {
    fn to_html(&self, out: &mut Write) -> io::Result<()> {
        write!(out, "{}", self.0)
    }
}
impl<T: Display> ToHtml for T {
    fn to_html(&self, out: &mut Write) -> io::Result<()> {
        let mut buf = Vec::new();
        try!(write!(buf, "{}", self));
        out.write_all(&buf.into_iter().fold(Vec::new(), |mut v, c| {
            match c {
                b'<' => v.extend_from_slice(b"&lt;"),
                b'>' => v.extend_from_slice(b"&gt;"),
                b'&' => v.extend_from_slice(b"&amp;"),
                c => v.push(c),
            };
            v
        }))
    }
}
