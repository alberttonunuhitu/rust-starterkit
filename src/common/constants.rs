// Pagination
pub const DEFAULT_PAGE: u64 = 1;
pub const DEFAULT_PER_PAGE: u64 = 10;
pub const MAX_PER_PAGE: u64 = 100;

// HTTP Headers
pub const X_REQUEST_ID_HEADER: &str = "x-request-id";

// Argon2 Password Hashing
pub const ARGON2_MEMORY_COST: u32 = 64 * 1024;
pub const ARGON2_TIME_COST: u32 = 3;
pub const ARGON2_PARALLELISM: u32 = 4;
