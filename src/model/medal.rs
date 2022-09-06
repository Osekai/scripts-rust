use serde::Serialize;

#[derive(Serialize)]
pub struct Medal {
    medal_id: u32,
    name: String,
    url: String,
    description: String,
    restriction: String,
    grouping: String,
    instructions: String,
    ordering: u8,
}
