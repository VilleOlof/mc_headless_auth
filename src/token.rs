use std::fmt::Debug;

use rand::RngExt;

use crate::User;

pub trait TokenGenerator: Debug + Clone {
    fn generate(&self, user: &User) -> String;
    fn display(&self, token: &str) -> String {
        token.to_string()
    }
}

#[derive(Debug, Clone)]
pub struct Token;
impl Token {
    const FANCY_CHARS: [char; 26] = [
        'ᴀ', 'ʙ', 'ᴄ', 'ᴅ', 'ᴇ', 'ꜰ', 'ɢ', 'ʜ', 'ɪ', 'ᴊ', 'ᴋ', 'ʟ', 'ᴍ', 'ɴ', 'ᴏ', 'ᴘ', 'ǫ', 'ʀ',
        'ꜱ', 'ᴛ', 'ᴜ', 'ᴠ', 'ᴡ', 'x', 'ʏ', 'ᴢ',
    ];
    const CHARS: [char; 26] = [
        'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R',
        'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z',
    ];
    const LENGTH: usize = 10;
}

impl TokenGenerator for Token {
    fn generate(&self, _: &User) -> String {
        let mut rng = rand::rng();

        let token: String = (0..Self::LENGTH)
            .map(|_| {
                let i = rng.random_range(0..Self::CHARS.len());
                char::from(Self::CHARS[i])
            })
            .collect();

        token
    }

    fn display(&self, token: &str) -> String {
        let mut out = String::with_capacity(token.len());
        for char in token.chars() {
            let idx = Self::CHARS.iter().position(|c| c == &char).unwrap();
            out.push(Self::FANCY_CHARS[idx]);
        }
        return out;
    }
}

pub mod storage {
    use std::{
        collections::HashMap,
        sync::{Arc, Mutex},
        thread,
        time::Duration,
    };

    use chrono::{DateTime, Utc};

    use crate::User;

    #[derive(Debug, Clone)]
    struct StorageCell {
        pub time: DateTime<Utc>,
        pub data: User,
    }

    type StorageInternal = Arc<Mutex<HashMap<String, StorageCell>>>;

    #[derive(Debug, Clone)]
    pub struct TokenStorage {
        /// A map of tokens that are mapped to a user and a date when the token was set
        tokens: StorageInternal,
    }

    impl TokenStorage {
        pub fn new(ttl: Duration) -> Self {
            let storage = Self {
                tokens: Arc::new(Mutex::new(HashMap::new())),
            };

            let _tokens = storage.tokens.clone();
            thread::spawn(move || Self::start_storage_cleaner(_tokens, ttl));

            storage
        }

        pub fn insert(&self, token: String, user: User) -> i64 {
            let mut lock = self.tokens.lock().unwrap();
            let time = Utc::now();

            lock.insert(token, StorageCell { time, data: user });

            time.timestamp()
        }

        pub fn get(&self, token: &String) -> Option<User> {
            self.tokens
                .lock()
                .unwrap()
                .get(token)
                .map(|s| s.data.clone())
        }

        fn start_storage_cleaner(tokens: StorageInternal, ttl: Duration) -> ! {
            loop {
                {
                    let mut lock = tokens.lock().unwrap();

                    let mut invalid_tokens = Vec::new();
                    for (token, data) in lock.iter() {
                        if Utc::now() > (data.time + ttl) {
                            invalid_tokens.push(token.clone());
                        }
                    }

                    for token in invalid_tokens {
                        lock.remove(&token);
                    }
                }

                // we can also use the ttl for the interval
                thread::sleep(ttl);
            }
        }
    }
}
