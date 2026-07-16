use crate::{
    domain::{
        AdventurerClass, AdventurerPersona, AdventuringGear, Ancestry, Garb, HairShape, Keepsake,
        PersonaAppearance,
    },
    ui::pixel::{Canvas, ColorRole, Palette},
};

use super::{
    appearance::{AppearanceRoles, appearance_roles_for_palette},
    state_pose::BodyPose,
};

const PROFILE_WIDTH: u16 = 16;
const PROFILE_HEIGHT: u16 = 32;

#[derive(Clone, Copy)]
struct ProfileLayout {
    head_x: u16,
    head_y: u16,
    head_width: u16,
    torso_x: u16,
    torso_y: u16,
    torso_width: u16,
    torso_height: u16,
    hips_y: u16,
    legs_y: u16,
    legs_height: u16,
    shoes_y: u16,
}

impl ProfileLayout {
    const fn for_pose(pose: BodyPose) -> Self {
        if pose.compact() {
            Self {
                head_x: 4,
                head_y: 5,
                head_width: 8,
                torso_x: if pose.broad() { 2 } else { 4 },
                torso_y: 13,
                torso_width: if pose.broad() { 12 } else { 8 },
                torso_height: 6,
                hips_y: 19,
                legs_y: 22,
                legs_height: 4,
                shoes_y: 26,
            }
        } else {
            Self {
                head_x: if pose.broad() { 4 } else { 5 },
                head_y: 2,
                head_width: if pose.broad() { 8 } else { 6 },
                torso_x: if pose.broad() { 2 } else { 4 },
                torso_y: 10,
                torso_width: if pose.broad() { 12 } else { 8 },
                torso_height: 9,
                hips_y: 19,
                legs_y: 22,
                legs_height: 7,
                shoes_y: 29,
            }
        }
    }
}

pub fn compose_profile_adventurer(persona: &AdventurerPersona) -> Canvas {
    compose_profile_adventurer_for_palette(persona, Palette::Xterm256)
}

pub fn compose_profile_adventurer_for_palette(
    persona: &AdventurerPersona,
    palette: Palette,
) -> Canvas {
    let body_pose = BodyPose::profile(persona.ancestry, persona.appearance.proportions);
    let mut canvas = compose_body(&persona.appearance, body_pose, palette);
    let roles = appearance_roles_for_palette(&persona.appearance, palette);
    overlay_ancestry(&mut canvas, persona.ancestry, roles, body_pose);
    overlay_class_gear(&mut canvas, persona.class, persona.class.gear());
    overlay_keepsake(&mut canvas, persona.appearance.keepsake, roles);
    canvas
}

fn compose_body(appearance: &PersonaAppearance, pose: BodyPose, palette: Palette) -> Canvas {
    let roles = appearance_roles_for_palette(appearance, palette);
    let layout = ProfileLayout::for_pose(pose);
    let mut canvas = Canvas::new(PROFILE_WIDTH, PROFILE_HEIGHT);

    draw_head(&mut canvas, appearance.hair, roles, layout);
    draw_body(&mut canvas, appearance.garb, roles, layout, pose);
    canvas
}

fn draw_head(canvas: &mut Canvas, hair: HairShape, roles: AppearanceRoles, layout: ProfileLayout) {
    canvas.fill_rect(
        layout.head_x,
        layout.head_y,
        layout.head_width,
        2,
        roles.hair,
    );
    canvas.fill_rect(
        layout.head_x,
        layout.head_y + 2,
        layout.head_width,
        5,
        roles.skin,
    );
    match hair {
        HairShape::Curls | HairShape::Bob => {
            canvas.fill_rect(
                layout.head_x.saturating_sub(1),
                layout.head_y + 1,
                1,
                5,
                roles.hair,
            );
            canvas.fill_rect(
                layout.head_x + layout.head_width,
                layout.head_y + 1,
                1,
                5,
                roles.hair,
            );
        }
        HairShape::Quiff | HairShape::Spikes => {
            canvas.set(layout.head_x + 1, layout.head_y - 1, roles.hair);
            canvas.set(layout.head_x + 3, layout.head_y - 1, roles.hair);
        }
        HairShape::Ponytail => canvas.fill_rect(
            layout.head_x + layout.head_width,
            layout.head_y + 2,
            2,
            6,
            roles.hair,
        ),
        HairShape::Fringe => canvas.fill_rect(
            layout.head_x,
            layout.head_y + 2,
            layout.head_width / 2 + 1,
            1,
            roles.hair,
        ),
        HairShape::Crop | HairShape::Shaved => {}
    }
}

