//! On-demand, local recaps of retained event evidence. No stored achievements.
use std::collections::BTreeSet;

use crate::domain::{AgentKey, Chronicle, ChronicleEntry, ChronicleEvent, Timestamp};

pub const CHAPTER_COLUMNS: u16 = 76;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ChapterWindow {
    from: Timestamp,
    through: Timestamp,
}

impl ChapterWindow {
    pub const fn last_hour(through: Timestamp) -> Self {
        Self {
            from: Timestamp::from_millis(through.as_millis().saturating_sub(3_600_000)),
            through,
        }
    }

    pub const fn from(self) -> Timestamp {
        self.from
    }
    pub const fn through(self) -> Timestamp {
        self.through
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ChapterRequest {
    pub window: ChapterWindow,
    /// None requests the whole guild. Selection is captured when requested.
    pub adventurer: Option<AgentKey>,
}

#[derive(Debug)]
pub struct ChronicleChapter<'a> {
    request: &'a ChapterRequest,
    sources: Vec<&'a ChronicleEntry>,
}

impl ChapterRequest {
    pub fn project<'a>(&'a self, chronicle: &'a Chronicle) -> ChronicleChapter<'a> {
        let mut seen = BTreeSet::new();
        let mut sources = chronicle
            .entries()
            .iter()
            .filter(|entry| {
                entry.occurred_at >= self.window.from
                    && entry.occurred_at <= self.window.through
                    && self
                        .adventurer
                        .as_ref()
                        .is_none_or(|key| entry.adventurer.as_ref() == Some(key))
                    && seen.insert(&entry.id)
            })
            .collect::<Vec<_>>();
        // Source order is independent of arrival order, including timestamp ties.
        sources.sort_by(|left, right| {
            right
                .occurred_at
                .cmp(&left.occurred_at)
                .then_with(|| left.id.cmp(&right.id))
        });
        ChronicleChapter {
            request: self,
            sources,
        }
    }
}

impl<'a> ChronicleChapter<'a> {
    pub fn sources(&self) -> &[&'a ChronicleEntry] {
        &self.sources
    }

    pub fn count(&self, event: ChronicleEvent) -> usize {
        self.sources
            .iter()
            .filter(|entry| entry.event == event)
            .count()
    }

    pub fn lines(&self) -> Vec<String> {
        let mut lines = vec![
            format!("From: {}", timestamp(self.request.window.from)),
            format!("Through: {}", timestamp(self.request.window.through)),
            format!(
                "Scope: {}",
                self.request
                    .adventurer
                    .as_ref()
                    .map_or("whole guild", AgentKey::as_str)
            ),
            "Retained records only; missing history is not inferred.".to_owned(),
            String::new(),
        ];
        if self.sources.is_empty() {
            lines.push("No retained events in this window.".to_owned());
            return lines.into_iter().flat_map(wrap_line).collect();
        }
        for event in ChronicleEvent::ALL {
            let count = self.count(*event);
            if count > 0 {
                lines.push(format!(
                    "{count} {} event{} recorded.",
                    event_noun(*event),
                    if count == 1 { "" } else { "s" }
                ));
            }
        }
        lines.push(String::new());
        lines.push("Sources (newest first):".to_owned());
        for source in &self.sources {
            lines.push(format!(
                "{} {}",
                source.event.sigil(),
                timestamp(source.occurred_at)
            ));
            // Preserve recorded names in summaries instead of relabelling history
            // from today's live topology. Empty summaries still expose event/IDs.
            let summary = if source.summary.trim().is_empty() {
                if source.event == ChronicleEvent::AdventurerJoined {
                    "identity event (summary unavailable)"
                } else {
                    source.event.label()
                }
            } else {
                &source.summary
            };
            lines.extend(summary.lines().map(str::to_owned));
            lines.push(format!("Source: {}", source.id.as_str()));
            lines.push(String::new());
        }
        let _ = lines.pop();
        lines.into_iter().flat_map(wrap_line).collect()
    }
}

fn wrap_line(text: String) -> Vec<String> {
    if ratatui::text::Line::from(text.as_str()).width() <= usize::from(CHAPTER_COLUMNS - 2) {
        return vec![text];
    }
    let mut lines = Vec::new();
    let mut line = String::new();
    for character in text.chars() {
        let prior = line.len();
        line.push(character);
        if ratatui::text::Line::from(line.as_str()).width() > usize::from(CHAPTER_COLUMNS - 2)
            && prior > 0
        {
            line.truncate(prior);
            lines.push(std::mem::take(&mut line));
            line.push(character);
        }
    }
    lines.push(line);
    lines
}

const fn event_noun(event: ChronicleEvent) -> &'static str {
    match event {
        ChronicleEvent::AdventurerJoined => "identity",
        ChronicleEvent::DelveBegan => "delve-start",
        ChronicleEvent::CounselRequested => "counsel-request",
        ChronicleEvent::SpoilsReturned => "spoils-return",
        ChronicleEvent::AdventurerRested => "rest",
        ChronicleEvent::AdventurerDeparted => "departure",
        ChronicleEvent::CampaignClosed => "campaign-closure",
    }
}

fn timestamp(at: Timestamp) -> String {
    time::OffsetDateTime::from_unix_timestamp_nanos(i128::from(at.as_millis()) * 1_000_000)
        .map_or_else(
            |_| format!("epoch ms {}", at.as_millis()),
            |date| {
                format!(
                    "{:04}-{:02}-{:02} {:02}:{:02}:{:02} UTC",
                    date.year(),
                    u8::from(date.month()),
                    date.day(),
                    date.hour(),
                    date.minute(),
                    date.second()
                )
            },
        )
}
