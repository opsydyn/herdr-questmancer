use crate::domain::Timestamp;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ReviewTarget {
    pub repository: PathBuf,
    pub base: String,
    pub head: String,
    pub recipe: String,
    pub runner: String,
}

impl ReviewTarget {
    pub(crate) fn valid(&self) -> bool {
        self.repository.is_absolute()
            && self.repository.to_str().is_some_and(label)
            && self.base.len() == self.head.len()
            && oid(&self.base)
            && oid(&self.head)
            && self.base != self.head
            && label(&self.recipe)
            && label(&self.runner)
    }

    pub(crate) fn matches(&self, receipt: &CheckReceipt) -> bool {
        self.repository == receipt.repository
            && self.head == receipt.head
            && self.recipe == receipt.recipe
            && self.runner == receipt.runner
    }
}

fn oid(value: &str) -> bool {
    matches!(value.len(), 40 | 64)
        && value
            .bytes()
            .all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c))
}

fn label(value: &str) -> bool {
    !value.trim().is_empty() && value.len() <= 4096 && !value.chars().any(char::is_control)
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CheckResult {
    Passed,
    Failed,
    Inconclusive,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CheckReceipt {
    pub repository: PathBuf,
    pub head: String,
    pub recipe: String,
    pub runner: String,
    pub captured_at: Timestamp,
    pub result: CheckResult,
}

impl CheckReceipt {
    pub(crate) fn parse(input: &str) -> Result<Self, String> {
        let value = crate::persistence::strict_json::parse(input.as_bytes())
            .map_err(|error| error.to_string())?;
        serde_json::from_value(value).map_err(|error| error.to_string())
    }

    pub(crate) fn valid(&self) -> bool {
        self.repository.is_absolute()
            && self.repository.to_str().is_some_and(label)
            && oid(&self.head)
            && label(&self.recipe)
            && label(&self.runner)
            && self.captured_at.as_millis() > 0
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum ReportAuthor {
    Pi,
    Codex,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ReviewReport {
    pub author: ReportAuthor,
    pub summary: String,
}

impl ReviewReport {
    pub(crate) fn valid(&self) -> bool {
        !self.summary.trim().is_empty()
            && self.summary.len() <= 2048
            && !self.summary.chars().any(|c| c.is_control() && c != '\n')
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ReviewSettlement {
    pub actor: String,
    pub at: Timestamp,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Review {
    pub target: ReviewTarget,
    pub report: Option<ReviewReport>,
    pub check: Option<CheckReceipt>,
    pub settlement: Option<ReviewSettlement>,
    pub(crate) requested_revision: u64,
    pub(crate) check_revision: Option<u64>,
}

impl Review {
    pub fn check_passed(&self) -> bool {
        self.check.as_ref().is_some_and(|receipt| {
            self.target.matches(receipt) && receipt.result == CheckResult::Passed
        })
    }
}
