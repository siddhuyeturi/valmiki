use std::{
    future::{Ready, ready},
    pin::Pin,
    rc::Rc,
};

use actix_web::{
    Error, HttpMessage,
    cookie::{CookieBuilder, CookieJar, Key},
    dev::{Service, ServiceRequest, ServiceResponse, Transform, forward_ready},
};
use rand::{Rng, distr::Alphanumeric};

#[derive(Debug, Clone)]
pub struct VisitorId(Rc<str>);

impl VisitorId {
    fn generate(len: usize) -> Self {
        Self(
            rand::rng()
                .sample_iter(Alphanumeric)
                .take(len)
                .map(char::from)
                .collect::<String>()
                .into(),
        )
    }
}

impl AsRef<str> for VisitorId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl<T> From<T> for VisitorId
where
    T: Into<Rc<str>>,
{
    fn from(value: T) -> Self {
        Self(value.into())
    }
}

pub struct VisitorMiddleware {
    key: Key,
}

impl VisitorMiddleware {
    pub fn new(key: Key) -> Self {
        Self { key }
    }
}

impl<S, B> Transform<S, ServiceRequest> for VisitorMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = VisitorMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;
    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(VisitorMiddlewareService {
            key: self.key.clone().into(),
            service: service.into(),
        }))
    }
}

static COOKIE_NAME: &str = "__Secure-USID";

pub struct VisitorMiddlewareService<S> {
    key: Rc<Key>,
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for VisitorMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + 'static>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let key = self.key.clone();
        let service = self.service.clone();
        Box::pin(async move {
            let mut should_set_cookie: bool = false;
            let mut jar = CookieJar::new();
            let vid: VisitorId = req
                .cookie(COOKIE_NAME)
                .and_then(|c| {
                    jar.private(&key).decrypt(c).and_then(|c| {
                        let cv = c.value();
                        if cv.chars().any(|c| !c.is_alphanumeric()) {
                            None
                        } else {
                            Some(VisitorId::from(cv))
                        }
                    })
                })
                .unwrap_or_else(|| {
                    should_set_cookie = true;
                    VisitorId::generate(64)
                });

            let _ = req.extensions_mut().insert(vid.clone());

            let mut res = service.call(req).await?;

            if should_set_cookie {
                let vidc = CookieBuilder::new(COOKIE_NAME, vid.as_ref().to_string())
                    .permanent()
                    .http_only(true)
                    .secure(true)
                    .path("/")
                    .same_site(actix_web::cookie::SameSite::Strict)
                    .finish();
                jar.private_mut(&key).add(vidc);
                if let Some(vc) = jar.get(COOKIE_NAME) {
                    res.response_mut().add_cookie(vc)?;
                }
            }

            Ok(res)
        })
    }
}
