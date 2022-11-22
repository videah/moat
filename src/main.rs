mod templates;
mod auth;
mod models;
mod db;
mod utils;

#[macro_use] extern crate rocket;

use std::time::Duration;

use rocket::{Build, Rocket};
use rocket::fairing::AdHoc;
use rocket::fs::FileServer;
use rocket::http::private::cookie::CookieBuilder;
use rocket_session_store::{
    memory::MemoryStore,
    SessionStore,
};

use sea_orm::EntityTrait;
use sea_orm_rocket::{Connection, Database};
use sea_orm::entity::*;

use db::pool::Db;
use migration::MigratorTrait;

use entity::prelude::*;
use crate::db::user::UserAuth;
use crate::models::AuthSessionInfo;

#[get("/")]
async fn index(conn: Connection<'_, Db>, user: UserAuth) -> templates::IndexTemplate {
    let db = conn.into_inner();

    let passkeys: Vec<entity::key::Model> = Key::find().all(db).await.unwrap();
    templates::IndexTemplate {
        passkeys,
        user: Some(user.0)
    }

}

#[get("/?<redirect>", rank = 2)]
async fn index_authenticate(redirect: Option<String>) -> templates::SignInTemplate {
    templates::SignInTemplate {
        user: None,
        redirect,
    }
}

#[get("/key/<id>")]
async fn key_details(conn: Connection<'_, Db>, id: String, user: UserAuth) -> templates::KeyTemplate {
    let db = conn.into_inner();
    let passkey = Key::find_by_id(id).one(db).await.unwrap().unwrap();
    templates::KeyTemplate {
        passkey,
        user: Some(user.0)
    }
}

#[get("/authenticate")]
async fn authenticate() -> templates::SignInTemplate {
    templates::SignInTemplate {
        user: None,
        redirect: None,
    }
}

#[get("/new-key")]
async fn new_key() -> templates::NewKeyTemplate {
    templates::NewKeyTemplate {
        is_link: false,
        link_id: "".to_string(),
        user: None,
    }
}

#[get("/admin/settings")]
async fn admin_settings() -> templates::AdminSettingsTemplate {
    templates::AdminSettingsTemplate {
        user: None,
    }
}

#[get("/new-key/<id>")]
async fn new_key_from_url(conn: Connection<'_, Db>, id: String) -> Option<templates::NewKeyTemplate> {

    use sea_orm::QueryFilter;
    use sea_orm::ColumnTrait;

    let db = conn.into_inner();
    let registration_link = RegistrationLink::find()
        .filter(entity::registration_link::Column::HumanId.eq(id))
        .one(db)
        .await.unwrap();

    if registration_link.is_some() {
        Some(templates::NewKeyTemplate { is_link: true, link_id: registration_link.unwrap().human_id, user: None })
    } else {
        None
    }
}

#[get("/admin/links")]
async fn registration_links(conn: Connection<'_, Db>) -> templates::RegistrationLinksTemplate {
    let db = conn.into_inner();

    let registration_links: Vec<entity::registration_link::Model> = RegistrationLink::find().all(db).await.unwrap();
    templates::RegistrationLinksTemplate {
        registration_links,
        user: None,
    }
}

async fn run_migrations(rocket: Rocket<Build>) -> rocket::fairing::Result {
    let conn = &Db::fetch(&rocket).unwrap().conn;
    let _ = migration::Migrator::up(conn, None).await;

    // If there's no links we can assume that there are no users.
    // We need a way for an initial user to be created so we create a link and print it.
    let links: Vec<entity::registration_link::Model> = RegistrationLink::find().all(conn).await.unwrap();
    if links.is_empty() {
        let link = entity::registration_link::ActiveModel {
            human_id: Set(human_id::id("-", false)),
            ..Default::default()
        };
        let link = link.insert(conn).await.unwrap();
        let rp_id = std::env::var("MOAT_HOSTNAME").unwrap();
        println!("No accounts detected. Creating registration link for initial admin account.");
        println!("This will only be displayed once.");
        println!("https://{rp_id}/new-key/{}", link.human_id);
    }
    Ok(rocket)
}

#[launch]
fn rocket() -> _ {

    // Get hostname from env
    let hostname = std::env::var("MOAT_HOSTNAME").expect("MOAT_HOSTNAME must be set. This is the domain will be hosting Moat on");

    let registration_memory_store: MemoryStore::<models::RegistrationSessionInfo> = MemoryStore::default();
    let reg_store: SessionStore<models::RegistrationSessionInfo> = SessionStore {
        store: Box::new(registration_memory_store),
        name: "moat_reg".to_string(),
        duration: Duration::from_secs(3600 * 24 * 3),
        cookie_builder: CookieBuilder::new("", "").secure(true),
    };

    let authentication_memory_store: MemoryStore::<AuthSessionInfo> = MemoryStore::default();
    let auth_store: SessionStore<AuthSessionInfo> = SessionStore {
        store: Box::new(authentication_memory_store),
        name: "moat_auth".to_string(),
        duration: Duration::from_secs(3600 * 24 * 3),
        cookie_builder: CookieBuilder::new("", "").secure(true),
    };

    let state = auth::AuthState::new();

    println!("🏰 Starting Moat on: {hostname}");
    rocket::build()
        .attach(Db::init())
        .attach(AdHoc::try_on_ignite("Migrations", run_migrations))
        .mount("/", routes![
            index,
            index_authenticate,
            key_details,
            authenticate,
            new_key,
            admin_settings,
            new_key_from_url,
            registration_links,
            auth::register_start,
            auth::register_finish,
            auth::authenticate_start,
            auth::authenticate_finish,
            auth::verify_authenticated,
            auth::verify_unauthenticated,
            auth::sign_out,
        ])
        .mount("/static", FileServer::from("static"))
        .manage(state)
        .attach(reg_store.fairing())
        .attach(auth_store.fairing())
}