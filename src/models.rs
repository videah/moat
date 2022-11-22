use webauthn_rs::prelude::{PasskeyAuthentication, PasskeyRegistration, Uuid};

#[derive(Clone, Debug)]
pub struct RegistrationSessionInfo {
    pub reg_state: PasskeyRegistration,
    pub email: String,
    pub unique_user_id: Uuid,
}

#[derive(Clone, Debug)]
pub struct AuthSessionInfo {
    pub current_user: Option<Uuid>,
    pub authentication_id: Option<Uuid>,
    pub auth_state: Option<PasskeyAuthentication>
}