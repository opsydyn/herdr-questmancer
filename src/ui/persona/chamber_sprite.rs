use crate::{
    domain::{
        AdventurerClass, AdventurerPersona, AdventuringGear, Ancestry, Garb, HairShape, Keepsake,
        PersonaAppearance,
    },
    ui::{
        pixel::{Canvas, ColorRole, Palette},
        theatre::{TheatreFrame, TheatrePose},
    },
};

use super::{
    appearance::{AppearanceRoles, appearance_roles_for_palette},
    state_pose::BodyPose,
};

const CHAMBER_WIDTH: u16 = 10;
const CHAMBER_HEIGHT: u16 = 12;

pub fn compose_chamber_adventurer(persona: &AdventurerPersona, theatre: TheatreFrame) -> Canvas {
    compose_chamber_adventurer_for_palette(persona, theatre, Palette::Xterm256)
}

pub fn compose_chamber_adventurer_for_palette(
    persona: &AdventurerPersona,
    theatre: TheatreFrame,
    palette: Palette,
) -> Canvas {
    let body_pose = BodyPose::chamber(persona.ancestry, persona.appearance.proportions);
    let mut canvas = Canvas::new(CHAMBER_WIDTH, CHAMBER_HEIGHT);

    if theatre.pose != TheatrePose::Departed {
        canvas = compose_body(&persona.appearance, body_pose, palette);
        let roles = appearance_roles_for_palette(&persona.appearance, palette);
        overlay_ancestry(&mut canvas, persona.ancestry, roles, body_pose);
        overlay_class_gear(&mut canvas, persona.class, persona.class.gear());
        overlay_keepsake(&mut canvas, persona.appearance.keepsake, roles);
    }
    overlay_state_prop(&mut canvas, theatre.pose, theatre.animation_frame);
    canvas
}

fn compose_body(appearance: &PersonaAppearance, pose: BodyPose, palette: Palette) -> Canvas {
    let roles = appearance_roles_for_palette(appearance, palette);
    let mut canvas = Canvas::new(CHAMBER_WIDTH, CHAMBER_HEIGHT);
    let compact = pose.compact();
    let broad = pose.broad();
    let head_x = if broad { 2 } else { 3 };
    let head_y = u16::from(compact);
    let head_width = if broad { 6 } else { 4 };
    let torso_x = if broad { 2 } else { 3 };
    let torso_y = if compact { 5 } else { 4 };
    let torso_width = if broad { 6 } else { 4 };
    let torso_height = if compact { 3 } else { 4 };

    canvas.fill_rect(head_x, head_y, head_width, 1, roles.hair);
    canvas.fill_rect(head_x, head_y + 1, head_width, 3, roles.skin);
    match appearance.hair {
        HairShape::Curls | HairShape::Bob => {
            canvas.set(head_x.saturating_sub(1), head_y + 2, roles.hair);
            canvas.set(head_x + head_width, head_y + 2, roles.hair);
        }
        HairShape::Quiff | HairShape::Spikes => {
            canvas.set(head_x + 1, head_y.saturating_sub(1), roles.hair);
        }
        HairShape::Ponytail => {
            canvas.set(head_x + head_width, head_y + 2, roles.hair);
            canvas.set(head_x + head_width, head_y + 3, roles.hair);
        }
        HairShape::Fringe => canvas.fill_rect(head_x, head_y + 1, 2, 1, roles.hair),
        HairShape::Crop | HairShape::Shaved => {}
    }

    canvas.fill_rect(torso_x, torso_y, torso_width, torso_height, roles.garb);
    draw_garb(
        &mut canvas,
        appearance.garb,
        roles,
        torso_x,
        torso_y,
        torso_width,
    );
    canvas.set(torso_x.saturating_sub(1), torso_y + 1, roles.skin);
    canvas.set(torso_x + torso_width, torso_y + 1, roles.skin);

    let hips_y = torso_y + torso_height;
    canvas.fill_rect(torso_x, hips_y, torso_width, 1, roles.legwear);
    let left_leg = if broad { 2 } else { 3 };
    let right_leg = 6;
    canvas.fill_rect(left_leg, hips_y + 1, 1, 2, roles.legwear);
    canvas.fill_rect(right_leg, hips_y + 1, 1, 2, roles.legwear);
    canvas.fill_rect(left_leg.saturating_sub(1), 11, 2, 1, roles.footwear);
    canvas.fill_rect(right_leg, 11, 2, 1, roles.footwear);
    canvas
}

