use bevy::prelude::*;

#[derive(Resource)]
pub struct MenuResource {
    pub username: String
}

impl Default for MenuResource {
    fn default() -> MenuResource {
        MenuResource {
            username: "user1".to_string()
        }
    }
}