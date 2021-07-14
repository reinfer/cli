use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub struct Id(pub String);

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub struct Name(pub String);

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct EntityDef {
    pub color: u32,
    pub id: Id,
    pub inherits_from: Vec<Id>,
    pub name: Name,
    pub title: String,
    pub trainable: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct NewEntityDef {
    pub inherits_from: Vec<Id>,
    pub name: Name,
    pub title: String,
    pub trainable: bool,
}