fn draw_body(
    canvas: &mut Canvas,
    garb: Garb,
    roles: AppearanceRoles,
    layout: ProfileLayout,
    pose: BodyPose,
) {
    canvas.fill_rect(
        layout.torso_x,
        layout.torso_y,
        layout.torso_width,
        layout.torso_height,
        roles.garb,
    );
    draw_garb(canvas, garb, roles, layout);
    let arm_width = if pose.broad() { 2 } else { 1 };
    canvas.fill_rect(
        layout.torso_x.saturating_sub(arm_width),
        layout.torso_y + 1,
        arm_width,
        layout.torso_height.saturating_sub(2),
        roles.skin,
    );
    canvas.fill_rect(
        layout.torso_x + layout.torso_width,
        layout.torso_y + 1,
        arm_width,
        layout.torso_height.saturating_sub(2),
        roles.skin,
    );
    canvas.fill_rect(
        layout.torso_x + 1,
        layout.hips_y,
        layout.torso_width.saturating_sub(2),
        3,
        roles.legwear,
    );
    let left_leg = layout.torso_x + 2;
    let right_leg = layout.torso_x + layout.torso_width - 3;
    canvas.fill_rect(
        left_leg,
        layout.legs_y,
        2,
        layout.legs_height,
        roles.legwear,
    );
    canvas.fill_rect(
        right_leg,
        layout.legs_y,
        2,
        layout.legs_height,
        roles.legwear,
    );
    canvas.fill_rect(
        left_leg.saturating_sub(1),
        layout.shoes_y,
        4,
        2,
        roles.footwear,
    );
    canvas.fill_rect(right_leg, layout.shoes_y, 4, 2, roles.footwear);
}

fn draw_garb(canvas: &mut Canvas, garb: Garb, roles: AppearanceRoles, layout: ProfileLayout) {
    match garb {
        Garb::Armour => canvas.fill_rect(
            layout.torso_x + 1,
            layout.torso_y,
            layout.torso_width.saturating_sub(2),
            2,
            roles.accent,
        ),
        Garb::Cloak | Garb::Robes => {
            canvas.fill_rect(
                layout.torso_x,
                layout.torso_y + 2,
                layout.torso_width,
                1,
                roles.accent,
            );
            canvas.fill_rect(
                layout.torso_x,
                layout.torso_y + layout.torso_height - 2,
                layout.torso_width,
                1,
                roles.accent,
            );
        }
        Garb::Doublet | Garb::Vestments | Garb::WorkApron => canvas.fill_rect(
            layout.torso_x + layout.torso_width / 2,
            layout.torso_y,
            1,
            layout.torso_height,
            roles.accent,
        ),
        Garb::Leathers => canvas.fill_rect(
            layout.torso_x + 1,
            layout.torso_y + 3,
            layout.torso_width.saturating_sub(2),
            1,
            roles.accent,
        ),
    }
}

fn overlay_keepsake(canvas: &mut Canvas, keepsake: Keepsake, roles: AppearanceRoles) {
    match keepsake {
        Keepsake::Feather => {
            canvas.set(2, 0, roles.keepsake);
            canvas.set(3, 1, roles.keepsake);
            canvas.set(4, 2, roles.keepsake);
        }
        Keepsake::LuckyCoin => canvas.fill_rect(12, 19, 2, 2, roles.keepsake),
        Keepsake::Mug => {
            canvas.fill_rect(12, 13, 2, 2, roles.keepsake);
            canvas.set(14, 14, roles.keepsake);
        }
        Keepsake::PressedLeaf => {
            canvas.set(3, 7, roles.keepsake);
            canvas.set(2, 8, roles.keepsake);
            canvas.set(3, 8, roles.keepsake);
            canvas.set(4, 8, roles.keepsake);
            canvas.set(3, 9, roles.keepsake);
            canvas.set(3, 10, roles.keepsake);
        }
        Keepsake::Ribbon => {
            canvas.set(12, 2, roles.keepsake);
            canvas.set(11, 3, roles.keepsake);
            canvas.set(12, 3, roles.keepsake);
            canvas.set(11, 4, roles.keepsake);
            canvas.set(12, 4, roles.keepsake);
            canvas.set(11, 5, roles.keepsake);
        }
        Keepsake::TinyFamiliar => {
            canvas.set(1, 25, roles.keepsake);
            canvas.fill_rect(0, 26, 3, 1, roles.keepsake);
            canvas.fill_rect(2, 27, 1, 4, roles.keepsake);
            canvas.fill_rect(2, 30, 4, 1, roles.keepsake);
            canvas.set(2, 31, roles.keepsake);
        }
    }
}