fn draw_garb(canvas: &mut Canvas, garb: Garb, roles: AppearanceRoles, x: u16, y: u16, width: u16) {
    match garb {
        Garb::Armour => canvas.fill_rect(x + 1, y, width.saturating_sub(2), 1, roles.accent),
        Garb::Cloak | Garb::Robes => canvas.fill_rect(x, y + 2, width, 1, roles.accent),
        Garb::Doublet | Garb::Vestments | Garb::WorkApron => {
            canvas.fill_rect(x + width / 2, y, 1, 3, roles.accent);
        }
        Garb::Leathers => canvas.set(x + width / 2, y + 1, roles.accent),
    }
}

fn overlay_keepsake(canvas: &mut Canvas, keepsake: Keepsake, roles: AppearanceRoles) {
    match keepsake {
        Keepsake::Feather => {
            canvas.set(4, 0, roles.keepsake);
            canvas.set(5, 0, roles.keepsake);
            canvas.set(4, 1, roles.keepsake);
        }
        Keepsake::LuckyCoin => {
            canvas.set(8, 11, roles.keepsake);
            canvas.set(9, 11, roles.keepsake);
        }
        Keepsake::Mug => {
            canvas.fill_rect(8, 5, 2, 2, roles.keepsake);
            canvas.set(7, 6, roles.keepsake);
        }
        Keepsake::PressedLeaf => {
            canvas.set(0, 0, roles.keepsake);
            canvas.set(0, 1, roles.keepsake);
            canvas.set(1, 2, roles.keepsake);
            canvas.set(1, 3, roles.keepsake);
            canvas.set(1, 4, roles.keepsake);
            canvas.set(1, 5, roles.keepsake);
            canvas.set(2, 6, roles.keepsake);
            canvas.set(1, 7, roles.keepsake);
        }
        Keepsake::Ribbon => {
            canvas.set(8, 0, roles.keepsake);
            canvas.set(8, 1, roles.keepsake);
            canvas.set(7, 1, roles.keepsake);
            canvas.set(7, 2, roles.keepsake);
        }
        Keepsake::TinyFamiliar => {
            canvas.set(1, 7, roles.keepsake);
            canvas.fill_rect(0, 8, 3, 1, roles.keepsake);
            canvas.set(2, 9, roles.keepsake);
        }
    }
}

fn overlay_ancestry(
    canvas: &mut Canvas,
    ancestry: Ancestry,
    roles: AppearanceRoles,
    pose: BodyPose,
) {
    let face_y = if pose.compact() { 3 } else { 2 };
    match ancestry {
        Ancestry::Human => canvas.set(2, face_y, roles.skin),
        Ancestry::Dwarf => {
            canvas.fill_rect(3, face_y + 1, 4, 2, roles.hair);
            canvas.set(2, face_y + 1, roles.hair);
        }
        Ancestry::Elf => {
            canvas.set(1, face_y, roles.skin);
            canvas.set(2, face_y, roles.skin);
            canvas.set(7, face_y, roles.skin);
            canvas.set(8, face_y, roles.skin);
        }
        Ancestry::Halfling => {
            canvas.set(2, face_y + 1, roles.hair);
            canvas.set(7, face_y + 1, roles.hair);
            canvas.set(1, 11, roles.footwear);
            canvas.set(8, 11, roles.footwear);
        }
        Ancestry::Orc => {
            canvas.set(2, face_y + 1, roles.highlight);
            canvas.set(7, face_y + 1, roles.highlight);
        }
        Ancestry::Gnome => {
            canvas.set(4, 0, roles.accent);
            canvas.fill_rect(3, 1, 3, 1, roles.accent);
        }
        Ancestry::Goblin => {
            canvas.set(1, face_y, ColorRole::Goblin);
            canvas.set(2, face_y, ColorRole::Goblin);
            canvas.set(7, face_y, ColorRole::Goblin);
            canvas.set(8, face_y, ColorRole::Goblin);
        }
    }
}

