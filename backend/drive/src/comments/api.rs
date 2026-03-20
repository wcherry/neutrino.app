use crate::comments::{
    dto::{CreateCommentRequest, CreateReplyRequest, UpdateCommentRequest},
    service::CommentsService,
};
use crate::common::{ApiError, AuthenticatedUser};
use actix_web::{delete, get, patch, post, web, HttpResponse};
use std::sync::Arc;

pub struct CommentsApiState {
    pub comments_service: Arc<CommentsService>,
}

#[get("/files/{id}/comments")]
pub async fn list_comments(
    state: web::Data<CommentsApiState>,
    user: AuthenticatedUser,
    path: web::Path<String>,
    query: web::Query<std::collections::HashMap<String, String>>,
) -> Result<HttpResponse, ApiError> {
    let file_id = path.into_inner();
    let status = query.get("status").map(|s| s.as_str());
    let result = state.comments_service.list_comments(&user, &file_id, status)?;
    Ok(HttpResponse::Ok().json(result))
}

#[post("/files/{id}/comments")]
pub async fn create_comment(
    state: web::Data<CommentsApiState>,
    user: AuthenticatedUser,
    path: web::Path<String>,
    body: web::Json<CreateCommentRequest>,
) -> Result<HttpResponse, ApiError> {
    let file_id = path.into_inner();
    let result = state.comments_service.create_comment(&user, &file_id, body.into_inner()).await?;
    Ok(HttpResponse::Created().json(result))
}

#[patch("/files/{id}/comments/{cid}")]
pub async fn update_comment(
    state: web::Data<CommentsApiState>,
    user: AuthenticatedUser,
    path: web::Path<(String, String)>,
    body: web::Json<UpdateCommentRequest>,
) -> Result<HttpResponse, ApiError> {
    let (file_id, comment_id) = path.into_inner();
    let result = state.comments_service.update_comment(&user, &file_id, &comment_id, body.into_inner())?;
    Ok(HttpResponse::Ok().json(result))
}

#[delete("/files/{id}/comments/{cid}")]
pub async fn delete_comment(
    state: web::Data<CommentsApiState>,
    user: AuthenticatedUser,
    path: web::Path<(String, String)>,
) -> Result<HttpResponse, ApiError> {
    let (file_id, comment_id) = path.into_inner();
    state.comments_service.delete_comment(&user, &file_id, &comment_id)?;
    Ok(HttpResponse::NoContent().finish())
}

#[post("/files/{id}/comments/{cid}/replies")]
pub async fn add_reply(
    state: web::Data<CommentsApiState>,
    user: AuthenticatedUser,
    path: web::Path<(String, String)>,
    body: web::Json<CreateReplyRequest>,
) -> Result<HttpResponse, ApiError> {
    let (file_id, comment_id) = path.into_inner();
    let result = state.comments_service.add_reply(&user, &file_id, &comment_id, body.into_inner()).await?;
    Ok(HttpResponse::Created().json(result))
}

#[delete("/files/{id}/comments/{cid}/replies/{rid}")]
pub async fn delete_reply(
    state: web::Data<CommentsApiState>,
    user: AuthenticatedUser,
    path: web::Path<(String, String, String)>,
) -> Result<HttpResponse, ApiError> {
    let (file_id, comment_id, reply_id) = path.into_inner();
    state.comments_service.delete_reply(&user, &file_id, &comment_id, &reply_id)?;
    Ok(HttpResponse::NoContent().finish())
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(list_comments)
        .service(create_comment)
        .service(update_comment)
        .service(delete_comment)
        .service(add_reply)
        .service(delete_reply);
}
