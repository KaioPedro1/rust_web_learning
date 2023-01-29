use actix_files as fs;
use actix_web::{
    cookie::{ Cookie},
    http::header::{ContentType, LOCATION},
    web::{self}, Error, HttpResponse,
};
use chrono::{Duration, Utc};
use actix_web::cookie::time::Duration as dr;
use jsonwebtoken::{encode, Header, EncodingKey};
use sqlx::PgPool;
use uuid::Uuid;
use crate::{model::{User, UserName, Claims}, database};


pub fn validade_and_build(form: FormData) -> Result<User, String> {
    let name = UserName::parse(form.name)?;
    let id =  Uuid::new_v4();
    Ok(User { name, id })
}

#[derive(serde::Deserialize, Debug)]
pub struct FormData {
    pub name: String,
}

pub async fn root_get() -> Result<fs::NamedFile, Error> {
    let file = fs::NamedFile::open_async("./static/index.html")
        .await?
        .use_last_modified(true)
        .use_etag(true);
    Ok(file)
}

pub async fn root_post(form: web::Form<FormData>, connection: web::Data<PgPool>) -> HttpResponse { 
    match validade_and_build(form.0) {
        Ok(register) =>{ 
            database::insert_user_db(&register, connection).await;
            let claims = Claims {
                sub: register.id.to_string(),
                name: register.name.0,
                exp: (Utc::now() + Duration::hours(24)).timestamp() as usize,
            };
            let token = encode(
                &Header::default(),
                &claims,
                &EncodingKey::from_secret("secret".as_ref()),
            )
            .unwrap();

            let url_to_redirect = "/lobby";
        
            let jwt_cookie = Cookie::build("jwt", token.clone())
            .path(url_to_redirect)
            .max_age(dr::hours(24))
            .finish();

            HttpResponse::Found()
                .content_type(ContentType::html())
                .append_header((LOCATION, url_to_redirect))
                .cookie(jwt_cookie)
                .finish()
        },
        Err(_) => return HttpResponse::BadRequest().finish(),
    }
}

