# Questmancer creative direction

## Decision

Pivot the product from `webmaster` to **Questmancer**, a cozy adventurers'
guild for commanding coding agents.

Questmancer is not a fantasy skin over a dashboard. Its world, language and
theatre form an honest projection of Herdr state. Every major fantasy device
must communicate state, urgency, history, ownership or an available action.

The approved visual balance is:

> Warm working guild at home. Legible old-school crawler in the Delve.

## Product identity

Product: **Questmancer**

User role: **the Questmancer**

Primary views: **The Guild Hall** and **The Delve**

Tagline:

> Assemble the party. Command the delve.

Secondary line:

> Every codebase holds a quest.

Opening proposition:

> Your agents have entered the dungeon.
>
> You are the Questmancer.

The Questmancer summons adventurers, oversees campaigns, answers calls for
counsel, observes expeditions and inspects the spoils returned by completed
work. They are neither an adventurer nor a dungeon master. Questmancy is the
original in-world discipline of coordinating distant parties through maps,
summons and scrying.

There is no Questmancer mascot or autonomous guild master. The user commands
the guild.

## Emotional promise

The guild should feel like a place worth leaving open all day: warm,
inhabited, quietly magical and usefully busy.

The experience is cozy without hiding urgency. Failure is recoverable. An
adventurer may become downed, regroup or depart, but never dies. Completion is
celebrated briefly, then becomes calm work awaiting inspection.

The emotional loop is:

```text
Assemble -> Delve -> Request counsel -> Return -> Inspect spoils -> Rest
```

## World canon

Questmancer has one continuous world with two perspectives:

```text
Realm
└── Guild Hall
    ├── Campaign banners
    ├── Party tables
    ├── Counsel bell
    ├── Spoils desk
    └── Doors into the Delves
        └── Connected chambers occupied by adventurers
```

The core mapping is:

| Questmancer world | Herdr concept |
| --- | --- |
| realm | current Herdr session |
| campaign | workspace |
| delve | worktree or isolated body of work |
| chamber | pane or active area |
| adventurer | coding agent |
| party | agents in one campaign |
| Chronicle | factual event and output history |
| call for counsel | blocked agent |
| returned spoils | completed work awaiting attention |
| observe adventurer | focus the agent pane |
| issue counsel | send text to the agent |
| inspect spoils | open the review surface |
| guild summons | user-attention item |

The metaphor must never invent operational facts. Activity is not progress. A
visual library does not claim the project contains documentation. A crypt does
not imply dead code. Personas do not imply measured agent ability.

## The Guild Hall

The Guild Hall is home: warm timber, old stone, rugs, maps, candles, books,
mugs, bedrolls and a communal hearth. It is both an operational control room
and an inhabited place.

Its stable landmarks are:

- **The Quest Board** holds campaigns and attention.
- **The Party Table** holds the roster and current conditions.
- **The Hearth** receives resting adventurers.
- **The Scrying Table** shows the selected adventurer and recent output.
- **The Counsel Bell** signals blocked adventurers.
- **The Spoils Desk** holds completed work awaiting inspection.
- **The Chronicle Lectern** holds factual history.
- **The Guild Door** communicates arrivals, departures and connection state.

Each campaign owns a banner, expedition table and route into its Delve. Active
adventurers can remain visible through roster portraits, carved tokens or
magical miniatures while physically away.

An empty Guild Hall is peaceful rather than barren: the hearth remains lit,
the tables are clear and the guild awaits its next commission.

## The Delve

The Delve is a connected expedition, not a collection of unrelated cards.
Chambers share walls, corridors, doors and camps. As the party grows, passages
open and new chambers extend the map.

Campaigns may receive deterministic architectural identities such as a ruined
archive, mossy undercroft, crystal cavern, forgotten library or old watchtower.
These variants establish visual identity only.

The party always has a route home. The Delve may be mysterious, but it is not
grim or hopeless.

## State theatre

| Real state | Guild Hall | Delve |
| --- | --- | --- |
| working | expedition active | lit chamber; adventurer exploring, studying or repairing |
| blocked | call for counsel | sealed door and signal lantern |
| done and unread | spoils returned | unopened chest or bundled spoils |
| done and read | victory recorded | cleared, calm chamber |
| idle | resting by the hearth | campfire and bedroll |
| failed | regrouping | cracked equipment and recovery posture |
| exited | departed | empty chamber and extinguished lantern |
| unknown | condition unknown | unexplored darkness |
| reconnecting | scrying pool clouded | fog through the passages |
| focused | observed at the scrying table | selection rune or lit chamber |

Theatre is semantic. A victory sparkle is a short transition, not permanent
state. Colour, motion and decorative art may reinforce a condition but cannot
replace its readable label.

## Adventurers

Agents are the cast. Each receives a stable fantasy identity with three
recognition layers:

```text
Elowen Typeweaver
Elven Wizard
Keeper of Schemas
```

- The name creates recognition.
- The class and ancestry create silhouette and personality.
- The epithet supplies restrained guild humour.

Classic fantasy classes are canonical: Wizard, Rogue, Cleric, Ranger, Bard,
Paladin, Barbarian, Artificer and their peers. Questmancer may later introduce
original guild classes such as Runewright, Testmender, Pathseeker,
Cartographer, Relicwright, Warden and Delver.