fn overlay_ancestry(
    canvas: &mut Canvas,
    ancestry: Ancestry,
    roles: AppearanceRoles,
    pose: BodyPose,
) {
    let layout = ProfileLayout::for_pose(pose);
    let face_y = layout.head_y + 3;
    match ancestry {
        Ancestry::Human => canvas.set(layout.head_x.saturating_sub(1), face_y, roles.skin),
        Ancestry::Dwarf => {
            canvas.fill_rect(
                layout.head_x + 1,
                face_y + 2,
                layout.head_width - 2,
                4,
                roles.hair,
            );
            canvas.fill_rect(
                layout.head_x.saturating_sub(1),
                face_y + 3,
                2,
                2,
                roles.hair,
            );
        }
        Ancestry::Elf => {
            canvas.fill_rect(layout.head_x.saturating_sub(2), face_y, 2, 1, roles.skin);
            canvas.fill_rect(layout.head_x + layout.head_width, face_y, 2, 1, roles.skin);
        }
        Ancestry::Halfling => {
            canvas.fill_rect(
                layout.head_x.saturating_sub(1),
                face_y + 1,
                1,
                3,
                roles.hair,
            );
            canvas.fill_rect(
                layout.head_x + layout.head_width,
                face_y + 1,
                1,
                3,
                roles.hair,
            );
            canvas.set(layout.torso_x, layout.shoes_y + 2, roles.footwear);
            canvas.set(
                layout.torso_x + layout.torso_width - 1,
                layout.shoes_y + 2,
                roles.footwear,
            );
        }
        Ancestry::Orc => {
            canvas.set(layout.head_x.saturating_sub(2), face_y + 2, roles.highlight);
            canvas.set(
                layout.head_x + layout.head_width + 1,
                face_y + 2,
                roles.highlight,
            );
        }
        Ancestry::Gnome => {
            canvas.set(
                layout.head_x + layout.head_width / 2,
                layout.head_y - 3,
                roles.accent,
            );
            canvas.fill_rect(
                layout.head_x + 2,
                layout.head_y - 2,
                layout.head_width.saturating_sub(3),
                1,
                roles.accent,
            );
            canvas.fill_rect(
                layout.head_x,
                layout.head_y - 1,
                layout.head_width,
                1,
                roles.accent,
            );
        }
        Ancestry::Goblin => {
            canvas.fill_rect(
                layout.head_x.saturating_sub(3),
                face_y,
                3,
                1,
                ColorRole::Goblin,
            );
            canvas.fill_rect(
                layout.head_x + layout.head_width,
                face_y,
                3,
                1,
                ColorRole::Goblin,
            );
        }
    }
}

fn overlay_class_gear(canvas: &mut Canvas, class: AdventurerClass, gear: AdventuringGear) {
    debug_assert_eq!(class.gear(), gear);
    match gear {
        AdventuringGear::Axe => {
            canvas.fill_rect(14, 8, 1, 20, ColorRole::Leather);
            canvas.fill_rect(11, 8, 3, 4, ColorRole::Steel);
        }
        AdventuringGear::BowAndQuiver => {
            canvas.fill_rect(0, 6, 1, 22, ColorRole::Leather);
            canvas.set(1, 6, ColorRole::Leather);
            canvas.set(3, 17, ColorRole::Leather);
            canvas.set(1, 27, ColorRole::Leather);
        }
        AdventuringGear::HolySymbol => {
            canvas.fill_rect(13, 9, 1, 8, ColorRole::Counsel);
            canvas.fill_rect(11, 12, 4, 1, ColorRole::Counsel);
        }
        AdventuringGear::Lute => {
            canvas.fill_rect(0, 13, 2, 7, ColorRole::Timber);
            canvas.fill_rect(2, 13, 1, 7, ColorRole::Leather);
            canvas.fill_rect(2, 8, 1, 5, ColorRole::Leather);
            canvas.set(1, 20, ColorRole::Leather);
        }
        AdventuringGear::MapAndCompass => {
            canvas.fill_rect(0, 8, 4, 7, ColorRole::Parchment);
            canvas.set(4, 14, ColorRole::Spoils);
        }
        AdventuringGear::RuneChisel => {
            for (x, y) in [(15, 6), (14, 7), (13, 8), (14, 9), (13, 10), (13, 11)] {
                canvas.set(x, y, ColorRole::RuneGlow);
            }
            canvas.fill_rect(12, 12, 1, 8, ColorRole::Steel);
        }
        AdventuringGear::Shield => {
            canvas.fill_rect(0, 11, 4, 12, ColorRole::Steel);
            canvas.fill_rect(1, 23, 2, 2, ColorRole::Steel);
        }
        AdventuringGear::SpellbookAndStaff => {
            canvas.fill_rect(15, 3, 1, 27, ColorRole::Leather);
            canvas.fill_rect(11, 15, 4, 5, ColorRole::Parchment);
        }
        AdventuringGear::TestKit => {
            canvas.fill_rect(0, 24, 5, 6, ColorRole::Leather);
            canvas.fill_rect(1, 23, 3, 1, ColorRole::Steel);
        }
        AdventuringGear::ThievesTools => {
            for (x, y) in [(0, 11), (1, 13), (0, 15), (1, 17), (0, 19), (1, 21)] {
                canvas.set(x, y, ColorRole::Steel);
            }
        }
        AdventuringGear::Toolkit => {
            canvas.fill_rect(12, 23, 4, 7, ColorRole::Leather);
            canvas.fill_rect(13, 21, 2, 2, ColorRole::Steel);
        }
    }
}
