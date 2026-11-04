use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct InitWorkspace {
    pub commands: Vec<String>,
}
