//! Interactive preview of a quest-centred inbox using fixed sample evidence.
use crossterm::event::{Event, KeyCode, KeyEventKind};
use ratatui::{
    Frame,
    layout::Rect,
    style::Modifier,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
};

use crate::ui::{
    theme::{PARCHMENT, PARCHMENT_BORDER},
    work_board::wrap_text,
};

pub(super) const STORY_ID: &str = "interaction.quest-inbox";

#[derive(Clone, Copy)]
enum Check {
    Passed {
        revision: &'static str,
        result: &'static str,
    },
    Running,
}

struct Quest {
    title: &'static str,
    campaign: &'static str,
    acceptance: &'static str,
    revision: &'static str,
    diff: &'static str,
    check: Check,
    pull_request: &'static str,
    report: &'static str,
}

const QUESTS: [Quest; 3] = [
    Quest {
        title: "Keep counsel visible in a full Hall",
        campaign: "The Guild Hall",
        acceptance: "A crowded Hall keeps the counsel cue legible and preserves actor selection.",
        revision: "c7409e1",
        diff: "party.rs adds a side position for a covered counsel cue. world_fit.rs exercises the crowded case.",
        check: Check::Passed {
            revision: "c7409e1",
            result: "20 renderer tests passed",
        },
        pull_request: "#41 · Open",
        report: "Pi implemented the cue. Codex reviewed the fallback position and actor hit mapping on c7409e1.",
    },
    Quest {
        title: "Walk to the Delve station",
        campaign: "The Delve",
        acceptance: "A fresh Working report moves the adventurer along a clear aisle. Selection follows the sprite.",
        revision: "781b96e",
        diff: "room_study.rs reserves clearance around neighbouring adventurers before starting the walk.",
        check: Check::Passed {
            revision: "55d18a7",
            result: "19 renderer tests passed",
        },
        pull_request: "#42 · Open",
        report: "Pi narrowed the route on 781b96e. Codex's test report covers the earlier 55d18a7 revision.",
    },
    Quest {
        title: "Restore the party name on restart",
        campaign: "The Guild Hall",
        acceptance: "The saved party name returns on restart and a second pane cannot overwrite it.",
        revision: "ab8c421",
        diff: "The profile test opens two panes against the same state directory and restarts the writer.",
        check: Check::Running,
        pull_request: "#43 · Open",
        report: "Codex supplied a restart test. The result is pending.",
    },
];

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
enum Attention {
    #[default]
    Unseen,
    Seen,
    Snoozed,
    Settled(&'static str),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Readiness {
    Ready,
    Stale,
    Waiting,
}

impl Quest {
    fn readiness(&self) -> Readiness {
        match self.check {
            Check::Passed { revision, .. } if revision == self.revision => Readiness::Ready,
            Check::Passed { .. } => Readiness::Stale,
            Check::Running => Readiness::Waiting,
        }
    }

