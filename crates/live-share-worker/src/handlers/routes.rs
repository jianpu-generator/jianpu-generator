//! Registers each route (`super::routes`) into both the worker `Router` and
//! the OpenAPI spec `web/` generates its typed client from, in one call.
//! Every part of a route's spec is derived from its handler's signature --
//! path params from an `IntoParams` struct, the request body from a
//! `ToSchema` type, the success response from its `Reply` type, and
//! `ApiError` as every route's `default` response -- so there's no separate
//! annotation to keep in step with the handler, and no hand-written TS to
//! keep in step with either.

use std::collections::{BTreeMap, BTreeSet};
use std::future::Future;
use std::rc::Rc;

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use utoipa::openapi::path::{
    HttpMethod, OperationBuilder, Parameter, ParameterIn, PathItem, PathsBuilder,
};
use utoipa::openapi::request_body::RequestBodyBuilder;
use utoipa::openapi::response::ResponseBuilder;
use utoipa::openapi::schema::{ComponentsBuilder, Schema};
use utoipa::openapi::{Content, Info, OpenApi, OpenApiBuilder, Ref, RefOr, Required};
use utoipa::{IntoParams, ToSchema};
use worker::{Request, Response, RouteContext, Router};

use crate::api_error::ApiError;

pub(crate) type HandlerResult<T> = Result<T, ApiError>;

const JSON: &str = "application/json";

/// The `components.schemas` section being accumulated, keyed by name.
#[derive(Default)]
pub(crate) struct Schemas(BTreeMap<String, RefOr<Schema>>);

impl Schemas {
    fn add<T: ToSchema>(&mut self) {
        let mut nested = Vec::new();
        T::schemas(&mut nested);
        self.0.extend(nested);
        self.0.insert(T::name().into_owned(), T::schema());
    }
}

fn json_content<T: ToSchema>() -> Content {
    Content::new(Some(Ref::from_schema_name(T::name())))
}

/// A handler's success value: how it becomes a `Response`, and how it's
/// described in the spec.
pub(crate) trait Reply {
    fn into_response(self) -> worker::Result<Response>;
    fn document(operation: OperationBuilder, schemas: &mut Schemas) -> OperationBuilder;
}

/// `200` with `T` as its JSON body.
pub(crate) struct Json<T>(pub T);

impl<T: Serialize + ToSchema> Reply for Json<T> {
    fn into_response(self) -> worker::Result<Response> {
        Response::from_json(&self.0)
    }

    fn document(operation: OperationBuilder, schemas: &mut Schemas) -> OperationBuilder {
        schemas.add::<T>();
        operation.response(
            "200",
            ResponseBuilder::new()
                .description("OK")
                .content(JSON, json_content::<T>()),
        )
    }
}

/// `204`, no body.
pub(crate) struct NoContent;

impl Reply for NoContent {
    fn into_response(self) -> worker::Result<Response> {
        Ok(Response::empty()?.with_status(204))
    }

    fn document(operation: OperationBuilder, _schemas: &mut Schemas) -> OperationBuilder {
        operation.response("204", ResponseBuilder::new().description("No Content"))
    }
}

/// Path params of a route whose path has none.
#[derive(Deserialize)]
pub(crate) struct NoPathParams {}

impl IntoParams for NoPathParams {
    fn into_params(_parameter_in: impl Fn() -> Option<ParameterIn>) -> Vec<Parameter> {
        Vec::new()
    }
}

/// `{name}` segments of an OpenAPI path template, e.g. `id` in
/// `/files/{id}/rename`.
fn template_param_names(path: &str) -> BTreeSet<String> {
    path.split('/')
        .filter_map(|segment| segment.strip_prefix('{')?.strip_suffix('}'))
        .map(str::to_owned)
        .collect()
}

/// `Params`' field names, checked against `path`'s `{name}` segments -- so a
/// template/struct mismatch fails `routes()` itself (and with it the spec
/// export test), rather than 400-ing every request at runtime.
fn path_param_names<Params: IntoParams>(path: &str) -> worker::Result<Rc<[String]>> {
    let names: Vec<String> = Params::into_params(|| Some(ParameterIn::Path))
        .into_iter()
        .map(|parameter| parameter.name)
        .collect();
    let declared: BTreeSet<String> = names.iter().cloned().collect();
    if declared != template_param_names(path) {
        return Err(worker::Error::RustError(format!(
            "route {path}: path params {declared:?} don't match its template"
        )));
    }
    Ok(names.into())
}

