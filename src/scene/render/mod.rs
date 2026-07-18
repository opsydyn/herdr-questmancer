pub mod lighting;

use crate::scene::{
    SceneFrame,
    assets::{
        adventurer::compact_adventurer_animation_frame,
        palette::{
            EMBER, FLAME, OAK, OAK_DARK, OAK_LIGHT, RUG, RUG_DARK, RUG_GOLD, SHADOW, STONE,
            STONE_DARK, STONE_LIGHT, VOID,
        },
    },
    pixel::{PixelPoint, PixelRect, PixelSize, Rgb, RgbBuffer},
    snapshot::SceneSnapshot,
    sprite::blit,
    stage::{ActorPlacement, SceneEffect, ScenePlan, TruthfulStation},
};

const ROOM_WIDTH: i32 = 120;
const ROOM_HEIGHT: i32 = 72;

pub fn paint(
    snapshot: &SceneSnapshot,
    plan: &ScenePlan,
    viewport: PixelSize,
    target: &mut RgbBuffer,
) -> SceneFrame {
    target.ensure_size(viewport.width, viewport.height, VOID);
    let origin = PixelPoint::new(
        (i32::from(viewport.width) - ROOM_WIDTH) / 2,
        (i32::from(viewport.height) - ROOM_HEIGHT) / 2,
    );

    paint_background(target);
    paint_floor_and_walls(target, origin);
    paint_architecture(target, origin);
    paint_furniture(target, origin);
    paint_actors(snapshot, plan, target, origin);
    lighting::apply_candle_light(target, translate(origin, 60, 36));
    paint_effects(plan, target, origin);

    SceneFrame {
        world: plan.world,
        next_frame_in: None,
    }
}

fn paint_background(target: &mut RgbBuffer) {
    target.clear(VOID);
}

fn paint_floor_and_walls(target: &mut RgbBuffer, origin: PixelPoint) {
    let size = target.size();
    let wall_break = (origin.y + 43).clamp(0, i32::from(size.height));
    for y in 0..size.height {
        for x in 0..size.width {
            let world_x = i32::from(x) - origin.x;
            let world_y = i32::from(y) - origin.y;
            let colour = if i32::from(y) < wall_break {
                match (world_x.div_euclid(6) + world_y.div_euclid(4)).rem_euclid(3) {
                    0 => STONE_DARK,
                    1 => STONE,
                    _ => STONE_LIGHT,
                }
            } else {
                match (world_x.div_euclid(10) + world_y).rem_euclid(3) {
                    0 => OAK_DARK,
                    1 => OAK,
                    _ => OAK_LIGHT,
                }
            };
            target.put(i32::from(x), i32::from(y), colour);
        }
    }
}

fn paint_architecture(target: &mut RgbBuffer, origin: PixelPoint) {
    fill(target, origin, PixelRect::new(3, 3, 114, 4), OAK_DARK);
    fill(target, origin, PixelRect::new(3, 40, 114, 3), OAK_DARK);
    for x in [3, 28, 59, 90, 114] {
        fill(target, origin, PixelRect::new(x, 3, 3, 40), OAK);
        fill(target, origin, PixelRect::new(x + 1, 4, 1, 37), OAK_LIGHT);
    }
    fill(target, origin, PixelRect::new(8, 12, 18, 24), SHADOW);
    fill(target, origin, PixelRect::new(10, 14, 14, 22), OAK_DARK);
    fill(target, origin, PixelRect::new(12, 18, 10, 18), STONE_DARK);
    fill(target, origin, PixelRect::new(96, 17, 14, 22), STONE_DARK);
    fill(target, origin, PixelRect::new(98, 19, 10, 18), SHADOW);
    fill(target, origin, PixelRect::new(101, 25, 4, 12), EMBER);
    fill(target, origin, PixelRect::new(100, 22, 6, 5), FLAME);
}

