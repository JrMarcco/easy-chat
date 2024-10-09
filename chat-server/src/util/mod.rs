mod jwt;

pub(crate) use jwt::{JwtDecodingKey, JwtEncodingKey};

const JWT_ISS: &str = "easy-chat";
const JWT_AUD: &str = "chat-client";
// jwt expire in 30 minutes
const JWT_EXPIRATION_TIME: i64 = 30 * 60;