    fn next_action(&self) -> String {
        match self.readiness() {
            Readiness::Ready => "Review the change and record a decision.".into(),
            Readiness::Stale => format!("Run tests for {} before settling.", self.revision),
            Readiness::Waiting => "Wait for the current tests to finish.".into(),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
enum Screen {
    #[default]
    Queue,
    Review {
        scroll: u16,
    },
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(super) struct Inbox {
    selected: usize,
    attention: [Attention; QUESTS.len()],
    screen: Screen,
    feedback: String,
}

impl Inbox {
    pub(super) fn handle(&mut self, event: &Event) -> bool {
        let Event::Key(key) = event else { return false };
        if key.kind == KeyEventKind::Release
            || !key.modifiers.is_empty() && key.code != KeyCode::BackTab
        {
            return false;
        }
        match (self.screen, key.code) {
            (Screen::Queue, KeyCode::Char('j') | KeyCode::Down | KeyCode::Tab) => {
                self.selected = (self.selected + 1).min(QUESTS.len() - 1);
                self.feedback.clear();
            }
            (Screen::Queue, KeyCode::Char('k') | KeyCode::Up | KeyCode::BackTab) => {
                self.selected = self.selected.saturating_sub(1);
                self.feedback.clear();
            }
            (Screen::Queue, KeyCode::Enter) => {
                self.screen = Screen::Review { scroll: 0 };
                self.feedback.clear();
            }
            (Screen::Queue, KeyCode::Char(' ')) => {
                if self.attention[self.selected] == Attention::Unseen {
                    self.attention[self.selected] = Attention::Seen;
                    self.feedback = "Marked seen.".into();
                }
            }
            (Screen::Queue, KeyCode::Char('s')) => match self.attention[self.selected] {
                Attention::Unseen | Attention::Seen => {
                    self.attention[self.selected] = Attention::Snoozed;
                    self.feedback = "Snoozed. Press s again to restore.".into();
                }
                Attention::Snoozed => {
                    self.attention[self.selected] = Attention::Seen;
                    self.feedback = "Snooze cleared.".into();
                }
                Attention::Settled(_) => {}
            },
            (Screen::Review { .. }, KeyCode::Esc) => {
                self.screen = Screen::Queue;
                self.feedback.clear();
            }
            (Screen::Review { .. }, KeyCode::Char('a')) => self.settle(),
            (Screen::Review { scroll }, KeyCode::Down | KeyCode::Char('j')) => {
                self.screen = Screen::Review {
                    scroll: scroll.saturating_add(1),
                };
            }
            (Screen::Review { scroll }, KeyCode::Up | KeyCode::Char('k')) => {
                self.screen = Screen::Review {
                    scroll: scroll.saturating_sub(1),
                };
            }
            (Screen::Review { .. }, KeyCode::Home) => self.screen = Screen::Review { scroll: 0 },
            _ => return false,
        }
        true
    }

    fn settle(&mut self) {
        let quest = &QUESTS[self.selected];
        if quest.readiness() == Readiness::Ready {
            self.attention[self.selected] = Attention::Settled(quest.revision);
            self.feedback = format!("Review settled for {}.", quest.revision);
        } else {
            self.feedback = quest.next_action();
        }
    }

    fn label(&self, index: usize) -> &'static str {
        match self.attention[index] {
            Attention::Settled(_) => "Review settled",
            Attention::Snoozed => "Snoozed",
            Attention::Unseen | Attention::Seen => match QUESTS[index].readiness() {
                Readiness::Ready => "Ready for review",
                Readiness::Stale => "Stale test evidence",
                Readiness::Waiting => "Waiting for tests",
            },
        }
    }

    fn counts(&self) -> String {
        let (mut due, mut waiting, mut snoozed) = (0, 0, 0);
        for (quest, attention) in QUESTS.iter().zip(self.attention) {
            match attention {
                Attention::Settled(_) => {}
                Attention::Snoozed => snoozed += 1,
                _ if quest.readiness() == Readiness::Waiting => waiting += 1,
                _ => due += 1,
            }
        }
        format!("{due} need you · {waiting} waiting · {snoozed} snoozed")
    }
}

pub(super) fn render(frame: &mut Frame<'_>, area: Rect, inbox: &Inbox) {
    let title = if matches!(inbox.screen, Screen::Queue) {
        " QUEST INBOX "
    } else {
        " QUEST REVIEW "
    };
    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .style(PARCHMENT)
        .border_style(PARCHMENT_BORDER);
    let inner = block.inner(area);
    frame.render_widget(block, area);
    let Rect {
        x,
        y,
        width,
        height,
    } = inner;
    let heading = match inbox.screen {
        Screen::Queue => inbox.counts(),
        Screen::Review { .. } => format!(
            "{} · {}",
            inbox.label(inbox.selected),
            QUESTS[inbox.selected].revision
        ),
    };
    let footer = match inbox.screen {
        Screen::Queue => {
            "[Enter] Review  [j/k] Select\n[Space] Mark seen  [s] Snooze/restore  [Esc] Back"
        }
        Screen::Review { .. } => "[a] Settle review  [j/k] Read  [Esc] Inbox",
    };
    let footer_text = wrap_text(footer, width);
    let footer_height = u16::try_from(footer_text.len())
        .unwrap_or(u16::MAX)
        .min(height.saturating_sub(1) / 2);
    let heading_height = height.min(1);
    let notice_height =
        u16::from(!inbox.feedback.is_empty() && height > heading_height + footer_height);
    frame.render_widget(
        Paragraph::new(heading).style(PARCHMENT_BORDER),
        Rect::new(x, y, width, heading_height),
    );
    let body = Rect::new(
        x,
        y + heading_height,
        width,
        height.saturating_sub(heading_height + footer_height + notice_height),
    );
    match inbox.screen {
        Screen::Queue => render_queue(frame, body, inbox),
        Screen::Review { scroll } => render_review(frame, body, inbox, scroll),
    }
    if notice_height > 0 {
        frame.render_widget(
            Paragraph::new(inbox.feedback.as_str()).style(PARCHMENT_BORDER),
            Rect::new(x, body.bottom(), width, notice_height),
        );
    }
    frame.render_widget(
        Paragraph::new(footer_text.join("\n")).style(PARCHMENT_BORDER),
        Rect::new(
            x,
            inner.bottom().saturating_sub(footer_height),
            width,
            footer_height,
        ),
    );
}

fn render_queue(frame: &mut Frame<'_>, body: Rect, inbox: &Inbox) {
    let wide = body.width >= 78;
    let list = Rect::new(
        body.x,
        body.y,
        if wide { 38 } else { body.width },
        body.height,
    );
    let mut rows = Vec::new();
    let mut selected_start = 0;
    for (index, quest) in QUESTS.iter().enumerate() {
        let selected = inbox.selected == index;
        let marker = if selected { ">" } else { " " };
        let unread = if inbox.attention[index] == Attention::Unseen {
            " · New"
        } else {
            ""
        };
        let style = if selected {
            PARCHMENT_BORDER
        } else {
            PARCHMENT
        };
        if selected {
            selected_start = rows.len();
        }
        rows.push(Line::styled(format!("{marker} {}", quest.title), style));
        rows.push(Line::styled(
            format!("  {}{unread}", inbox.label(index)),
            style,
        ));
    }
    let visible = usize::from(body.height);
    let start = selected_start
        .saturating_sub(visible.saturating_sub(2))
        .min(rows.len().saturating_sub(visible));
    frame.render_widget(
        Paragraph::new(rows.into_iter().skip(start).collect::<Vec<_>>()),
        list,
    );
    if wide {
        let quest = &QUESTS[inbox.selected];
        let details = vec![
            Line::from(Span::styled(
                quest.campaign,
                PARCHMENT.add_modifier(Modifier::BOLD),
            )),
            Line::default(),
            Line::from(quest.acceptance),
            Line::default(),
            Line::from(
                if let Attention::Settled(revision) = inbox.attention[inbox.selected] {
                    format!("Review settled for {revision} · PR {}", quest.pull_request)
                } else {
                    quest.next_action()
                },
            ),
        ];
        frame.render_widget(
            Paragraph::new(details)
                .style(PARCHMENT)
                .wrap(Wrap { trim: true }),
            Rect::new(body.x + 40, body.y, body.width - 40, body.height),
        );
    }
}

fn render_review(frame: &mut Frame<'_>, body: Rect, inbox: &Inbox, scroll: u16) {
    let quest = &QUESTS[inbox.selected];
    let tests = match quest.check {
        Check::Passed { revision, result } if revision == quest.revision => {
            format!("{result} · {revision}")
        }
        Check::Passed { revision, result } => format!(
            "{result} on {revision}. Current revision is {}.",
            quest.revision
        ),
        Check::Running => format!("Running for {}", quest.revision),
    };
    let content = format!(
        "{}\n{} · PR {}\n\nAcceptance\n{}\n\nChanges · {}\n{}\n\nTests\n{tests}\n\nAgent reports\n{}",
        quest.title,
        quest.campaign,
        quest.pull_request,
        quest.acceptance,
        quest.revision,
        quest.diff,
        quest.report
    );
    let lines = wrap_text(&content, body.width);
    let scroll = usize::from(scroll).min(lines.len().saturating_sub(usize::from(body.height)));
    frame.render_widget(
        Paragraph::new(
            lines
                .into_iter()
                .skip(scroll)
                .collect::<Vec<_>>()
                .join("\n"),
        )
        .style(PARCHMENT),
        body,
    );
}