fn overlay_class_gear(canvas: &mut Canvas, class: AdventurerClass, gear: AdventuringGear) {
    debug_assert_eq!(class.gear(), gear);
    match gear {
        AdventuringGear::Axe => {
            canvas.fill_rect(9, 3, 1, 8, ColorRole::Leather);
            canvas.fill_rect(7, 3, 2, 2, ColorRole::Steel);
        }
        AdventuringGear::BowAndQuiver => {
            canvas.fill_rect(0, 2, 1, 9, ColorRole::Leather);
            canvas.set(1, 2, ColorRole::Leather);
            canvas.set(2, 6, ColorRole::Leather);
            canvas.set(1, 10, ColorRole::Leather);
        }
        AdventuringGear::HolySymbol => {
            canvas.fill_rect(8, 3, 1, 4, ColorRole::Counsel);
            canvas.fill_rect(7, 4, 3, 1, ColorRole::Counsel);
        }
        AdventuringGear::Lute => {
            canvas.set(1, 3, ColorRole::Leather);
            canvas.fill_rect(0, 4, 1, 4, ColorRole::Timber);
            canvas.fill_rect(1, 4, 1, 4, ColorRole::Leather);
            canvas.set(1, 8, ColorRole::Leather);
        }
        AdventuringGear::MapAndCompass => {
            canvas.fill_rect(0, 2, 3, 3, ColorRole::Parchment);
            canvas.set(2, 5, ColorRole::Spoils);
        }
        AdventuringGear::RuneChisel => {
            canvas.set(9, 2, ColorRole::RuneGlow);
            canvas.set(8, 3, ColorRole::RuneGlow);
            canvas.set(9, 4, ColorRole::RuneGlow);
            canvas.set(8, 5, ColorRole::Steel);
            canvas.set(8, 6, ColorRole::Steel);
        }
        AdventuringGear::Shield => {
            canvas.fill_rect(0, 4, 2, 5, ColorRole::Steel);
            canvas.set(1, 9, ColorRole::Steel);
        }
        AdventuringGear::SpellbookAndStaff => {
            canvas.fill_rect(9, 1, 1, 10, ColorRole::Leather);
            canvas.fill_rect(7, 6, 2, 2, ColorRole::Parchment);
        }
        AdventuringGear::TestKit => {
            canvas.fill_rect(0, 9, 3, 2, ColorRole::Leather);
            canvas.set(1, 8, ColorRole::Steel);
        }
        AdventuringGear::ThievesTools => {
            for (x, y) in [(0, 4), (1, 5), (0, 6), (1, 7), (0, 8)] {
                canvas.set(x, y, ColorRole::Steel);
            }
        }
        AdventuringGear::Toolkit => {
            canvas.fill_rect(7, 8, 3, 3, ColorRole::Leather);
            canvas.set(8, 7, ColorRole::Steel);
        }
    }
}

fn overlay_state_prop(canvas: &mut Canvas, pose: TheatrePose, frame: u8) {
    match pose {
        TheatrePose::Delving => {
            canvas.set(4, 10, ColorRole::RuneGlow);
            canvas.set(
                if frame.is_multiple_of(2) { 8 } else { 7 },
                11,
                ColorRole::RuneGlow,
            );
        }
        TheatrePose::SeekingCounsel => {
            canvas.fill_rect(8, 0, 2, 2, ColorRole::Counsel);
            canvas.set(9, 2, ColorRole::Counsel);
            canvas.fill_rect(0, 9, 2, 3, ColorRole::Stone);
        }
        TheatrePose::SpoilsUnopened => {
            canvas.fill_rect(0, 9, 4, 3, ColorRole::Spoils);
            if (1..=8).contains(&frame) {
                const SPARKLE: [(u16, u16); 8] = [
                    (7, 9),
                    (1, 8),
                    (4, 9),
                    (5, 9),
                    (4, 10),
                    (5, 10),
                    (4, 11),
                    (5, 11),
                ];
                let (x, y) = SPARKLE[usize::from(frame - 1)];
                canvas.set(x, y, ColorRole::Parchment);
            }
        }
        TheatrePose::VictoryRecorded => {
            canvas.fill_rect(0, 0, 1, 5, ColorRole::Selection);
            canvas.fill_rect(1, 0, 3, 2, ColorRole::Selection);
            canvas.fill_rect(7, 9, 3, 2, ColorRole::Parchment);
        }
        TheatrePose::Resting => {
            canvas.set(0, 10, ColorRole::Timber);
            canvas.set(2, 10, ColorRole::Timber);
            canvas.fill_rect(1, 9, 1, 3, ColorRole::Hearth);
        }
        TheatrePose::Departed => {
            canvas.fill_rect(1, 2, 1, 9, ColorRole::Stone);
            canvas.fill_rect(8, 2, 1, 9, ColorRole::Stone);
            canvas.fill_rect(2, 1, 6, 1, ColorRole::Stone);
            canvas.fill_rect(1, 11, 8, 1, ColorRole::Timber);
        }
        TheatrePose::Unknown => {
            for (x, y) in [(0, 1), (2, 5), (8, 2), (9, 7), (1, 10), (8, 11)] {
                canvas.set(x, y, ColorRole::Fog);
            }
        }
    }
}
