use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::RwLock;
use tokio::time::{sleep, timeout, Duration};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BullyMessage {
    Election { from_id: u32 },
    Answer { from_id: u32 },
    Coordinator { leader_id: u32 },
    Heartbeat { from_id: u32 },
    HeartbeatAck { from_id: u32 },
}

#[derive(Debug, Clone)]
pub struct NodeInfo {
    pub id: u32,
    pub address: String,
}

pub struct BullyElection {
    pub node_id: u32,
    pub node_address: String,
    pub peers: Arc<RwLock<HashMap<u32, NodeInfo>>>,
    pub current_leader: Arc<RwLock<Option<u32>>>,
    pub leader_alive: Arc<RwLock<bool>>,
}

impl BullyElection {
    pub fn new(node_id: u32, node_address: String) -> Self {
        BullyElection {
            node_id,
            node_address,
            peers: Arc::new(RwLock::new(HashMap::new())),
            current_leader: Arc::new(RwLock::new(None)),
            leader_alive: Arc::new(RwLock::new(true)),
        }
    }

    pub async fn add_peer(&self, id: u32, address: String) {
        let mut peers = self.peers.write().await;
        peers.insert(id, NodeInfo { id, address });
    }

    pub async fn get_leader(&self) -> Option<u32> {
        *self.current_leader.read().await
    }

    pub async fn set_leader(&self, leader_id: u32) {
        let mut leader = self.current_leader.write().await;
        *leader = Some(leader_id);
        let mut alive = self.leader_alive.write().await;
        *alive = true;
        println!("Node {}: New leader is Node {}", self.node_id, leader_id);
    }

    pub async fn is_leader(&self) -> bool {
        if let Some(leader_id) = self.get_leader().await {
            leader_id == self.node_id
        } else {
            false
        }
    }

    /// Start monitoring the leader with heartbeats
    pub async fn start_leader_monitoring(self: Arc<Self>) {
        tokio::spawn(async move {
            loop {
                sleep(Duration::from_secs(5)).await;

                let leader_id = {
                    let leader = self.current_leader.read().await;
                    *leader
                };

                // If I'm not the leader, check if leader is alive
                if let Some(leader_id) = leader_id {
                    if leader_id != self.node_id {
                        let is_alive = self.check_leader_alive(leader_id).await;

                        if !is_alive {
                            println!("Node {}: Leader {} is DOWN! Starting new election...",
                                self.node_id, leader_id);
                            self.start_election().await;
                        }
                    }
                }
            }
        });
    }

    /// Check if the leader is alive by sending heartbeat
    async fn check_leader_alive(&self, leader_id: u32) -> bool {
        let peers = self.peers.read().await;
        if let Some(leader_info) = peers.get(&leader_id) {
            match self.send_heartbeat(&leader_info.address).await {
                Ok(true) => {
                    let mut alive = self.leader_alive.write().await;
                    *alive = true;
                    true
                }
                _ => {
                    let mut alive = self.leader_alive.write().await;
                    *alive = false;
                    false
                }
            }
        } else {
            false
        }
    }

    /// Send heartbeat to leader
    async fn send_heartbeat(&self, address: &str) -> Result<bool, String> {
        let result = timeout(Duration::from_secs(2), async {
            let mut stream = TcpStream::connect(address).await
                .map_err(|e| e.to_string())?;

            let msg = BullyMessage::Heartbeat { from_id: self.node_id };
            let msg_json = serde_json::to_string(&msg)
                .map_err(|e| e.to_string())?;
            stream.write_all(msg_json.as_bytes()).await
                .map_err(|e| e.to_string())?;
            stream.write_all(b"\n").await
                .map_err(|e| e.to_string())?;

            let mut buffer = vec![0u8; 1024];
            let n = stream.read(&mut buffer).await
                .map_err(|e| e.to_string())?;

            if n > 0 {
                if let Ok(BullyMessage::HeartbeatAck { .. }) = serde_json::from_slice(&buffer[..n]) {
                    return Ok::<bool, String>(true);
                }
            }
            Ok(false)
        })
        .await;

        match result {
            Ok(Ok(ack)) => Ok(ack),
            _ => Ok(false),
        }
    }

