use crate::{
    domain::{
        Accessory, BodyProportions, FaceDetail, HairShape, HeadShape, OutfitTop, PersonaAppearance,
    },
    ui::pixel::{Canvas, ColorRole, Palette},
};

use super::appearance::{AppearanceRoles, appearance_roles, appearance_roles_for_palette};

const PROFILE_WIDTH: u16 = 16;
const PROFILE_HEIGHT: u16 = 32;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct ProfileLayout {
    head_x: u16,
    head_y: u16,
    head_width: u16,
    head_height: u16,
    torso_x: u16,
    torso_y: u16,
    torso_width: u16,
    torso_height: u16,
    hip_x: u16,
    hip_y: u16,
    hip_width: u16,
    leg_y: u16,
    leg_height: u16,
    leg_width: u16,
}

impl ProfileLayout {
    const fn for_proportions(proportions: BodyProportions) -> Self {
        match proportions {
            BodyProportions::Compact => Self {
                head_x: 4,
                head_y: 3,
                head_width: 8,
                head_height: 7,
                torso_x: 4,
                torso_y: 11,
                torso_width: 8,
                torso_height: 7,
                hip_x: 5,
                hip_y: 18,
                hip_width: 6,
                leg_y: 21,
                leg_height: 7,
                leg_width: 2,
            },
            BodyProportions::Average => Self {
                head_x: 4,
                head_y: 2,
                head_width: 8,
                head_height: 7,
                torso_x: 4,
                torso_y: 10,
                torso_width: 8,
                torso_height: 9,
                hip_x: 5,
                hip_y: 19,
                hip_width: 6,
                leg_y: 22,
                leg_height: 7,
                leg_width: 2,
            },
            BodyProportions::Tall => Self {
                head_x: 5,
                head_y: 1,
                head_width: 6,
                head_height: 8,
                torso_x: 5,
                torso_y: 10,
                torso_width: 6,
                torso_height: 10,
                hip_x: 5,
                hip_y: 20,
                hip_width: 6,
                leg_y: 22,
                leg_height: 8,
                leg_width: 2,
            },
            BodyProportions::Broad => Self {
                head_x: 4,
                head_y: 2,
                head_width: 8,
                head_height: 7,
                torso_x: 2,
                torso_y: 10,
                torso_width: 12,
                torso_height: 9,
                hip_x: 3,
                hip_y: 19,
                hip_width: 10,
                leg_y: 22,
                leg_height: 7,
                leg_width: 3,
            },
        }
    }
}

pub fn compose_profile(appearance: &PersonaAppearance) -> Canvas {
    compose_profile_with_roles(appearance, appearance_roles(appearance))
}

pub fn compose_profile_for_palette(appearance: &PersonaAppearance, palette: Palette) -> Canvas {
    compose_profile_with_roles(
        appearance,
        appearance_roles_for_palette(appearance, palette),
    )
}

fn compose_profile_with_roles(appearance: &PersonaAppearance, roles: AppearanceRoles) -> Canvas {
    let mut canvas = Canvas::new(PROFILE_WIDTH, PROFILE_HEIGHT);
    let layout = ProfileLayout::for_proportions(appearance.proportions);

    draw_head(&mut canvas, appearance, roles, layout);
    draw_torso_and_arms(&mut canvas, appearance, roles, layout);
    draw_legs_and_shoes(&mut canvas, roles, layout);
    draw_accessory(&mut canvas, appearance.accessory, roles, layout);
    canvas
}

fn draw_head(
    canvas: &mut Canvas,
    appearance: &PersonaAppearance,
    roles: AppearanceRoles,
    layout: ProfileLayout,
) {
    draw_head_shape(canvas, appearance.head_shape, roles, layout);
    draw_hair(canvas, appearance.hair, roles, layout);
    draw_face_detail(canvas, appearance.face_detail, roles, layout);
}

fn draw_head_shape(
    canvas: &mut Canvas,
    shape: HeadShape,
    roles: AppearanceRoles,
    layout: ProfileLayout,
) {
    if matches!(shape, HeadShape::Round | HeadShape::Angular) {
        fill(
            canvas,
            layout.head_x + 1,
            layout.head_y,
            layout.head_width - 2,
            1,
            roles.hair,
        );
        fill(
            canvas,
            layout.head_x,
            layout.head_y + 1,
            layout.head_width,
            layout.head_height - 2,
            roles.skin,
        );
        fill(
            canvas,
            layout.head_x + 1,
            layout.head_y + layout.head_height - 1,
            layout.head_width - 2,
            1,
            roles.skin,
        );
        return;
    }

    fill(
        canvas,
        layout.head_x,
        layout.head_y,
        layout.head_width,
        layout.head_height,
        roles.skin,
    );
    fill(
        canvas,
        layout.head_x,
        layout.head_y,
        layout.head_width,
        1,
        roles.hair,
    );
}

