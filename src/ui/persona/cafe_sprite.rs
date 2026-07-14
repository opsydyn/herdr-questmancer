use crate::{
    domain::{BodyProportions, FaceDetail, HairShape, OutfitTop, PersonaAppearance, Shoes},
    ui::{
        pixel::{Canvas, ColorRole, Palette},
        theatre::TheatreFrame,
    },
};

use super::{
    appearance::{AppearanceRoles, appearance_roles, appearance_roles_for_palette},
    state_pose::{SeatedLayout, SeatedPose, seated_pose},
};

const CAFE_WIDTH: u16 = 10;
const CAFE_HEIGHT: u16 = 12;

pub fn compose_seated(appearance: &PersonaAppearance, frame: TheatreFrame) -> Canvas {
    compose_seated_with_roles(appearance, frame, appearance_roles(appearance))
}

pub fn compose_seated_for_palette(
    appearance: &PersonaAppearance,
    frame: TheatreFrame,
    palette: Palette,
) -> Canvas {
    compose_seated_with_roles(
        appearance,
        frame,
        appearance_roles_for_palette(appearance, palette),
    )
}

fn compose_seated_with_roles(
    appearance: &PersonaAppearance,
    frame: TheatreFrame,
    roles: AppearanceRoles,
) -> Canvas {
    let mut canvas = Canvas::new(CAFE_WIDTH, CAFE_HEIGHT);
    let pose = seated_pose(frame);
    if pose == SeatedPose::Absent {
        return canvas;
    }

    let layout = SeatedLayout::for_proportions(appearance.proportions);
    draw_head(&mut canvas, appearance, roles, layout);
    draw_torso(&mut canvas, appearance, roles, layout);
    draw_pose(&mut canvas, roles, layout, pose);
    draw_seated_legs(&mut canvas, appearance, roles, layout);
    draw_accessory(&mut canvas, appearance, roles, layout);
    canvas
}

fn draw_head(
    canvas: &mut Canvas,
    appearance: &PersonaAppearance,
    roles: AppearanceRoles,
    layout: SeatedLayout,
) {
    let inset = matches!(
        appearance.head_shape,
        crate::domain::HeadShape::Round | crate::domain::HeadShape::Angular
    );
    if inset {
        draw_run(
            canvas,
            layout.head_x + 1,
            layout.head_y,
            layout.head_width - 2,
            roles.hair,
        );
    } else {
        draw_run(
            canvas,
            layout.head_x,
            layout.head_y,
            layout.head_width,
            roles.hair,
        );
    }
    canvas.fill_rect(
        layout.head_x,
        layout.head_y + 1,
        layout.head_width,
        3,
        roles.skin,
    );

    match appearance.hair {
        HairShape::Shaved => draw_run(
            canvas,
            layout.head_x,
            layout.head_y,
            layout.head_width,
            roles.hair,
        ),
        HairShape::Fringe => draw_run(
            canvas,
            layout.head_x,
            layout.head_y + 1,
            layout.head_width / 2 + 1,
            roles.hair,
        ),
        HairShape::Curls | HairShape::Bob => {
            canvas.set(layout.head_x, layout.head_y + 2, roles.hair);
            canvas.set(
                layout.head_x + layout.head_width - 1,
                layout.head_y + 2,
                roles.hair,
            );
        }
        HairShape::Quiff | HairShape::Spikes => {
            canvas.set(
                layout.head_x + 1,
                layout.head_y.saturating_sub(1),
                roles.hair,
            );
            canvas.set(layout.head_x + 2, layout.head_y, roles.hair);
        }
        HairShape::Ponytail => {
            canvas.set(
                layout.head_x + layout.head_width,
                layout.head_y + 2,
                roles.hair,
            );
            canvas.set(
                layout.head_x + layout.head_width,
                layout.head_y + 3,
                roles.hair,
            );
        }
        HairShape::Crop => {}
    }

    match appearance.face_detail {
        FaceDetail::None => canvas.set(layout.head_x + 1, layout.head_y + 2, roles.shadow),
        FaceDetail::RoundGlasses | FaceDetail::SquareGlasses | FaceDetail::Visor => {
            draw_run(
                canvas,
                layout.head_x + 1,
                layout.head_y + 2,
                layout.head_width.saturating_sub(2),
                roles.accent,
            );
        }
        FaceDetail::Freckles => {
            canvas.set(layout.head_x + 1, layout.head_y + 2, roles.accent);
            canvas.set(
                layout.head_x + layout.head_width - 2,
                layout.head_y + 2,
                roles.accent,
            );
        }
        FaceDetail::Moustache => {
            draw_run(canvas, layout.head_x + 1, layout.head_y + 3, 2, roles.hair);
        }
    }
}

