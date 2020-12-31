use crate::components::prelude::*;
use crate::entities::create_blaster_tail;
use crate::frame_cnt::FrameCnt;
use crate::levels::LevelInfo;
use bevy::prelude::*;

#[system("move")]
pub fn move_blaster_head(
    commands: &mut Commands,
    frame_cnt: Res<FrameCnt>,
    level_info: Res<LevelInfo>,
    destroyable: Query<&Destroyable>,
    mut queries: QuerySet<(
        Query<(&Position, Entity), Without<Wall>>,
        Query<(Entity, &mut Position, &MovingDir), With<BlasterHead>>,
    )>
) {
    if !frame_cnt.is_keyframe() {
        return;
    }
    let mut occupied = level_info.get_occupied(queries.q0());

    for (entity, mut position, dir) in queries.q1_mut().iter_mut() {
        let new_pos = position.add(&*dir);
        let old_pos = *position;
        let is_wall = level_info.is_occupied(&new_pos);
        if is_wall {
            commands.despawn(entity);
        } else if let Some(&entity) = occupied.get_entity(&new_pos) {
            let is_destroyable = destroyable.get_component::<Destroyable>(entity).is_ok();
            if is_destroyable {
                commands.despawn(entity);
                occupied.remove(&new_pos);
                occupied.mv(&*position, &new_pos);
                *position = new_pos;
            } else {
                let entity = occupied.get_entity(&position).unwrap();
                commands.despawn(*entity);
            }
        } else {
            occupied.mv(&*position, &new_pos);
            *position = new_pos;
        }
        create_blaster_tail(commands).with(old_pos);
    }
}
