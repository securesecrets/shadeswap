use subtle::ConstantTimeEq;
use std::fmt;
use cosmwasm_std::{Env, Binary, MessageInfo};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::utils::crypto::{sha_256, Prng};

pub const VIEWING_KEY_SIZE: usize = 32;
const VIEWING_KEY_PREFIX: &str = "api_key_";

#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug, PartialEq)]
pub struct ViewingKey(pub String);

pub fn create_hashed_password(s1: &str) -> [u8; VIEWING_KEY_SIZE] {
    sha_256(s1.as_bytes())
}

impl ViewingKey {
    pub fn new(env: &Env, info: &MessageInfo, seed: &[u8], entropy: &[u8]) -> Self {
        // 16 here represents the lengths in bytes of the block height and time.
        let entropy_len = 16 + info.sender.to_string().len() + entropy.len();
        let mut rng_entropy = Vec::with_capacity(entropy_len);
        rng_entropy.extend_from_slice(&env.block.height.to_be_bytes());
        rng_entropy.extend_from_slice(&env.block.time.nanos().to_be_bytes());
        rng_entropy.extend_from_slice(&info.sender.as_bytes());
        rng_entropy.extend_from_slice(entropy);

        let mut rng = Prng::new(seed, &rng_entropy);

        let rand_slice = rng.rand_bytes();

        let key = sha_256(&rand_slice);

        Self(VIEWING_KEY_PREFIX.to_string() + &Binary::from(&key).to_base64())
    }

    pub fn to_hashed(&self) -> [u8; VIEWING_KEY_SIZE] {
        create_hashed_password(&self.0)
    }

    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }

    pub fn check_viewing_key(&self, hashed_pw: &[u8]) -> bool {
        let mine_hashed = create_hashed_password(&self.0);

        bool::from(mine_hashed.ct_eq(hashed_pw))
    }
}

impl fmt::Display for ViewingKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<&str> for ViewingKey {
    fn from (vk: &str) -> Self {
        ViewingKey(vk.into())
    }
}

pub fn create_viewing_key(env: &Env, info: &MessageInfo, seed: Binary, entroy: Binary) -> ViewingKey {
    ViewingKey::new(&env, info, seed.as_slice(), entroy.as_slice())
}