fn draw_torso(
    canvas: &mut Canvas,
    appearance: &PersonaAppearance,
    roles: AppearanceRoles,
    layout: SeatedLayout,
) {
    canvas.fill_rect(
        layout.torso_x,
        layout.torso_y,
        layout.torso_width,
        layout.torso_height,
        roles.top,
    );
    match appearance.top {
        OutfitTop::StripeJumper | OutfitTop::TrackTop => {
            draw_run(
                canvas,
                layout.torso_x,
                layout.torso_y + 1,
                layout.torso_width,
                roles.accent,
            );
            draw_run(
                canvas,
                layout.torso_x,
                layout.torso_y + 3,
                layout.torso_width,
                roles.accent,
            );
        }
        OutfitTop::HighCollar => draw_run(
            canvas,
            layout.torso_x + 1,
            layout.torso_y,
            layout.torso_width - 2,
            roles.accent,
        ),
        OutfitTop::WorkShirt | OutfitTop::Cardigan | OutfitTop::Waistcoat => {
            for y in layout.torso_y..layout.torso_y + layout.torso_height {
                canvas.set(layout.torso_x + layout.torso_width / 2, y, roles.accent);
            }
        }
        OutfitTop::BandTee | OutfitTop::Hoodie => canvas.set(
            layout.torso_x + layout.torso_width / 2,
            layout.torso_y + 2,
            roles.accent,
        ),
    }
}

fn draw_pose(canvas: &mut Canvas, roles: AppearanceRoles, layout: SeatedLayout, pose: SeatedPose) {
    let shoulder_y = layout.torso_y + 1;
    match pose {
        SeatedPose::CrtFacing { hand_phase } => {
            canvas.set(layout.torso_x.saturating_sub(1), shoulder_y, roles.skin);
            let hand_x = if hand_phase { 9 } else { 8 };
            let hand_y = if hand_phase {
                shoulder_y + 2
            } else {
                shoulder_y + 1
            };
            canvas.set(hand_x, hand_y, roles.skin);
        }
        SeatedPose::RaisedHand => {
            canvas.set(layout.torso_x.saturating_sub(1), shoulder_y + 1, roles.skin);
            canvas.set(8, 3, roles.skin);
            canvas.set(8, 4, roles.skin);
            canvas.set(9, 4, roles.skin);
        }
        SeatedPose::Relaxed => {
            canvas.set(layout.torso_x.saturating_sub(1), shoulder_y + 1, roles.skin);
            canvas.set(
                layout.torso_x + layout.torso_width,
                shoulder_y + 1,
                roles.skin,
            );
        }
        SeatedPose::Absent => {}
    }
}

fn draw_seated_legs(
    canvas: &mut Canvas,
    appearance: &PersonaAppearance,
    roles: AppearanceRoles,
    layout: SeatedLayout,
) {
    let hips_y = (layout.torso_y + layout.torso_height).min(9);
    draw_run(
        canvas,
        layout.torso_x,
        hips_y,
        layout.torso_width,
        roles.bottom,
    );

    let left = match appearance.proportions {
        BodyProportions::Broad | BodyProportions::Compact => 2,
        BodyProportions::Average | BodyProportions::Tall => 3,
    };
    let leg_width = match appearance.proportions {
        BodyProportions::Compact => 2,
        BodyProportions::Broad => 3,
        BodyProportions::Average | BodyProportions::Tall => 1,
    };
    let right = 6;
    for y in hips_y.saturating_add(1)..=10 {
        draw_run(canvas, left, y, leg_width, roles.bottom);
        draw_run(canvas, right, y, leg_width, roles.bottom);
    }
    let shoe_width = if appearance.shoes == Shoes::Platforms {
        leg_width + 1
    } else {
        leg_width.max(2)
    };
    draw_run(canvas, left.saturating_sub(1), 11, shoe_width, roles.shoes);
    draw_run(canvas, right, 11, shoe_width, roles.shoes);
}

fn draw_accessory(
    canvas: &mut Canvas,
    appearance: &PersonaAppearance,
    roles: AppearanceRoles,
    layout: SeatedLayout,
) {
    use crate::domain::Accessory;
    match appearance.accessory {
        Accessory::Headphones => {
            canvas.set(
                layout.head_x.saturating_sub(1),
                layout.head_y + 2,
                roles.accessory,
            );
            canvas.set(
                layout.head_x + layout.head_width,
                layout.head_y + 2,
                roles.accessory,
            );
        }
        Accessory::Pager | Accessory::Badge | Accessory::PocketPen => canvas.set(
            layout.torso_x + layout.torso_width - 1,
            layout.torso_y + 1,
            roles.accessory,
        ),
        Accessory::Lanyard => {
            canvas.set(layout.torso_x + 1, layout.torso_y, roles.accessory);
            canvas.set(layout.torso_x + 2, layout.torso_y + 1, roles.accessory);
        }
        Accessory::Wristband => canvas.set(
            layout.torso_x.saturating_sub(1),
            layout.torso_y + 2,
            roles.accessory,
        ),
        Accessory::Scarf => draw_run(
            canvas,
            layout.torso_x,
            layout.torso_y,
            layout.torso_width,
            roles.accessory,
        ),
        Accessory::ShoulderBag => {
            canvas.set(layout.torso_x + 1, layout.torso_y, roles.accessory);
            canvas.set(layout.torso_x + 2, layout.torso_y + 1, roles.accessory);
            canvas.set(
                layout.torso_x + layout.torso_width - 1,
                layout.torso_y + 3,
                roles.accessory,
            );
        }
    }
}

fn draw_run(canvas: &mut Canvas, x: u16, y: u16, width: u16, role: ColorRole) {
    canvas.fill_rect(x, y, width, 1, role);
}