Small persistent possessions deepen recognition: a patched satchel, oversized
mug, crooked staff, feathered hat, tiny familiar or bundle of scrolls.

Social arrangements may be inferred only from known state. Two resting
adventurers may share a table, but the product must not claim they spoke,
collaborated or exchanged information unless the host reports it.

## Goblin containment

Questmancer includes a restrained Easter egg inspired by OpenAI's
[Where the goblins came from](https://openai.com/index/where-the-goblins-came-from/).
The homage is original and uses no copied character or visual asset.

Canonical guild notice:

> Goblins were officially removed from the Guild's records.
>
> This has not prevented them from living in the walls.

Extremely rare background sightings may include eyes inside a chest, a stolen
biscuit, a hand behind the Chronicle or tiny figures carrying a scroll through
the rafters. Goblins do not represent errors, agents or hidden system activity.

A secret incantation, `release the goblins`, may trigger a brief contained
outbreak followed by the Chronicle notice `CREATURES DETECTED`. A rare stable
adventurer persona may also be a Goblin.

Goblins are never a mascot and never become a recurring verbal tic. Their
scarcity is part of the callback.

## Voice

Questmancer speaks like a warm, capable guild chronicler: observant,
economical and occasionally amused.

Voice principles:

- warm, never twee;
- magical, never obscure;
- concise, never breathless;
- wry, never slapstick;
- heroic in moderation;
- honest about recoverable failure.

Avoid faux-archaic language. Do not use `ye`, `thy`, `hath` or `forsooth`.
Avoid exclamation marks in routine status and jokes in urgent messages.

### Controls

```text
Observe
Issue Counsel
Inspect Spoils
Acknowledge Summons
Open Chronicle
Enter the Delve
Return to Guild Hall
```

### State copy

```text
Elowen is delving.
Elowen requests counsel at a sealed gate.
Elowen has returned with unopened spoils.
Elowen's victory is recorded.
Elowen is resting by the hearth.
Elowen is regrouping after a difficult encounter.
Elowen has departed the campaign.
Elowen's condition is unknown.
```

### World copy

```text
Welcome, Questmancer. Your guild awaits.
The hearth is warm. The guild awaits its next commission.
No adventurer or campaign answers that name.
The scrying pool is still.
Counsel issued.
Summons acknowledged.
The fog lifts. The realm is visible again.
The Guild Hall settles into quiet.
```

Diagnostics pair the atmosphere with the real cause:

```text
The scrying pool has clouded. Reconnecting...
The realm could not be reached: socket disconnected.
The spoils cannot be inspected here: Reviewr is unavailable.
```

Humour belongs at the edges: epithets, item descriptions, Chronicle marginalia,
empty-state props and rare goblin sightings. It does not belong in a call for
counsel or failure message.

## Visual language

The approved aesthetic combines a cozy working guild with a legible old-school
terminal dungeon crawler.

Guild Hall qualities:

- warm amber, moss green, worn red and dark timber;
- inhabited room geometry rather than dashboard panels floating in space;
- compact full-body pixel adventurers;
- persistent furniture and recognition props;
- restrained animation from hearths, candles and scrying runes.

Delve qualities:

- connected chambers and corridors;
- readable stone geometry and selective darkness;
- each adventurer anchored to a chamber object;
- warm camps and visible routes home;
- state labels that survive monochrome, ASCII and no-motion presentation.

Avoid polished modern game HUDs, card-game layouts, copied role-playing trade
dress, fake 3D, excessive particles and decoration that hides operational
state.

## Creative guardrails

- Questmancer is an original cozy fantasy world, not a licensed D&D setting.
- The user remains the only Questmancer.
- Agents are adventurers; they are not pets or disposable units.
- Never infer enemies, combat, objectives, progress or relationships from
  terminal output.
- Never portray failure as death.
- Keep presence separate from the user's unread or acknowledged attention.
- Give every major prop a semantic or interactive purpose.
- Preserve utility when colour, motion and unusual glyphs are disabled.
- Prefer one coherent world over a theme framework.
- Let rare delight remain rare.

## What survives from Webmaster

The pivot retains the strongest product principles:

- the user occupies a named operator role;
- one domain supports an operational and theatrical projection;
- every major joke communicates state or action;
- personas are stable and deterministic;
- the ambient view remains interactive;
- attention and presence remain distinct;
- the product remains local, event-driven and inexpensive while idle.

The webmaster, website, modem, café and 1990s internet vocabulary do not carry
forward. Questmancer has one coherent fiction.

## Creative acceptance criteria

The direction is successful when:

1. A new user can explain their role as the Questmancer after one screen.
2. Guild Hall and Delve read as two locations in one world.
3. Every core state remains legible without knowing the fantasy vocabulary.
4. The guild feels warm during ordinary work without softening urgent states.
5. Adventurers remain recognizable across sessions.
6. The Chronicle never invents events or dialogue.
7. Failure feels recoverable rather than fatal.
8. The visual world remains coherent at compact terminal sizes.
9. Goblins remain an Easter egg rather than a product mascot.
10. Technical implementation can change without changing this creative canon.
