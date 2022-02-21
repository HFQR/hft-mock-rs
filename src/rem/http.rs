use std::{convert::Infallible, future::Future};

use bytes::Bytes;
use xitca_http::{
    h1::RequestBody,
    http::{header, Response, StatusCode},
    Request, ResponseBody,
};
use xitca_service::{ready::ReadyService, Service, ServiceFactory};

use super::SharedState;

#[derive(Clone)]
pub struct Factory {
    shared_state: SharedState,
}

impl Factory {
    pub fn new(shared_state: SharedState) -> Self {
        Self { shared_state }
    }
}

impl ServiceFactory<Request<RequestBody>> for Factory {
    type Response = Response<ResponseBody>;
    type Error = Infallible;
    type Service = Factory;
    type Future = impl Future<Output = Result<Self::Service, Self::Error>>;

    fn new_service(&self, _: ()) -> Self::Future {
        let this = self.clone();
        Box::pin(async { Ok(this) })
    }
}

impl Service<Request<RequestBody>> for Factory {
    type Response = Response<ResponseBody>;
    type Error = Infallible;
    type Future<'f> = impl Future<Output = Result<Self::Response, Self::Error>>;

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

impl ReadyService<Request<RequestBody>> for Factory {
    type Ready = ();
    type ReadyFuture<'f> = impl Future<Output = Result<Self::Ready, Self::Error>>;

    #[inline]
    fn ready(&self) -> Self::ReadyFuture<'_> {
        async { Ok(()) }
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
