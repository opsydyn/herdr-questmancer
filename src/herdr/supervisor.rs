use std::{collections::BTreeSet, time::Duration};

use tokio::{
    sync::{mpsc, watch},
    time::sleep,
};

use super::{
    client::HerdrClient,
    protocol::{SessionSnapshot, WireEvent},
    subscription::{HerdrSubscription, SubscriptionRequest},
};

pub const SUPPORTED_PROTOCOL: u32 = 19;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Backoff {
    initial: Duration,
    maximum: Duration,
}

impl Backoff {
    #[must_use]
    pub const fn new(initial: Duration, maximum: Duration) -> Self {
        Self { initial, maximum }
    }

    #[must_use]
    pub fn delay(self, attempt: u32) -> Duration {
        let shift = attempt.saturating_sub(1).min(31);
        self.initial
            .checked_mul(1_u32 << shift)
            .unwrap_or(self.maximum)
            .min(self.maximum)
    }
}

impl Default for Backoff {
    fn default() -> Self {
        Self::new(Duration::from_millis(250), Duration::from_secs(10))
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum ConnectionUpdate {
    Connected(SessionSnapshot),
    Event(WireEvent),
    Disconnected(String),
    Reconnecting { attempt: u32, delay: Duration },
    Resyncing,
    Incompatible { expected: u32, actual: u32 },
}

#[derive(Clone, Debug)]
pub struct ConnectionSupervisor {
    client: HerdrClient,
    backoff: Backoff,
}

impl ConnectionSupervisor {
    #[must_use]
    pub const fn new(client: HerdrClient, backoff: Backoff) -> Self {
        Self { client, backoff }
    }

    pub async fn run(
        self,
        update_tx: mpsc::Sender<ConnectionUpdate>,
        mut shutdown_rx: watch::Receiver<bool>,
    ) {
        let mut attempt = 0_u32;
        loop {
            if *shutdown_rx.borrow() {
                return;
            }

            let outcome = tokio::select! {
                outcome = self.connection_cycle(&update_tx) => outcome,
                changed = shutdown_rx.changed() => {
                    if changed.is_err() || *shutdown_rx.borrow() {
                        return;
                    }
                    continue;
                }
            };

            match outcome {
                CycleOutcome::Stopped => return,
                CycleOutcome::Incompatible { actual } => {
                    let _ = update_tx
                        .send(ConnectionUpdate::Incompatible {
                            expected: SUPPORTED_PROTOCOL,
                            actual,
                        })
                        .await;
                    return;
                }
                CycleOutcome::Resync => {
                    attempt = 0;
                    if update_tx.send(ConnectionUpdate::Resyncing).await.is_err() {
                        return;
                    }
                }
                CycleOutcome::Disconnected { message, connected } => {
                    if connected {
                        attempt = 0;
                    }
                    if update_tx
                        .send(ConnectionUpdate::Disconnected(message))
                        .await
                        .is_err()
                    {
                        return;
                    }
                    attempt = attempt.saturating_add(1);
                    let delay = self.backoff.delay(attempt);
                    if update_tx
                        .send(ConnectionUpdate::Reconnecting { attempt, delay })
                        .await
                        .is_err()
                    {
                        return;
                    }
                    tokio::select! {
                        () = sleep(delay) => {}
                        changed = shutdown_rx.changed() => {
                            if changed.is_err() || *shutdown_rx.borrow() {
                                return;
                            }
                        }
                    }
                }
            }
        }
    }

    async fn connection_cycle(&self, update_tx: &mpsc::Sender<ConnectionUpdate>) -> CycleOutcome {
        let pong = match self.client.ping().await {
            Ok(pong) => pong,
            Err(error) => return CycleOutcome::disconnected(error, false),
        };
        if pong.protocol != SUPPORTED_PROTOCOL {
            return CycleOutcome::Incompatible {
                actual: pong.protocol,
            };
        }

        let snapshot = match self.client.snapshot().await {
            Ok(snapshot) => snapshot,
            Err(error) => return CycleOutcome::disconnected(error, false),
        };
        if snapshot.protocol != SUPPORTED_PROTOCOL {
            return CycleOutcome::Incompatible {
                actual: snapshot.protocol,
            };
        }

        let request = SubscriptionRequest::for_snapshot(&snapshot);
        let subscribed_pane_ids = pane_subscription_ids(&snapshot);
        let mut subscription =
            match HerdrSubscription::connect(self.client.socket_path(), request).await {
                Ok(subscription) => subscription,
                Err(error) => return CycleOutcome::disconnected(error, false),
            };

        if update_tx
            .send(ConnectionUpdate::Connected(snapshot))
            .await
            .is_err()
        {
            return CycleOutcome::Stopped;
        }

        loop {
            let event = match subscription.next_event().await {
                Ok(Some(event)) => event,
                Ok(None) => {
                    return CycleOutcome::Disconnected {
                        message: "Herdr event subscription closed".to_owned(),
                        connected: true,
                    };
                }
                Err(error) => return CycleOutcome::disconnected(error, true),
            };
            let topology_changed = is_topology_event(&event.event);
            if update_tx
                .send(ConnectionUpdate::Event(event))
                .await
                .is_err()
            {
                return CycleOutcome::Stopped;
            }
            if topology_changed {
                let refreshed_snapshot = match self.client.snapshot().await {
                    Ok(snapshot) => snapshot,
                    Err(error) => return CycleOutcome::disconnected(error, true),
                };
                if refreshed_snapshot.protocol != SUPPORTED_PROTOCOL {
                    return CycleOutcome::Incompatible {
                        actual: refreshed_snapshot.protocol,
                    };
                }
                if pane_subscription_ids(&refreshed_snapshot) != subscribed_pane_ids {
                    return CycleOutcome::Resync;
                }
            }
        }
    }
}

#[derive(Debug)]
enum CycleOutcome {
    Stopped,
    Incompatible { actual: u32 },
    Resync,
    Disconnected { message: String, connected: bool },
}

impl CycleOutcome {
    fn disconnected(error: impl std::fmt::Display, connected: bool) -> Self {
        Self::Disconnected {
            message: error.to_string(),
            connected,
        }
    }
}

fn is_topology_event(event: &str) -> bool {
    matches!(
        event,
        "pane_created"
            | "pane_closed"
            | "pane_moved"
            | "pane_agent_detected"
            | "pane.created"
            | "pane.closed"
            | "pane.moved"
            | "pane.agent_detected"
    )
}

fn pane_subscription_ids(snapshot: &SessionSnapshot) -> BTreeSet<String> {
    snapshot
        .panes
        .iter()
        .map(|pane| pane.pane_id.clone())
        .chain(snapshot.agents.iter().map(|agent| agent.pane_id.clone()))
        .collect()
}
