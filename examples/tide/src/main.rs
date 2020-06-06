// And finally, include the generated code for templates and static files.
include!(concat!(env!("OUT_DIR"), "/templates.rs"));

mod ructe_tide;
use ructe_tide::Render;

use tide::{Response, StatusCode};

#[async_std::main]
async fn main() -> Result<(), std::io::Error> {
    let mut app = tide::new();

    app.at("/").get(|_| async {
        let mut res = Response::new(StatusCode::Ok);
        res.render_html(|o| Ok(templates::hello(o, "world")?))?;
        Ok(res)
    });

    let addr = "127.0.0.1:3000";
    println!("Starting server on http://{}/", addr);
    app.listen(addr).await?;

    Ok(())
}
