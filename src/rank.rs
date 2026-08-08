//! The guild's standing: one score for this Questmancer, not one per sprite.
//!
//! Adventurers come and go with panes and sessions, so a level attached to one
//! would reset whenever an agent was recreated. The guild is the thing with
//! continuity, so the guild is what keeps the score.
//!
//! This is the one number Questmancer keeps that is not a fact about your
//! agents, and it is deliberately a flourish rather than an instrument. It
//! unlocks nothing and gates nothing; a tool that withheld features until you
//! had used it enough would be a worse tool.

use crate::app::CharacterSet;

/// A rung on the ladder, and the experience needed to stand on it.
#[derive(Debug)]
pub struct GuildRank {
    pub level: u8,
    pub title: &'static str,
    pub threshold: u64,
}

/// Thresholds follow a doubling curve rather than a flat one, so early
/// progress is quick and later ranks are a genuine accumulation.
pub const RANKS: &[GuildRank] = &[
    GuildRank {
        level: 1,
        title: "Novice",
        threshold: 0,
    },
    GuildRank {
        level: 2,
        title: "Apprentice",
        threshold: 50,
    },
    GuildRank {
        level: 3,
        title: "Journeyman",
        threshold: 150,
    },
    GuildRank {
        level: 4,
        title: "Adept",
        threshold: 350,
    },
    GuildRank {
        level: 5,
        title: "Veteran",
        threshold: 750,
    },
    GuildRank {
        level: 6,
        title: "Master",
        threshold: 1_500,
    },
    GuildRank {
        level: 7,
        title: "Grandmaster",
        threshold: 3_000,
    },
    GuildRank {
        level: 8,
        title: "Guildlord",
        threshold: 6_000,
    },
];

/// The highest rank this much experience has earned.
#[must_use]
pub fn rank_for(experience: u64) -> &'static GuildRank {
    RANKS
        .iter()
        .rev()
        .find(|rank| experience >= rank.threshold)
        .unwrap_or(&RANKS[0])
}

/// Experience still owed before the next rung, and what that rung is.
///
/// `None` at the top of the ladder: there is nothing left to owe, and a
/// progress bar that never fills would be worse than no bar.
#[must_use]
pub fn next_rank(experience: u64) -> Option<&'static GuildRank> {
    RANKS.iter().find(|rank| rank.threshold > experience)
}

/// The badge that sits in the corner of the room.
///
/// Compact on purpose: it is permanent chrome over a scene that is the point,
/// so it earns a single short line and no more.
#[must_use]
pub fn badge(experience: u64, character_set: CharacterSet) -> String {
    let rank = rank_for(experience);
    let glyph = match character_set {
        CharacterSet::Unicode => "❖",
        CharacterSet::Ascii => "*",
    };
    format!("{glyph} {} · {experience} xp", rank.title)
}

/// The fuller reading for the Librarian's Ledger.
#[must_use]
pub fn ledger_lines(experience: u64) -> Vec<String> {
    let rank = rank_for(experience);
    let mut lines = vec![
        format!("Rank {} · {}", rank.level, rank.title),
        format!("{experience} experience earned"),
    ];
    match next_rank(experience) {
        Some(next) => lines.push(format!(
            "{} more to reach {}",
            next.threshold - experience,
            next.title
        )),
        None => lines.push("The ladder has no higher rung.".to_owned()),
    }
    lines
}
