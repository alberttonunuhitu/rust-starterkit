use uuid::{Error, Uuid};

pub fn generate_uuidv7() -> Uuid {
    Uuid::now_v7()
}

pub fn generate_uuidv7_string() -> String {
    generate_uuidv7().to_string()
}

pub fn parse_uuid(uuid_str: &str) -> Result<Uuid, Error> {
    Uuid::parse_str(uuid_str)
}
