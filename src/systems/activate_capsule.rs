use crate::components::prelude::*;
use crate::entities::repair_capsule;
use crate::inventory::Inventory;
use crate::levels::LevelInfo;
use crate::plugins::audio::Sound;

use bevy::prelude::*;

#[system("tick")]
pub fn activate_capsule_system(
    commands: &mut Commands,
    inventory: Res<Inventory>,
    level_info: Res<LevelInfo>,
    mut sounds: ResMut<Events<Sound>>,
    mut query: Query<Entity, (With<Capsule>, Without<Usable>)>,
) {
    for capsule in query.iter_mut() {
        if inventory.screws >= level_info.screws {
            repair_capsule(commands, capsule);
            sounds.send(Sound::BOMB);
        }
    }
}
