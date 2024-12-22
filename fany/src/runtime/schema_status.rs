use crate::schema::{Schema, Session};

pub struct SchemaStatus {
    schema: Schema,
    session: Option<Session>,
}
