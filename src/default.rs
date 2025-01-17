use std::{marker::PhantomData, task::Context, task::Poll};

use ntex::service::{Service, ServiceFactory};
use ntex::util::Ready;

use crate::error::LinkError;
use crate::{types::Link, ControlFrame, State};

/// Default publish service
pub(crate) struct DefaultPublishService<S, E>(PhantomData<(S, E)>);

impl<S, E> Default for DefaultPublishService<S, E> {
    fn default() -> Self {
        DefaultPublishService(PhantomData)
    }
}

impl<S, E> ServiceFactory<Link<S>, State<S>> for DefaultPublishService<S, E> {
    type Response = ();
    type Error = E;
    type InitError = LinkError;
    type Service = DefaultPublishService<S, E>;
    type Future = Ready<Self::Service, Self::InitError>;

    fn new_service(&self, _: State<S>) -> Self::Future {
        Ready::Err(LinkError::force_detach().description("not configured"))
    }
}

impl<S, E> Service<Link<S>> for DefaultPublishService<S, E> {
    type Response = ();
    type Error = E;
    type Future = Ready<Self::Response, Self::Error>;

    #[inline]
    fn poll_ready(&self, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    #[inline]
    fn call(&self, _pkt: Link<S>) -> Self::Future {
        log::warn!("AMQP Publish service is not configured");
        Ready::Ok(())
    }
}

/// Default control service
pub struct DefaultControlService<S, E>(PhantomData<(S, E)>);

impl<S, E> Default for DefaultControlService<S, E> {
    fn default() -> Self {
        DefaultControlService(PhantomData)
    }
}

impl<S, E> ServiceFactory<ControlFrame, State<S>> for DefaultControlService<S, E> {
    type Response = ();
    type Error = E;
    type InitError = E;
    type Service = DefaultControlService<S, E>;
    type Future = Ready<Self::Service, Self::InitError>;

    fn new_service(&self, _: State<S>) -> Self::Future {
        Ready::Ok(DefaultControlService(PhantomData))
    }
}

impl<S, E> Service<ControlFrame> for DefaultControlService<S, E> {
    type Response = ();
    type Error = E;
    type Future = Ready<Self::Response, Self::Error>;

    #[inline]
    fn poll_ready(&self, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    #[inline]
    fn call(&self, _pkt: ControlFrame) -> Self::Future {
        Ready::Ok(())
    }
}
