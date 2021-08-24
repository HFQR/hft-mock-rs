use std::{
    convert::Infallible,
    future::Future,
    task::{Context, Poll},
};

use bytes::Bytes;
use xitca_http::{
    h1::RequestBody,
    http::{header, Request, Response, StatusCode},
    HttpServiceBuilder, ResponseBody,
};
use xitca_server::net::TcpStream;
use xitca_service::{Service, ServiceFactory};

use super::SharedState;
use xitca_http::config::HttpServiceConfig;

pub(super) fn factory(shared_state: SharedState) -> impl ServiceFactory<TcpStream, Config = ()> {
    let config = HttpServiceConfig::new().max_request_headers::<16>();
    HttpServiceBuilder::h1(Factory { shared_state })
        .config(config)
        .with_logger()
}

#[derive(Clone)]
struct Factory {
    shared_state: SharedState,
}

impl ServiceFactory<Request<RequestBody>> for Factory {
    type Response = Response<ResponseBody>;
    type Error = Infallible;
    type Config = ();
    type Service = Factory;
    type InitError = ();
    type Future = impl Future<Output = Result<Self::Service, Self::InitError>>;

    fn new_service(&self, _: Self::Config) -> Self::Future {
        let this = self.clone();
        Box::pin(async { Ok(this) })
    }
}

impl Service<Request<RequestBody>> for Factory {
    type Response = Response<ResponseBody>;
    type Error = Infallible;
    type Future<'f> = impl Future<Output = Result<Self::Response, Self::Error>>;

    #[inline(always)]
    fn poll_ready(&self, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&self, req: Request<RequestBody>) -> Self::Future<'_> {
        async move {
            let (parts, _) = req.into_parts();

            let res = match parts.uri.path() {
                "/" => get(&self.shared_state),
                "/clear" => clear(&self.shared_state),
                _ => not_found(),
            };

            Ok(res)
        }
    }
}

fn get(shared_state: &SharedState) -> Response<ResponseBody> {
    use sailfish::TemplateOnce;
    let state = shared_state.collect().render_once().unwrap();

    Response::builder()
        .status(StatusCode::OK)
        .header(
            header::CONTENT_TYPE,
            header::HeaderValue::from_static("text/html; charset=utf-8"),
        )
        .body(Bytes::from(state).into())
        .unwrap()
}

fn clear(shared_state: &SharedState) -> Response<ResponseBody> {
    shared_state.clear();

    Response::builder()
        .status(StatusCode::SEE_OTHER)
        .header(header::LOCATION, header::HeaderValue::from_static("/"))
        .body(Bytes::new().into())
        .unwrap()
}

#[cold]
#[inline(never)]
fn not_found() -> Response<ResponseBody> {
    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body(Bytes::new().into())
        .unwrap()
}
