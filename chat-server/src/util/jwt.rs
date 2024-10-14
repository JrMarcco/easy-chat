use anyhow::Error;
use chrono::Utc;
use jsonwebtoken::{
    decode, encode, errors::ErrorKind, Algorithm, DecodingKey, EncodingKey, Validation,
};
use serde::{Deserialize, Serialize};

use crate::{model::SessionUser, AppErr};

use super::{JWT_AUD, JWT_EXPIRATION_TIME, JWT_ISS};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    uid: i64,
    username: String,
    email: String,
    iss: String,
    aud: String,
    exp: i64,
}

// private key
// openssl genpkey -algorithm ed25519 -out private.pem
pub struct JwtEncodingKey(EncodingKey);

impl JwtEncodingKey {
    pub fn load(pem: &str) -> Result<Self, Error> {
        let ek = EncodingKey::from_ed_pem(pem.as_bytes())?;
        Ok(Self(ek))
    }

    pub fn sign(&self, user: impl Into<SessionUser>) -> Result<String, AppErr> {
        let session_user = user.into();

        let claims = Claims {
            uid: session_user.id,
            username: session_user.username,
            email: session_user.email,
            iss: JWT_ISS.to_string(),
            aud: JWT_AUD.to_string(),
            exp: Utc::now().timestamp() + JWT_EXPIRATION_TIME,
        };

        let token = encode(
            &jsonwebtoken::Header::new(Algorithm::EdDSA),
            &claims,
            &self.0,
        )?;
        Ok(token)
    }
}

// public key
// openssl pkey -in private.pem -outform PEM -pubout -out public.pem
pub struct JwtDecodingKey(DecodingKey);

impl JwtDecodingKey {
    pub fn load(pem: &str) -> Result<Self, Error> {
        let dk = DecodingKey::from_ed_pem(pem.as_bytes())?;
        Ok(Self(dk))
    }

    pub fn verify(&self, token: &str) -> Result<SessionUser, AppErr> {
        let mut validation = Validation::new(Algorithm::EdDSA);
        validation.set_issuer(&[JWT_ISS]);
        validation.set_audience(&[JWT_AUD]);

        let token_data = match decode::<Claims>(token, &self.0, &validation) {
            Ok(td) => td,
            Err(e) => {
                let msg = match *e.kind() {
                    ErrorKind::InvalidToken => "invalid token",
                    ErrorKind::InvalidIssuer => "invalid issuer",
                    ErrorKind::ExpiredSignature => "expired token",
                    _ => "some other error",
                };
                return Err(AppErr::AuthErr(msg.to_string()));
            }
        };

        let claims = token_data.claims;
        Ok(SessionUser::new(claims.uid, claims.username, claims.email))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::User;
    use anyhow::{Ok, Result};

    impl User {
        pub fn new_for_test(id: i64, username: String, email: String) -> Self {
            Self {
                id,
                username,
                email,
                passwd: "".to_string(),
                avatar: "".to_string(),
                created_at: Utc::now(),
                updated_at: Utc::now(),
            }
        }
    }

    #[test]
    fn test_jwt_sign_and_verify() -> Result<()> {
        let user = User::new_for_test(0, "foo".to_string(), "foo@acme.com".to_string());

        let ek = JwtEncodingKey::load(include_str!("../../fixtures/private.pem"))?;
        let token = ek.sign(user.clone())?;
        assert!(!token.is_empty());

        let dk = JwtDecodingKey::load(include_str!("../../fixtures/public.pem"))?;
        let session_user = dk.verify(&token)?;
        assert_eq!(session_user.id, user.id);
        assert_eq!(session_user.username, user.username);
        assert_eq!(session_user.email, user.email);

        Ok(())
    }
}
