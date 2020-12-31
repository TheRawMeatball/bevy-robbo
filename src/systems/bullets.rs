use crate::components::prelude::*;
use crate::entities::create_small_explosion;
use crate::frame_cnt::FrameCnt;
use crate::levels::LevelInfo;
use crate::resources::DamageMap;
use bevy::prelude::*;

#[system("move")]
pub fn move_bullet(
    commands: &mut Commands,
    frame_cnt: Res<FrameCnt>,
    level_info: ResMut<LevelInfo>,
    mut damage_map: ResMut<DamageMap>,
    mut queries: QuerySet<(
        Query<(Entity, &mut Position, &mut MovingDir), With<Bullet>>,
        Query<(&Position, Entity), Without<Wall>>,
    )>,
) {
    if !frame_cnt.is_keyframe() {
        return;
    }
    let mut occupied = level_info.get_occupied( queries.q1());
    for (entity, mut position, mut dir) in queries.q0_mut().iter_mut() {
        let new_pos = position.add(&*dir);
        if occupied.is_occupied(&new_pos) {
            *dir = MovingDir::zero();
            commands.despawn(entity);
            create_small_explosion(commands).with(*position);
            damage_map.do_damage(&new_pos, false);
        } else {
            occupied.mv(&*position, &new_pos);
            *position = new_pos;
        }
    }
}
