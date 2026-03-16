use crate::common::{ApiError, AuthenticatedUser};
use crate::slides::{
    dto::{CreateSlideRequest, ListSlidesResponse, SaveSlideRequest, SlideMetaResponse, SlideResponse},
    service::SlidesService,
};
use actix_web::{get, patch, post, web, HttpResponse};
use std::sync::Arc;
use utoipa::OpenApi;

pub struct SlidesApiState {
    pub slides_service: Arc<SlidesService>,
}

#[utoipa::path(
    get,
    path = "/api/v1/slides",
    responses(
        (status = 200, description = "List of presentations", body = ListSlidesResponse),
    ),
    security(("bearer_auth" = [])),
    tag = "slides"
)]
#[get("/slides")]
pub async fn list_slides(
    state: web::Data<SlidesApiState>,
    user: AuthenticatedUser,
) -> Result<web::Json<ListSlidesResponse>, ApiError> {
    let result = state.slides_service.list_slides(&user).await?;
    Ok(web::Json(result))
}

#[utoipa::path(
    post,
    path = "/api/v1/slides",
    request_body = CreateSlideRequest,
    responses(
        (status = 201, description = "Presentation created", body = SlideResponse),
        (status = 400, description = "Invalid request"),
    ),
    security(("bearer_auth" = [])),
    tag = "slides"
)]
#[post("/slides")]
pub async fn create_slide(
    state: web::Data<SlidesApiState>,
    user: AuthenticatedUser,
    body: web::Json<CreateSlideRequest>,
) -> Result<HttpResponse, ApiError> {
    let slide = state.slides_service.create_slide(&user, body.into_inner()).await?;
    Ok(HttpResponse::Created().json(slide))
}

#[utoipa::path(
    get,
    path = "/api/v1/slides/{id}",
    params(
        ("id" = String, Path, description = "Presentation ID")
    ),
    responses(
        (status = 200, description = "Presentation content", body = SlideResponse),
        (status = 403, description = "Access denied"),
        (status = 404, description = "Not found"),
    ),
    security(("bearer_auth" = [])),
    tag = "slides"
)]
#[get("/slides/{id}")]
pub async fn get_slide(
    state: web::Data<SlidesApiState>,
    user: AuthenticatedUser,
    path: web::Path<String>,
) -> Result<web::Json<SlideResponse>, ApiError> {
    let slide_id = path.into_inner();
    let slide = state.slides_service.get_slide(&user, &slide_id).await?;
    Ok(web::Json(slide))
}

#[utoipa::path(
    patch,
    path = "/api/v1/slides/{id}",
    params(
        ("id" = String, Path, description = "Presentation ID")
    ),
    request_body = SaveSlideRequest,
    responses(
        (status = 200, description = "Presentation saved", body = SlideMetaResponse),
        (status = 403, description = "Access denied"),
        (status = 404, description = "Not found"),
    ),
    security(("bearer_auth" = [])),
    tag = "slides"
)]
#[patch("/slides/{id}")]
pub async fn save_slide(
    state: web::Data<SlidesApiState>,
    user: AuthenticatedUser,
    path: web::Path<String>,
    body: web::Json<SaveSlideRequest>,
) -> Result<web::Json<SlideMetaResponse>, ApiError> {
    let slide_id = path.into_inner();
    let meta = state
        .slides_service
        .save_slide(&user, &slide_id, body.into_inner())
        .await?;
    Ok(web::Json(meta))
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(list_slides)
        .service(create_slide)
        .service(get_slide)
        .service(save_slide);
}

#[derive(OpenApi)]
#[openapi(
    paths(list_slides, create_slide, get_slide, save_slide),
    components(schemas(
        CreateSlideRequest,
        SaveSlideRequest,
        SlideResponse,
        SlideMetaResponse,
        ListSlidesResponse,
    )),
    tags((name = "slides", description = "Native presentation editor")),
    security(("bearer_auth" = []))
)]
pub struct SlidesApiDoc;
