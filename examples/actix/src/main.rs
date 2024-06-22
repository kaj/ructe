//! An example web service using ructe with actix web.
use actix_web::body::{BoxBody, EitherBody, MessageBody};
use actix_web::dev::ServiceResponse;
use actix_web::http::header::{ContentType, Expires};
use actix_web::http::{header, StatusCode};
use actix_web::middleware::{ErrorHandlerResponse, ErrorHandlers};
use actix_web::web::{resource, Path};
use actix_web::{App, HttpResponse, HttpServer, Result};
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
            .wrap(ErrorHandlers::new().default_handler(render_error))
            .service(resource("/").to(home_page))
            .service(resource("/static/{filename}").to(static_file))
            .service(resource("/int/{i}").to(take_int))
            .service(resource("/bad").to(make_error))
    })
    .bind("127.0.0.1:8088")
    .unwrap()
    .run()
    .await
    .expect("Run server");
}

/// Home page handler; just render a template with some arguments.
async fn home_page() -> HttpResponse {
    HttpResponse::Ok().body(
        render!(
            templates::page_html,
            &[("first", 3), ("second", 7), ("third", 2)]
        )
        .unwrap(),
    )
}

/// Handler for static files.
/// Create a response from the file data with a correct content type
/// and a far expires header (or a 404 if the file does not exist).
async fn static_file(path: Path<String>) -> HttpResponse {
    let name = &path.into_inner();
    if let Some(data) = StaticFile::get(name) {
        let far_expires = SystemTime::now() + FAR;
        HttpResponse::Ok()
            .insert_header(Expires(far_expires.into()))
            .insert_header(ContentType(data.mime.clone()))
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
    let i = args.into_inner();
    Ok(HttpResponse::Ok().body(render!(
        templates::page_html,
        &[(&format!("number {}", i), 1 + i % 7)],
    )?))
}

async fn make_error() -> Result<HttpResponse, ExampleAppError> {
    let i = "three".parse()?;
    Ok(HttpResponse::Ok()
        .body(render!(templates::page_html, &[("first", i)])?))
}

/// The error type that can be returned from resource handlers.
///
/// This needs to be convertible from any error types used with `?` in
/// handlers, and implement the actix ResponseError type.
#[derive(Debug)]
enum ExampleAppError {
    // May have other cases, for e.g. a backend not responding.
    InternalError,
}
impl actix_web::error::ResponseError for ExampleAppError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::new(StatusCode::INTERNAL_SERVER_ERROR)
    }
}
impl std::fmt::Display for ExampleAppError {
    fn fmt(&self, o: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(o, "{:?}", self)
    }
}
impl<E: std::error::Error> From<E> for ExampleAppError {
    fn from(value: E) -> Self {
        tracing::error!("Internal error: {value}");
        ExampleAppError::InternalError
    }
}

/// This method can be used as a "template tag", i.e. a method that
/// can be called directly from a template.
fn footer(out: &mut impl Write) -> io::Result<()> {
    templates::footer_html(
        out,
        &[
            ("ructe", "https://crates.io/crates/ructe"),
            ("actix-web", "https://crates.io/crates/actix-web"),
        ],
    )
}

fn render_error(
    res: ServiceResponse,
) -> Result<ErrorHandlerResponse<BoxBody>> {
    let req = res.request();
    let method = req.method().clone();
    let uri = req.uri().clone();
    let code = res.status();
    Ok(ErrorHandlerResponse::Response(res.map_body(
        move |head, body| {
            let body = body.try_into_bytes().unwrap_or_default();
            let body = String::from_utf8_lossy(&body);
            tracing::info!("Error {code} on '{method} {uri}': {body}");
            head.headers.insert(
                header::CONTENT_TYPE,
                header::HeaderValue::from_static(
                    mime::TEXT_HTML_UTF_8.as_ref(),
                ),
            );
            EitherBody::right(MessageBody::boxed(
                render!(templates::error_html, code, &body)
                    .unwrap_or(b"Error".into()),
            ))
        },
    )))
}

/// A duration to add to current time for a far expires header.
static FAR: Duration = Duration::from_secs(180 * 24 * 60 * 60);

// And finally, include the generated code for templates and static files.
include!(concat!(env!("OUT_DIR"), "/templates.rs"));
