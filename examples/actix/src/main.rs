//! An example web service using ructe with actix web.
use actix_web::body::Body;
use actix_web::dev::ServiceResponse;
use actix_web::http::header::{ContentType, Expires};
use actix_web::http::{header, StatusCode};
use actix_web::middleware::errhandlers::{
    ErrorHandlerResponse, ErrorHandlers,
};
use actix_web::web::Path;
use actix_web::{web, App, HttpResponse, HttpServer, Result};
use std::io::{self, Write};
use std::time::{Duration, SystemTime};
use templates::statics::StaticFile;

#[macro_use]
mod actix_ructe;

/// Main program: Set up routes and start server.
#[actix_web::main]
async fn main() {
    env_logger::init();
    HttpServer::new(|| {
        App::new()
            .wrap(
                ErrorHandlers::new()
                    .handler(StatusCode::NOT_FOUND, render_404)
                    .handler(StatusCode::INTERNAL_SERVER_ERROR, render_500),
            )
            .service(web::resource("/").to(home_page))
            .service(web::resource("/static/{filename}").to(static_file))
            .service(web::resource("/int/{i}").to(take_int))
            .service(web::resource("/bad").to(make_error))
    })
    .bind("127.0.0.1:8088")
    .unwrap()
    .run()
    .await
    .expect("Run server");
}

/// Home page handler; just render a template with some arguments.
fn home_page() -> HttpResponse {
    HttpResponse::Ok().body(render!(
        templates::page,
        &[("first", 3), ("second", 7), ("third", 2)]
    ))
}

/// Handler for static files.
/// Create a response from the file data with a correct content type
/// and a far expires header (or a 404 if the file does not exist).
fn static_file(path: Path<String>) -> HttpResponse {
    let name = &path.0;
    if let Some(data) = StaticFile::get(name) {
        let far_expires = SystemTime::now() + FAR;
        HttpResponse::Ok()
            .set(Expires(far_expires.into()))
            .set(ContentType(data.mime.clone()))
            .body(data.content)
    } else {
        HttpResponse::NotFound()
            .reason("No such static file.")
            .finish()
    }
}

async fn take_int(
    args: Path<usize>,
) -> Result<HttpResponse, ExampleAppError> {
    let i = args.0;
    Ok(HttpResponse::Ok().body(render!(
        templates::page,
        &[(&format!("number {}", i), 1 + i % 7)],
    )))
}

async fn make_error() -> Result<HttpResponse, ExampleAppError> {
    let i = "three".parse()?;
    Ok(HttpResponse::Ok().body(render!(templates::page, &[("first", i)])))
}

/// The error type that can be returned from resource handlers.
///
/// This needs to be convertible from any error types used with `?` in
/// handlers, and implement the actix ResponseError type.
#[derive(Debug)]
enum ExampleAppError {
    ParseInt(std::num::ParseIntError),
}
impl std::fmt::Display for ExampleAppError {
    fn fmt(&self, o: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(o, "{:?}", self)
    }
}
impl std::error::Error for ExampleAppError {}

impl From<std::num::ParseIntError> for ExampleAppError {
    fn from(e: std::num::ParseIntError) -> Self {
        ExampleAppError::ParseInt(e)
    }
}
impl actix_web::error::ResponseError for ExampleAppError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::new(StatusCode::INTERNAL_SERVER_ERROR)
    }
}

/// This method can be used as a "template tag", i.e. a method that
/// can be called directly from a template.
fn footer(out: &mut dyn Write) -> io::Result<()> {
    templates::footer(
        out,
        &[
            ("ructe", "https://crates.io/crates/ructe"),
            ("actix-web", "https://crates.io/crates/actix-web"),
        ],
    )
}

fn render_404(
    res: ServiceResponse<Body>,
) -> Result<ErrorHandlerResponse<Body>> {
    error_response(
        res,
        StatusCode::NOT_FOUND,
        "The resource you requested can't be found.",
    )
}

fn render_500(
    res: ServiceResponse<Body>,
) -> Result<ErrorHandlerResponse<Body>> {
    error_response(
        res,
        StatusCode::INTERNAL_SERVER_ERROR,
        "Sorry, Something went wrong.  This is probably not your fault.",
    )
}

fn error_response(
    mut res: ServiceResponse<Body>,
    status_code: StatusCode,
    message: &str,
) -> Result<ErrorHandlerResponse<Body>> {
    res.headers_mut().insert(
        header::CONTENT_TYPE,
        header::HeaderValue::from_str(mime::TEXT_HTML_UTF_8.as_ref())
            .unwrap(),
    );
    Ok(ErrorHandlerResponse::Response(res.map_body(
        |_head, _body| {
            actix_web::dev::ResponseBody::Body(
                render!(templates::error, status_code, message).into(),
            )
        },
    )))
}

/// A duration to add to current time for a far expires header.
static FAR: Duration = Duration::from_secs(180 * 24 * 60 * 60);

// And finally, include the generated code for templates and static files.
include!(concat!(env!("OUT_DIR"), "/templates.rs"));