/// Builds `Params` from the matched route's params, by name.
fn read_path_params<Params: DeserializeOwned>(
    ctx: &RouteContext<()>,
    names: &[String],
) -> HandlerResult<Params> {
    let values = names
        .iter()
        .map(|name| {
            let value = ctx.param(name).ok_or(ApiError::BadRequest)?;
            Ok((name.clone(), serde_json::Value::String(value.clone())))
        })
        .collect::<HandlerResult<serde_json::Map<_, _>>>()?;
    serde_json::from_value(serde_json::Value::Object(values)).map_err(|_| ApiError::BadRequest)
}

fn respond<R: Reply>(outcome: HandlerResult<R>) -> worker::Result<Response> {
    let mut response = match outcome {
        Ok(reply) => reply.into_response()?,
        Err(error) => Response::from_json(&error)?.with_status(error.status_code()),
    };
    // A viewer's page reload is the only way they ever see an owner's later
    // edit (see `useSyncedShareViewer.ts`'s doc comment) -- letting the
    // browser cache `GET /shares/{share_id}` would mean that reload
    // sometimes doesn't actually re-fetch. Nothing here is ever worth
    // caching, so this applies to every route.
    response.headers_mut().set("Cache-Control", "no-store")?;
    Ok(response)
}

/// matchit (the worker `Router`'s matcher) spells `{id}` as `:id`.
fn router_pattern(path: &str) -> String {
    path.replace('{', ":").replace('}', "")
}

pub(crate) struct Routes {
    router: Router<'static, ()>,
    paths: PathsBuilder,
    schemas: Schemas,
}

impl Routes {
    pub(crate) fn new() -> Self {
        let mut schemas = Schemas::default();
        schemas.add::<ApiError>();
        Routes {
            router: Router::new(),
            paths: PathsBuilder::new(),
            schemas,
        }
    }

    fn operation<Params: IntoParams, R: Reply>(&mut self) -> OperationBuilder {
        let operation = OperationBuilder::new()
            .parameters(Some(Params::into_params(|| Some(ParameterIn::Path))))
            .response(
                "default",
                ResponseBuilder::new()
                    .description("Error")
                    .content(JSON, json_content::<ApiError>()),
            );
        R::document(operation, &mut self.schemas)
    }

    /// A `GET` route: path params only, no request body.
    pub(crate) fn get<Params, R, F, Fut>(mut self, path: &str, handler: F) -> worker::Result<Self>
    where
        Params: DeserializeOwned + IntoParams + 'static,
        R: Reply + 'static,
        F: Fn(RouteContext<()>, Params) -> Fut + Copy + 'static,
        Fut: Future<Output = HandlerResult<R>> + 'static,
    {
        let names = path_param_names::<Params>(path)?;
        let operation = self.operation::<Params, R>();
        self.paths = self
            .paths
            .path(path, PathItem::new(HttpMethod::Get, operation));
        self.router = self
            .router
            .get_async(&router_pattern(path), move |_req: Request, ctx| {
                let names = Rc::clone(&names);
                async move {
                    respond(
                        async {
                            let params = read_path_params(&ctx, &names)?;
                            handler(ctx, params).await
                        }
                        .await,
                    )
                }
            });
        Ok(self)
    }

    /// A `POST` route with a JSON request body.
    pub(crate) fn post<Params, Body, R, F, Fut>(
        mut self,
        path: &str,
        handler: F,
    ) -> worker::Result<Self>
    where
        Params: DeserializeOwned + IntoParams + 'static,
        Body: DeserializeOwned + ToSchema + 'static,
        R: Reply + 'static,
        F: Fn(RouteContext<()>, Params, Body) -> Fut + Copy + 'static,
        Fut: Future<Output = HandlerResult<R>> + 'static,
    {
        let names = path_param_names::<Params>(path)?;
        self.schemas.add::<Body>();
        let operation = self.operation::<Params, R>().request_body(Some(
            RequestBodyBuilder::new()
                .content(JSON, json_content::<Body>())
                .required(Some(Required::True))
                .build(),
        ));
        self.paths = self
            .paths
            .path(path, PathItem::new(HttpMethod::Post, operation));
        self.router =
            self.router
                .post_async(&router_pattern(path), move |mut req: Request, ctx| {
                    let names = Rc::clone(&names);
                    async move {
                        respond(
                            async {
                                let params = read_path_params(&ctx, &names)?;
                                let body =
                                    req.json::<Body>().await.map_err(|_| ApiError::BadRequest)?;
                                handler(ctx, params, body).await
                            }
                            .await,
                        )
                    }
                });
        Ok(self)
    }

    pub(crate) fn into_router(self) -> Router<'static, ()> {
        self.router
    }

    pub(crate) fn into_openapi(self) -> OpenApi {
        OpenApiBuilder::new()
            .info(Info::new("live-share-worker", env!("CARGO_PKG_VERSION")))
            .paths(self.paths)
            .components(Some(
                ComponentsBuilder::new()
                    .schemas_from_iter(self.schemas.0)
                    .build(),
            ))
            .build()
    }
}
