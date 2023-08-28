use std::{convert::Infallible, future::Future};

use bytes::Bytes;
use xitca_http::{
    h1::RequestBody,
    http::{header, RequestExt, Response, StatusCode},
    Request, ResponseBody,
};
use xitca_service::{ready::ReadyService, Service};

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

impl Service for Factory {
    type Response = Factory;
    type Error = Infallible;
    type Future<'f> = impl Future<Output = Result<Self::Response, Self::Error>> where Self: 'f;

    fn call<'s>(&'s self, _: ()) -> Self::Future<'s>
    where
        (): 's,
    {
        let this = self.clone();
        Box::pin(async { Ok(this) })
    }
}

impl Service<Request<RequestExt<RequestBody>>> for Factory {
    type Response = Response<ResponseBody>;
    type Error = Infallible;
    type Future<'f> = impl Future<Output = Result<Self::Response, Self::Error>> + 'f where Self: 'f;

    fn call<'s>(&'s self, req: Request<RequestExt<RequestBody>>) -> Self::Future<'s>
    where
        Request<RequestExt<RequestBody>>: 's,
    {
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

impl ReadyService for Factory {
    type Ready = ();
    type Future<'f> = impl Future<Output = Self::Ready> where Self: 'f;

    #[inline]
    fn ready(&self) -> Self::Future<'_> {
        async {}
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
