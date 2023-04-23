use std::collections::{HashMap, HashSet};
use std::time::Duration;

use anyhow::Context;
use rustengan::*;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::io::StdoutLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
enum Payload {
    Broadcast {
        message: usize,
    },
    BroadcastOk,
    Read,
    ReadOk {
        messages: HashSet<usize>,
    },
    Topology {
        topology: HashMap<String, Vec<String>>,
    },
    TopologyOk,
    Gossip {
        seen: HashSet<usize>,
    },
    GossipOk {
        seen: HashSet<usize>,
    },
}

enum InjectedPayload {
    Gossip,
}

struct BroadcastNode {
    node: String,
    id: usize,
    messages: HashSet<usize>,
    known: HashMap<String, HashSet<usize>>,
    neighborhood: Vec<String>,
}

impl Node<(), Payload, InjectedPayload> for BroadcastNode {
    fn from_init(
        _state: (),
        init: Init,
        tx: std::sync::mpsc::Sender<Event<Payload, InjectedPayload>>,
    ) -> anyhow::Result<Self> {
        std::thread::spawn(move || {
            // generate gossip events
            // TODO: handle EOF signal
            loop {
                std::thread::sleep(Duration::from_millis(300));
                if tx.send(Event::Injected(InjectedPayload::Gossip)).is_err() {
                    break;
                }
            }
        });
        Ok(Self {
            id: 1,
            node: init.node_id,
            messages: HashSet::new(),
            known: init
                .node_ids
                .into_iter()
                .map(|nid| (nid, HashSet::new()))
                .collect(),
            neighborhood: Vec::new(),
        })
    }

    fn step(
        &mut self,
        input: Event<Payload, InjectedPayload>,
        output: &mut StdoutLock,
    ) -> anyhow::Result<()>
    where
        Payload: DeserializeOwned,
    {
        match input {
            Event::Message(input) => {
                let mut reply = input.into_reply(Some(&mut self.id));
                match reply.body.payload {
                    Payload::Broadcast { message } => {
                        self.messages.insert(message);
                        reply.body.payload = Payload::BroadcastOk;
                        reply.send(&mut *output).context("reply to broadcast")?;
                        self.id += 1;
                    }
                    Payload::Read => {
                        reply.body.payload = Payload::ReadOk {
                            messages: self.messages.clone(),
                        };
                        reply.send(&mut *output).context("reply to read")?;
                        self.id += 1;
                    }
                    Payload::Topology { mut topology } => {
                        self.neighborhood = topology
                            .remove(&self.node)
                            .unwrap_or_else(|| panic!("no topology given for node {}", self.node));
                        reply.body.payload = Payload::TopologyOk;
                        reply.send(&mut *output).context("reply to topology")?;
                        self.id += 1;
                    }
                    Payload::Gossip { seen } => {
                        self.messages.extend(seen);
                    }
                    Payload::BroadcastOk
                    | Payload::ReadOk { .. }
                    | Payload::TopologyOk
                    | Payload::GossipOk { .. } => {}
                }
            }
            Event::Injected(payload) => match payload {
                InjectedPayload::Gossip => {
                    for n in &self.neighborhood {
                        let known_to_n = &self.known[n];
                        let msg = Message {
                            src: self.node.clone(),
                            dst: n.clone(),
                            body: Body {
                                id: None,
                                in_reply_to: None,
                                payload: Payload::Gossip {
                                    seen: self
                                        .messages
                                        .iter()
                                        .copied()
                                        .filter(|m| !known_to_n.contains(m))
                                        .collect(),
                                },
                            },
                        };
                        msg.send(&mut *output)
                            .with_context(|| format!("gossip to {}", n))?;
                    }
                }
            },
            Event::EOF => {}
        }
        Ok(())
    }
}

fn main() -> anyhow::Result<()> {
    main_loop::<_, BroadcastNode, _, _>(())
}