    /// Start an election
    pub async fn start_election(&self) {
        println!("Node {}: Starting election", self.node_id);

        let peers = self.peers.read().await.clone();
        let higher_nodes: Vec<_> = peers
            .iter()
            .filter(|(id, _)| **id > self.node_id)
            .collect();

        if higher_nodes.is_empty() {
            // I have the highest ID, I'm the leader
            println!("Node {}: I am the new leader!", self.node_id);
            self.set_leader(self.node_id).await;
            self.announce_coordinator().await;
            return;
        }

        // Send ELECTION message to all higher nodes
        let mut received_answer = false;

        for (_peer_id, peer_info) in higher_nodes {
            match self
                .send_message(&peer_info.address, BullyMessage::Election { from_id: self.node_id })
                .await
            {
                Ok(Some(BullyMessage::Answer { .. })) => {
                    received_answer = true;
                }
                _ => {}
            }
        }

        if !received_answer {
            // No one responded, I'm the leader
            println!("Node {}: No response, I am the new leader!", self.node_id);
            self.set_leader(self.node_id).await;
            self.announce_coordinator().await;
        } else {
            // Wait for coordinator announcement
            println!(
                "Node {}: Received answer, waiting for coordinator announcement",
                self.node_id
            );
        }
    }

    /// Announce that this node is the coordinator
    async fn announce_coordinator(&self) {
        let peers = self.peers.read().await.clone();

        for (_, peer_info) in peers.iter() {
            let _ = self
                .send_message(
                    &peer_info.address,
                    BullyMessage::Coordinator {
                        leader_id: self.node_id,
                    },
                )
                .await;
        }
    }

    /// Handle incoming Bully messages
    pub async fn handle_message(&self, msg: BullyMessage) -> Option<BullyMessage> {
        match msg {
            BullyMessage::Election { from_id } => {
                println!(
                    "Node {}: Received ELECTION from Node {}",
                    self.node_id, from_id
                );

                if self.node_id > from_id {
                    // Respond with ANSWER and start own election
                    tokio::spawn({
                        let bully = self.clone();
                        async move {
                            sleep(Duration::from_millis(100)).await;
                            bully.start_election().await;
                        }
                    });

                    return Some(BullyMessage::Answer {
                        from_id: self.node_id,
                    });
                }
                None
            }
            BullyMessage::Coordinator { leader_id } => {
                println!(
                    "Node {}: Received COORDINATOR announcement - Node {} is leader",
                    self.node_id, leader_id
                );
                self.set_leader(leader_id).await;
                None
            }
            BullyMessage::Heartbeat { from_id: _ } => {
                // Respond with heartbeat acknowledgment
                Some(BullyMessage::HeartbeatAck {
                    from_id: self.node_id,
                })
            }
            BullyMessage::HeartbeatAck { .. } => {
                // Just note the acknowledgment
                None
            }
            _ => None,
        }
    }

    /// Send a message to a peer
    async fn send_message(
        &self,
        address: &str,
        message: BullyMessage,
    ) -> Result<Option<BullyMessage>, Box<dyn std::error::Error>> {
        let result = timeout(Duration::from_secs(2), async {
            let mut stream = TcpStream::connect(address).await?;

            // Send message
            let msg_json = serde_json::to_string(&message)?;
            stream.write_all(msg_json.as_bytes()).await?;
            stream.write_all(b"\n").await?;

            // Wait for response if needed
            match message {
                BullyMessage::Election { .. } => {
                    let mut buffer = vec![0u8; 1024];
                    let n = stream.read(&mut buffer).await?;
                    if n > 0 {
                        let response: BullyMessage = serde_json::from_slice(&buffer[..n])?;
                        Ok::<Option<BullyMessage>, Box<dyn std::error::Error>>(Some(response))
                    } else {
                        Ok(None)
                    }
                }
                _ => Ok(None),
            }
        })
        .await;

        match result {
            Ok(Ok(response)) => Ok(response),
            _ => Err("Timeout or error".into()),
        }
    }

    pub fn clone(&self) -> Self {
        BullyElection {
            node_id: self.node_id,
            node_address: self.node_address.clone(),
            peers: Arc::clone(&self.peers),
            current_leader: Arc::clone(&self.current_leader),
            leader_alive: Arc::clone(&self.leader_alive),
        }
    }
}
