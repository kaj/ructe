macro_rules! render {
    ($template:path, $($arg:expr),* $(,)*) => {{
        let mut buf = Vec::new();
        $template(&mut buf, $($arg),*).map(|()| buf)
    }};
}