fn draw_hair(canvas: &mut Canvas, hair: HairShape, roles: AppearanceRoles, layout: ProfileLayout) {
    match hair {
        HairShape::Crop | HairShape::Shaved => fill(
            canvas,
            layout.head_x,
            layout.head_y,
            layout.head_width,
            2,
            roles.hair,
        ),
        HairShape::Fringe => {
            fill(
                canvas,
                layout.head_x,
                layout.head_y,
                layout.head_width,
                2,
                roles.hair,
            );
            fill(
                canvas,
                layout.head_x,
                layout.head_y + 2,
                layout.head_width / 2 + 1,
                1,
                roles.hair,
            );
        }
        HairShape::Curls | HairShape::Bob => {
            fill(
                canvas,
                layout.head_x,
                layout.head_y,
                layout.head_width,
                2,
                roles.hair,
            );
            fill(
                canvas,
                layout.head_x.saturating_sub(1),
                layout.head_y + 1,
                2,
                5,
                roles.hair,
            );
            fill(
                canvas,
                layout.head_x + layout.head_width - 1,
                layout.head_y + 1,
                2,
                5,
                roles.hair,
            );
        }
        HairShape::Quiff | HairShape::Spikes => {
            fill(
                canvas,
                layout.head_x + 1,
                layout.head_y,
                layout.head_width - 1,
                2,
                roles.hair,
            );
            canvas.set(
                layout.head_x + 2,
                layout.head_y.saturating_sub(1),
                roles.hair,
            );
            canvas.set(
                layout.head_x + 4,
                layout.head_y.saturating_sub(1),
                roles.hair,
            );
        }
        HairShape::Ponytail => {
            fill(
                canvas,
                layout.head_x,
                layout.head_y,
                layout.head_width,
                2,
                roles.hair,
            );
            fill(
                canvas,
                layout.head_x + layout.head_width,
                layout.head_y + 2,
                2,
                5,
                roles.hair,
            );
        }
    }
}

fn draw_face_detail(
    canvas: &mut Canvas,
    detail: FaceDetail,
    roles: AppearanceRoles,
    layout: ProfileLayout,
) {
    let face_y = layout.head_y + 3;
    match detail {
        FaceDetail::None => canvas.set(layout.head_x + 2, face_y, roles.shadow),
        FaceDetail::RoundGlasses | FaceDetail::SquareGlasses => {
            fill(canvas, layout.head_x + 1, face_y, 2, 2, roles.accent);
            fill(
                canvas,
                layout.head_x + layout.head_width - 3,
                face_y,
                2,
                2,
                roles.accent,
            );
            fill(
                canvas,
                layout.head_x + 3,
                face_y,
                layout.head_width.saturating_sub(6),
                1,
                roles.accent,
            );
        }
        FaceDetail::Visor => fill(
            canvas,
            layout.head_x + 1,
            face_y,
            layout.head_width - 2,
            2,
            roles.accent,
        ),
        FaceDetail::Freckles => {
            canvas.set(layout.head_x + 2, face_y + 1, roles.accent);
            canvas.set(
                layout.head_x + layout.head_width - 3,
                face_y + 1,
                roles.accent,
            );
        }
        FaceDetail::Moustache => fill(
            canvas,
            layout.head_x + layout.head_width / 2 - 1,
            face_y + 2,
            3,
            1,
            roles.hair,
        ),
    }
}

fn draw_torso_and_arms(
    canvas: &mut Canvas,
    appearance: &PersonaAppearance,
    roles: AppearanceRoles,
    layout: ProfileLayout,
) {
    fill(
        canvas,
        layout.torso_x,
        layout.torso_y,
        layout.torso_width,
        layout.torso_height,
        roles.top,
    );
    let arm_width = if appearance.proportions == BodyProportions::Broad {
        2
    } else {
        1
    };
    fill(
        canvas,
        layout.torso_x.saturating_sub(arm_width),
        layout.torso_y + 1,
        arm_width,
        layout.torso_height - 2,
        roles.skin,
    );
    fill(
        canvas,
        layout.torso_x + layout.torso_width,
        layout.torso_y + 1,
        arm_width,
        layout.torso_height - 2,
        roles.skin,
    );

    match appearance.top {
        OutfitTop::StripeJumper | OutfitTop::TrackTop => {
            fill(
                canvas,
                layout.torso_x,
                layout.torso_y + 2,
                layout.torso_width,
                1,
                roles.accent,
            );
            fill(
                canvas,
                layout.torso_x,
                layout.torso_y + 5,
                layout.torso_width,
                1,
                roles.accent,
            );
        }
        OutfitTop::HighCollar => fill(
            canvas,
            layout.torso_x + 1,
            layout.torso_y,
            layout.torso_width - 2,
            2,
            roles.accent,
        ),
        OutfitTop::WorkShirt | OutfitTop::Cardigan | OutfitTop::Waistcoat => fill(
            canvas,
            layout.torso_x + layout.torso_width / 2,
            layout.torso_y,
            1,
            layout.torso_height,
            roles.accent,
        ),
        OutfitTop::BandTee | OutfitTop::Hoodie => fill(
            canvas,
            layout.torso_x + layout.torso_width / 2 - 1,
            layout.torso_y + 3,
            2,
            2,
            roles.accent,
        ),
    }
    fill(
        canvas,
        layout.hip_x,
        layout.hip_y,
        layout.hip_width,
        3,
        roles.bottom,
    );
}

