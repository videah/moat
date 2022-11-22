use askama::Template;

#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate {
    pub passkeys: Vec<entity::key::Model>,
    pub user: Option<entity::user::Model>,
}

#[derive(Template)]
#[template(path = "key.html")]
pub struct KeyTemplate {
    pub passkey: entity::key::Model,
    pub user: Option<entity::user::Model>
}

#[derive(Template)]
#[template(path = "sign_in.html")]
pub struct SignInTemplate {
    pub user: Option<entity::user::Model>,
    pub redirect: Option<String>,
}

#[derive(Template)]
#[template(path = "new_key.html")]
pub struct NewKeyTemplate {
    pub is_link: bool,
    pub link_id: String,
    pub user: Option<entity::user::Model>
}

#[derive(Template)]
#[template(path = "admin_settings.html")]
pub struct AdminSettingsTemplate {
    pub user: Option<entity::user::Model>
}

#[derive(Template)]
#[template(path = "admin/registration_links.html")]
pub struct RegistrationLinksTemplate {
    pub registration_links: Vec<entity::registration_link::Model>,
    pub user: Option<entity::user::Model>
}