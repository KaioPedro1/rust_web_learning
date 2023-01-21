use actix_web::{http::header::LOCATION,HttpRequest, HttpResponse, web, Responder};
use actix_files as fs;
use sqlx::PgPool;
use uuid::Uuid;

use crate::database;

pub enum FilesOptions {
    Lobby,
    Room,
}
pub async fn open_file_return_http_response(req: &HttpRequest, opt: FilesOptions) -> HttpResponse {
    let file_path = match opt {
        FilesOptions::Lobby => "./static/lobby.html",
        FilesOptions::Room => "./static/room.html",
    };
    match fs::NamedFile::open_async(file_path).await {
        Ok(file) => file.use_last_modified(true).use_etag(true).respond_to(req),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}
pub async fn check_if_cookie_is_valid(
    req: &HttpRequest,
    conn: web::Data<PgPool>,
) -> Result<Uuid, HttpResponse> {
    let cookie = req.cookie("uuid").ok_or(
        HttpResponse::TemporaryRedirect()
            .append_header((LOCATION, "/"))
            .finish(),
    )?;
    let user_uuid = Uuid::parse_str(cookie.value()).map_err(|_| {
        HttpResponse::TemporaryRedirect()
            .append_header((LOCATION, "/"))
            .finish()
    })?;

    match database::check_user_id_db(user_uuid, conn).await {
        Ok(_) => Ok(user_uuid),
        Err(_) => Err(HttpResponse::TemporaryRedirect()
            .append_header((LOCATION, "/"))
            .finish()),
    }
}