fn draw_legs_and_shoes(canvas: &mut Canvas, roles: AppearanceRoles, layout: ProfileLayout) {
    let left_x = layout.hip_x + 1;
    let right_x = layout.hip_x + layout.hip_width - layout.leg_width - 1;
    fill(
        canvas,
        left_x,
        layout.leg_y,
        layout.leg_width,
        layout.leg_height,
        roles.bottom,
    );
    fill(
        canvas,
        right_x,
        layout.leg_y,
        layout.leg_width,
        layout.leg_height,
        roles.bottom,
    );
    let shoe_y = layout.leg_y + layout.leg_height;
    fill(
        canvas,
        left_x.saturating_sub(1),
        shoe_y,
        layout.leg_width + 2,
        2,
        roles.shoes,
    );
    fill(
        canvas,
        right_x,
        shoe_y,
        layout.leg_width + 2,
        2,
        roles.shoes,
    );
}

fn draw_accessory(
    canvas: &mut Canvas,
    accessory: Accessory,
    roles: AppearanceRoles,
    layout: ProfileLayout,
) {
    match accessory {
        Accessory::Headphones => draw_headphones(canvas, roles.accessory, layout),
        Accessory::Pager => fill(
            canvas,
            layout.torso_x + layout.torso_width - 1,
            layout.torso_y + 4,
            2,
            2,
            roles.accessory,
        ),
        Accessory::Lanyard => {
            canvas.set(
                layout.torso_x + layout.torso_width / 2 - 1,
                layout.torso_y + 1,
                roles.accessory,
            );
            fill(
                canvas,
                layout.torso_x + layout.torso_width / 2,
                layout.torso_y + 2,
                1,
                4,
                roles.accessory,
            );
        }
        Accessory::Wristband => fill(
            canvas,
            layout.torso_x.saturating_sub(1),
            layout.torso_y + 5,
            2,
            1,
            roles.accessory,
        ),
        Accessory::Scarf => {
            fill(
                canvas,
                layout.torso_x,
                layout.torso_y,
                layout.torso_width,
                2,
                roles.accessory,
            );
            fill(
                canvas,
                layout.torso_x + 1,
                layout.torso_y + 2,
                2,
                4,
                roles.accessory,
            );
        }
        Accessory::Badge => fill(
            canvas,
            layout.torso_x + layout.torso_width - 3,
            layout.torso_y + 2,
            2,
            2,
            roles.accessory,
        ),
        Accessory::PocketPen => fill(
            canvas,
            layout.torso_x + layout.torso_width - 2,
            layout.torso_y + 1,
            1,
            3,
            roles.accessory,
        ),
        Accessory::ShoulderBag => draw_shoulder_bag(canvas, roles.accessory, layout),
    }
}

fn draw_headphones(canvas: &mut Canvas, role: ColorRole, layout: ProfileLayout) {
    fill(
        canvas,
        layout.head_x.saturating_sub(1),
        layout.head_y + 2,
        1,
        4,
        role,
    );
    fill(
        canvas,
        layout.head_x + layout.head_width,
        layout.head_y + 2,
        1,
        4,
        role,
    );
}

fn draw_shoulder_bag(canvas: &mut Canvas, role: ColorRole, layout: ProfileLayout) {
    for offset in 0..layout.torso_height.min(7) {
        canvas.set(layout.torso_x + 1 + offset, layout.torso_y + offset, role);
    }
    fill(
        canvas,
        layout.torso_x + layout.torso_width - 2,
        layout.torso_y + layout.torso_height - 3,
        3,
        3,
        role,
    );
}

fn fill(canvas: &mut Canvas, x: u16, y: u16, width: u16, height: u16, role: ColorRole) {
    canvas.fill_rect(x, y, width, height, role);
}
