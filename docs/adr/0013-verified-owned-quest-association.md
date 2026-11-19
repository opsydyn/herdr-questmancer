---
status: accepted
date: 2026-11-19
decision-makers: Questmancer owner
---

# Verify the live adventurer associated with an owned quest

## Context

An owned quest can name a durable assignee. Pane IDs can be reused and
display names can match by coincidence. The review Inbox already verifies a
session before observing a pane. The Quest Board needs the same evidence
for an explicit, optional association to an owned quest.

## Decision

The Questmancer selects an adventurer in the Hall or Delve, selects an owned
quest in the Board and presses `l` to link or replace its association. `L`
clears the association. The link requires a current quest in the open owned
store and a unique, connected pane occupant with a complete Herdr terminal
and session identity. Display the selected link separately from the assignee.

Keep one link per owned quest in memory for the current connection. Retain
it across acknowledged edits to the same quest. Drop it on connection
change, loss of current source or quest, identity or pane changes, exit, and
managed-pane exclusion. Calling Observe from the Board, War Table or Inbox
uses the shared verified focus command. The Inbox keeps its more specific
review link when one is set. Source-provided bindings and attention routing
retain their existing authority.

Observe checks `pane.get` before `pane.focus` and confirms the identity in
the focus response. An identity mismatch clears links for the rejected
pane and identity and requests a fresh snapshot. Herdr protocol 22 accepts
only a pane ID for focus. If the occupant changes between the two requests,
Questmancer can report the mismatch but cannot make focus atomic.

## Alternatives and consequences

| Option | Consequence |
| --- | --- |
| Resolve the saved assignee name | Invalid when names overlap or a pane or session is replaced. |
| Put live links into the owned work snapshot | Gives a volatile Herdr observation durable authority and makes restart unsafe. |
| Keep a separate in-memory link and reuse verified focus | Requires a new explicit link after reconnect or restart. No migration or new Herdr method is required. |
