use std::sync::Arc;
use rocket::http::Status;
use rocket::State;
use rocket::serde::json::{Json, serde_json};
use rocket::serde::json;
use rocket_session_store::Session;
use webauthn_rs::prelude::*;
use webauthn_rs_proto::attest::CreationChallengeResponse;
use sea_orm_rocket::Connection;
use rocket::response::Redirect;

use sea_orm::{entity::*, query::*};
use webauthn_rs_proto::RequestChallengeResponse;
use crate::db::pool::Db;
use crate::models::{AuthSessionInfo, RegistrationSessionInfo};

use entity::prelude::*;
use crate::UserAuth;

pub struct AuthState {
    webauthn: Arc<Webauthn>,
}

impl AuthState {
    pub fn new() -> AuthState {
        let rp_id = std::env::var("MOAT_HOSTNAME").unwrap();
        let rp_origin = Url::parse(&format!("https://{rp_id}")).unwrap();
        let builder = WebauthnBuilder::new(&rp_id, &rp_origin).unwrap();
        let builder = builder.rp_name("Moat");

        let webauthn = Arc::new(builder.build().unwrap());

        AuthState { webauthn }
    }
}

#[post("/register_key/start?<email>&<registration_id>")]
pub async fn register_start(email: &str, registration_id: &str, reg_session: Session<'_, RegistrationSessionInfo>, auth_session: Session<'_, AuthSessionInfo>, state: &State<AuthState>, conn: Connection<'_, Db>)
    -> Result<Json<CreationChallengeResponse>, ()> {
    let db = conn.into_inner();

    // Check if the user already exists in the database using the email provided.
    let user_query = User::find()
        .filter(entity::user::Column::Email.eq(email))
        .one(db)
        .await.unwrap();

    println!("{:?}", user_query);

    // If the user already exists, check if we're currently authenticated as that user.
    if let Some(user) = user_query {
        println!("{:?}", user);
        let auth = auth_session.get().await
            .expect("Failed to access auth session")
            .expect("Failed to get auth session");
        if let Some(current_user) = auth.current_user {
            if current_user == Uuid::parse_str(&user.id).unwrap() {
                // We're already authenticated as the user!
            }
        }
    }

    let unique_user_id = Uuid::new_v4();

    reg_session.remove().await.expect("Failed to remove session");

    match state.webauthn.start_passkey_registration(
        unique_user_id,
        email,
        email,
        Some(vec![])
    ) {
        Ok((ccr, reg_state)) => {
            reg_session.set( RegistrationSessionInfo { reg_state, email: email.to_string(), unique_user_id }).await.unwrap();
            Ok(Json(ccr))
        }
        Err(e) => {
            Err(())
        }
    }
}

#[post("/register_key/finish?<name>&<color>", format = "json", data = "<data>")]
pub async fn register_finish(reg_session: Session<'_, RegistrationSessionInfo>, auth_session: Session<'_, AuthSessionInfo>, name: Option<String>, color: Option<String>,
                             data: Json<RegisterPublicKeyCredential>, state: &State<AuthState>, conn: Connection<'_, Db>) -> Status {

    let db = conn.into_inner();

    let session_info = reg_session.get().await.unwrap().unwrap();

    // Check if the user already exists in the database using the email provided.
    let user_query = User::find()
        .filter(entity::user::Column::Email.eq(&*session_info.email))
        .one(db)
        .await.unwrap();

    let user_already_existed = user_query.is_some();

    if !user_already_existed {
        let user = entity::user::ActiveModel {
            id: Set(session_info.unique_user_id.to_string()),
            email: Set(session_info.email.clone()),
            role: Set("user".to_string()),
            created_at: Set("".to_string()),
            last_authenticated: Set("".to_string()),
        };
        user.insert(db).await.unwrap();
    }

    // Run the query again since we may have inserted a new user.
    let user_query = User::find()
        .filter(entity::user::Column::Email.eq(&*session_info.email))
        .one(db)
        .await.unwrap();

    if let Some(user) = user_query {
        let res =  match state.webauthn.finish_passkey_registration(
            &data,
            &session_info.reg_state,
        ) {
            Ok(credential) => {
                // Save the key to the database
                let key = entity::key::ActiveModel {
                    id: Set(Uuid::new_v4().to_string()),
                    user_id: Set(user.id.clone()),
                    name: Set(name.unwrap()),
                    color: Set(color.unwrap()),
                    created_at: Set("".to_string()),
                    last_used: Set("".to_string()),
                    credential: Set(json::to_string(&credential).unwrap()),
                };
                key.insert(db).await.unwrap();

                if !user_already_existed {
                    println!("Created new user, authenticating.");
                    auth_session.set(
                        AuthSessionInfo {
                            current_user: Some(Uuid::parse_str(&user.id).expect("User ID is invalid.")),
                            authentication_id: None,
                            auth_state: None
                        }
                    ).await.expect("Could not authenticate user using auth session.");
                }

                Status::Ok
            }
            Err(_) => {
                Status::BadRequest
            }
        };
        return res;
    }

    Status::NotFound
}

#[post("/authenticate/start?<email>")]
pub async fn authenticate_start(email: &str, auth_session: Session<'_, AuthSessionInfo>, conn: Connection<'_, Db>, state: &State<AuthState>) -> Option<Json<RequestChallengeResponse>> {
    let db = conn.into_inner();

    // Find user using the provided email and check if they exist.
    let user: Option<entity::user::Model> = User::find()
        .filter(entity::user::Column::Email.eq(email))
        .one(db)
        .await.unwrap();

    if let Some(user) = user {
        // Get all passkeys associated with the provided email.
        let keys: Vec<entity::key::Model> = Key::find()
            .filter(entity::key::Column::UserId.eq(user.id.clone()))
            .all(db)
            .await.unwrap();

        // Convert all the stored passkey credential JSON into a usable format
        let credentials: Vec<Passkey> = keys.into_iter().map(
            |m| serde_json::from_str::<Passkey>(&m.credential).unwrap()
        ).collect();

        match state.webauthn.start_passkey_authentication(&credentials) {
            Ok((rcr, auth_state)) => {

                // Set required session values to pass to the next stage of authentication.
                auth_session.set(
                    AuthSessionInfo {
                        current_user: None,
                        authentication_id: Some(Uuid::parse_str(&user.id).unwrap()),
                        auth_state: Some(auth_state)
                    }
                ).await.expect("Could not create required auth session state.");

                return Some(Json(rcr))
            },
            Err(e) => {
                println!("Uh Oh");
            }
        };

        println!("{:?}", credentials);
    }

    None
}

#[post("/authenticate/finish", format = "json", data = "<data>")]
pub async fn authenticate_finish(auth_session: Session<'_, AuthSessionInfo>, data: Json<PublicKeyCredential>, state: &State<AuthState>) -> Status {

    let session = auth_session.get().await.unwrap().unwrap();
    let (user_unique_id, auth_state) = (session.authentication_id.unwrap(), session.auth_state.unwrap());

    // Wipe our auth session just in case
    auth_session.set(
        AuthSessionInfo {
            current_user: None,
            authentication_id: None,
            auth_state: None
        }
    ).await.expect("Could not reset ");

    println!("{:?} | {:?}", data, auth_state);

    match state.webauthn.finish_passkey_authentication(&data, &auth_state) {
        Ok(_auth_result) => {
            auth_session.set(
                AuthSessionInfo {
                    current_user: Some(user_unique_id),
                    authentication_id: None,
                    auth_state: None
                }
            ).await.expect("Could not set authentication state in session");
            Status::Ok
        },
        Err(e) => {
            println!("{e}");
            Status::BadRequest
        }
    }
}

#[get("/verify")]
pub async fn verify_authenticated(_user: UserAuth) -> &'static str {
    "Successfully Authenticated!"
}

#[get("/verify?<redirect>", rank = 2)]
pub async fn verify_unauthenticated(redirect: String) -> Redirect {
    let url = urlencoding::encode(&redirect);
    let rp_id = std::env::var("MOAT_HOSTNAME").unwrap();
    Redirect::to(format!("https://{rp_id}/?redirect={url}"))
}

#[get("/signout")]
pub async fn sign_out(session: Session<'_, AuthSessionInfo>) -> Redirect {
    session.set(
        AuthSessionInfo{
            current_user: None,
            authentication_id: None,
            auth_state: None
        }
    ).await.expect("Could not sign out current user.");
    Redirect::to("/")
}