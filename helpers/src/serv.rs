pub fn run(routes: &[Route]) {
    use {
        async_executor::Executor,
        async_net::TcpListener,
        futures_lite::future,
        http_body_util::Full,
        hyper::{
            Method, Request, Response, StatusCode, header, http::HeaderValue, server::conn::http1,
            service,
        },
        smol_hyper::rt::FuturesIo,
        std::{
            collections::HashMap,
            convert::Infallible,
            net::{Ipv4Addr, SocketAddr},
            sync::Arc,
        },
    };

    let routes = {
        let routes: HashMap<_, _> = routes.iter().copied().collect();
        Arc::new(routes)
    };

    let page = move |req: Request<_>| match (req.method(), routes.get(req.uri().path())) {
        (&Method::GET, Some(page)) => {
            let mut res = Response::new(Full::new(page.body));
            res.headers_mut().insert(
                header::CONTENT_TYPE,
                HeaderValue::from_static(page.content_type),
            );

            Some(res)
        }
        _ => None,
    };

    let no_found = || {
        let mut res = Response::default();
        *res.status_mut() = StatusCode::NOT_FOUND;
        res
    };

    let ex = Executor::new();
    let run = async {
        let addr = SocketAddr::from((Ipv4Addr::LOCALHOST, 3000));
        let listener = match TcpListener::bind(addr).await {
            Ok(listener) => listener,
            Err(e) => return e,
        };

        println!("server listens on http://{addr}");

        loop {
            let stream = match listener.accept().await {
                Ok((stream, _)) => FuturesIo::new(stream),
                Err(e) => return e,
            };

            let serve = service::service_fn(async |req| {
                let res = page(req).unwrap_or_else(no_found);
                Ok::<_, Infallible>(res)
            });

            let task = async move {
                if let Err(e) = http1::Builder::new().serve_connection(stream, serve).await {
                    eprintln!("connection error: {e}");
                }
            };

            ex.spawn(task).detach();
        }
    };

    let e = future::block_on(ex.run(run));
    eprintln!("io error: {e}");
}

type Route = (&'static str, Page);

#[derive(Clone, Copy)]
pub struct Page {
    content_type: &'static str,
    body: &'static [u8],
}

impl Page {
    pub const fn html(body: &'static str) -> Self {
        Self {
            content_type: "text/html; charset=utf-8",
            body: body.as_bytes(),
        }
    }

    pub const fn js(body: &'static str) -> Self {
        Self {
            content_type: "text/javascript",
            body: body.as_bytes(),
        }
    }

    pub const fn wasm(body: &'static [u8]) -> Self {
        Self {
            content_type: "application/wasm",
            body,
        }
    }
}
