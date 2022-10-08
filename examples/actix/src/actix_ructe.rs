macro_rules! render {
    ($template:path) => (Render(|o| $template(o)));
    ($template:path, $($arg:expr),* $(,)*) => {{
        use $crate::actix_ructe::Render;
        Render(move |o| $template(o, $($arg),*))
    }};
}

pub struct Render<T: FnOnce(&mut Vec<u8>) -> std::io::Result<()>>(pub T);

impl<T: FnOnce(&mut Vec<u8>) -> std::io::Result<()>> From<Render<T>>
    for actix_web::body::Body
{
    fn from(t: Render<T>) -> Self {
        let mut buf = Vec::new();
        match t.0(&mut buf) {
            Ok(()) => buf.into(),
            Err(_e) => {
                //log::warn!("Failed to render: {}", e);
                "Render failed".into()
            }
        }
    }
}
