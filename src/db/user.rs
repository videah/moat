use entity::prelude::*;

use rocket::http::Status;
use rocket::outcome::Outcome;
use rocket::outcome::Outcome::Success;
use rocket::Request;
use rocket::request::FromRequest;
use rocket_session_store::Session;
use sea_orm::DatabaseConnection;
use sea_orm_rocket::Connection;
use crate::{AuthSessionInfo, Db};

use sea_orm::EntityTrait;

pub struct UserAuth(pub entity::user::Model);

#[derive(Debug)]
pub enum UserError {
    NotLoggedIn,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for UserAuth {
    type Error = UserError;

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, (Status, Self::Error), ()> {
        let user_result: &Option<entity::user::Model> = request.local_cache_async( async {
            let conn = request.guard::<Connection<'_, Db>>().await.succeeded().unwrap();
            let db: &DatabaseConnection = conn.into_inner();

            let session = request.guard::<Session<'_, AuthSessionInfo>>().await.succeeded().unwrap();
            let auth = session.get().await
                .expect("Failed to access auth session");

            if let Some(auth) = auth {
                println!("{:?}", auth.current_user);
                if let Some(user) = auth.current_user {
                    let user = User::find_by_id(user.to_string()).one(db).await;
                    return user.expect("Could not access users in database.")
                }
            }
            None
        }).await;

        match user_result {
            Some(user) => Success(UserAuth(user.clone())),
            None => Outcome::Forward(())
        }
    }
}