fn paint_furniture(target: &mut RgbBuffer, origin: PixelPoint) {
    fill(target, origin, PixelRect::new(27, 48, 65, 20), RUG_DARK);
    fill(target, origin, PixelRect::new(30, 50, 59, 16), RUG);
    fill(target, origin, PixelRect::new(34, 53, 51, 10), RUG_GOLD);
    fill(target, origin, PixelRect::new(35, 54, 49, 8), RUG_DARK);

    fill(target, origin, PixelRect::new(42, 38, 38, 13), OAK_DARK);
    fill(target, origin, PixelRect::new(44, 36, 34, 11), OAK);
    fill(target, origin, PixelRect::new(46, 37, 30, 2), OAK_LIGHT);
    fill(target, origin, PixelRect::new(47, 47, 4, 9), OAK_DARK);
    fill(target, origin, PixelRect::new(71, 47, 4, 9), OAK_DARK);

    fill(target, origin, PixelRect::new(59, 33, 3, 4), OAK_LIGHT);
    put(target, origin, 60, 31, FLAME);
    put(target, origin, 59, 32, EMBER);
    put(target, origin, 60, 32, FLAME);
    put(target, origin, 61, 32, EMBER);

    for (x, y) in [(39, 42), (51, 41), (68, 43), (75, 40)] {
        put(target, origin, x, y, STONE_LIGHT);
    }
}

fn paint_actors(
    snapshot: &SceneSnapshot,
    plan: &ScenePlan,
    target: &mut RgbBuffer,
    origin: PixelPoint,
) {
    let mut actors = plan
        .actors
        .iter()
        .filter_map(|placement| {
            snapshot
                .agents
                .iter()
                .find(|agent| agent.key == placement.agent)
                .map(|agent| (actor_anchor(placement), placement, agent))
        })
        .collect::<Vec<_>>();
    actors.sort_by_key(|(anchor, placement, _)| (anchor.y, placement.agent.clone()));
    for (anchor, placement, agent) in actors {
        let sprite = compact_adventurer_animation_frame(&agent.persona, placement.pose, 0);
        blit(&sprite, translate(origin, anchor.x, anchor.y), target);
    }
}

fn actor_anchor(placement: &ActorPlacement) -> PixelPoint {
    match &placement.station {
        TruthfulStation::CampaignToken(_) => PixelPoint::new(36, 48),
        TruthfulStation::CounselBell | TruthfulStation::DelveGate(_) => PixelPoint::new(82, 47),
        TruthfulStation::SpoilsBench | TruthfulStation::DelveExit(_) => PixelPoint::new(99, 47),
        TruthfulStation::Hearth => PixelPoint::new(87, 49),
        TruthfulStation::DelveActive(_) => PixelPoint::new(38, 48),
        TruthfulStation::DelveCamp(_) => PixelPoint::new(74, 49),
    }
}

fn paint_effects(plan: &ScenePlan, target: &mut RgbBuffer, origin: PixelPoint) {
    for effect in &plan.effects {
        match effect {
            SceneEffect::FreshSpoils { .. } => {
                for (x, y) in [(96, 45), (104, 43), (109, 48), (101, 40)] {
                    put(target, origin, x, y, RUG_GOLD);
                }
            }
            SceneEffect::RecentDeparture { .. } => {
                put(target, origin, 18, 16, STONE_LIGHT);
            }
        }
    }
}

fn fill(target: &mut RgbBuffer, origin: PixelPoint, rect: PixelRect, colour: Rgb) {
    target.fill_rect(
        PixelRect::new(
            origin.x.saturating_add(rect.x),
            origin.y.saturating_add(rect.y),
            rect.width,
            rect.height,
        ),
        colour,
    );
}

fn put(target: &mut RgbBuffer, origin: PixelPoint, x: i32, y: i32, colour: Rgb) {
    let point = translate(origin, x, y);
    target.put(point.x, point.y, colour);
}

const fn translate(origin: PixelPoint, x: i32, y: i32) -> PixelPoint {
    PixelPoint::new(origin.x.saturating_add(x), origin.y.saturating_add(y))
}
