use anyhow::{anyhow, Error};
use chrono::Utc;
use jsonwebtoken::{
    decode, encode, errors::ErrorKind, Algorithm, DecodingKey, EncodingKey, Validation,
};
use serde::{Deserialize, Serialize};

use crate::{model::User, AppErr};

use super::{JWT_AUD, JWT_EXPIRATION_TIME, JWT_ISS};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    // user id
    sub: i64,
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

    pub fn sign(&self, user: impl Into<User>) -> Result<String, AppErr> {
        let user = user.into();

        let claims = Claims {
            sub: user.id,
            username: user.username,
            email: user.email,
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

    pub fn verify(&self, token: &str) -> Result<Claims, AppErr> {
        let mut validation = Validation::new(Algorithm::EdDSA);
        validation.set_issuer(&[JWT_ISS]);
        validation.set_audience(&[JWT_AUD]);

        let token_data = match decode::<Claims>(token, &self.0, &validation) {
            Ok(td) => td,
            Err(e) => {
                return Err(AppErr::from(anyhow!(match *e.kind() {
                    ErrorKind::InvalidToken => "invalid token",
                    ErrorKind::InvalidIssuer => "invalid issuer",
                    ErrorKind::ExpiredSignature => "expired token",
                    _ => "some other err.",
                })))
            }
        };

        Ok(token_data.claims)
    }
}

#[cfg(test)]
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

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::{Ok, Result};

    #[test]
    fn test_jwt_sign_and_verify() -> Result<()> {
        let user = User::new_for_test(0, "foo".to_string(), "foo@acme.com".to_string());

        let ek = JwtEncodingKey::load(include_str!("../../fixtures/private.pem"))?;
        let token = ek.sign(user.clone())?;
        assert!(!token.is_empty());

        let dk = JwtDecodingKey::load(include_str!("../../fixtures/public.pem"))?;
        let claims = dk.verify(&token)?;
        assert_eq!(claims.sub, user.id);
        assert_eq!(claims.username, user.username);
        assert_eq!(claims.email, user.email);

        Ok(())
    }
}
