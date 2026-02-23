use uuid::Uuid;

#[derive(Clone)]
pub struct Group {
    pub id: Uuid,
    pub name: String,
    pub password: Option<String>,
    pub persistent: bool,
    pub hidden: bool,
    pub group_type: i32, // Normal = 0, Open = 1, Isolated = 2
}
