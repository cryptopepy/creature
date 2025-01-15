// MIT License

/*Copyright (c) 2024 Based Labs

Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the "Software"), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.*/

use warp::Filter;
use std::sync::{Arc, Mutex};
use futures::{SinkExt, StreamExt};
use tokio::sync::broadcast;
use serde_json::json;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::models::Thought;
use crate::systems::colony::Colony;

use tokio::sync::RwLock;

pub async fn start_server(
    colony_data: Arc<RwLock<Colony>>, //Arc<Mutex<Colony>>,
    mut shutdown_signal: broadcast::Receiver<()>,
) {
    let colony_data_filter = warp::any().map(move || Arc::clone(&colony_data));

    let ws_route = warp::path("ws")
        .and(warp::ws())
        .and(colony_data_filter)
        //.map(|ws: warp::ws::Ws, colony_data: Arc<Mutex<Colony>>| {
        .map(|ws: warp::ws::Ws, colony_data: Arc<RwLock<Colony>>| {
            ws.on_upgrade(move |socket| handle_connection(socket, colony_data))
        });

    // Create initialization event sender
    let (init_tx, init_rx) = tokio::sync::mpsc::channel::<serde_json::Value>(100);
    let init_tx = Arc::new(init_tx);
        
    // Share init_tx with the colony initialization code
    let _init_tx_clone = Arc::clone(&init_tx);
    tokio::spawn(async move {
        let mut rx = init_rx;
        while let Some(_event) = rx.recv().await {
            // Broadcast initialization events to all connected clients
            // Implementation will be added in the colony code
        }
    });

    let server = warp::serve(ws_route).run(([127, 0, 0, 1], 3030));

    tokio::select! {
        _ = server => {},
        _ = shutdown_signal.recv() => {
            println!("Shutting down WebSocket server...");
        }
    }
}

#[derive(Clone)]
struct HeartbeatData {
    cell_states: Vec<serde_json::Value>,
    total_energy: f64,
    total_thoughts: usize,
    total_plans: usize,
    cells_count: usize,
    mutation_rate: f64,
    cluster_count: usize,
    grid_depth: usize,
}

fn prepare_heartbeat_data(colony: &Colony) -> HeartbeatData {
    let mut cell_states = Vec::new();
    let mut total_energy = 0.0;
    let mut total_thoughts = 0;
    let mut total_plans = 0;
    
    for (id, cell) in &colony.cells {
        total_energy += cell.energy;
        total_thoughts += cell.thoughts.len();
        if cell.current_plan.is_some() {
            total_plans += 1;
        }

        cell_states.push(json!({
            "id": id,
            "energy": cell.energy,
            "position": {
                "x": cell.position.x,
                "y": cell.position.y,
                "z": cell.position.z,
                "heat": cell.position.heat
            },
            "dimensions": {
                "emergence": cell.dimensional_position.emergence,
                "coherence": cell.dimensional_position.coherence,
                "resilience": cell.dimensional_position.resilience,
                "intelligence": cell.dimensional_position.intelligence,
                "efficiency": cell.dimensional_position.efficiency,
                "integration": cell.dimensional_position.integration
            },
            "thoughts": cell.thoughts.len(),
            "has_plan": cell.current_plan.is_some(),
            "dopamine": cell.dopamine,
            "neighbors": cell.neighbors.len()
        }));
    }

    HeartbeatData {
        cell_states,
        total_energy,
        total_thoughts,
        total_plans,
        cells_count: colony.cells.len(),
        mutation_rate: colony.get_mutation_rate(),
        cluster_count: colony.get_cluster_count(),
        grid_depth: colony.get_max_depth(),
    }
}

async fn generate_heartbeat_from_data(data: HeartbeatData) -> serde_json::Value {
    let avg_energy = if data.cells_count > 0 { 
        data.total_energy / data.cells_count as f64 
    } else { 
        0.0 
    };

    json!({
        "type": "heartbeat",
        "timestamp": SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
        "colony_stats": {
            "total_cells": data.cells_count,
            "total_thoughts": data.total_thoughts,
            "total_plans": data.total_plans,
            "average_energy": avg_energy,
            "mutation_rate": data.mutation_rate,
            "cluster_count": data.cluster_count
        },
        "platform_stats": {
            "memory_usage": data.total_thoughts * std::mem::size_of::<Thought>(),
            "cells_per_cluster": if data.cluster_count > 0 { 
                data.cells_count as f64 / data.cluster_count as f64 
            } else { 
                0.0 
            },
            "grid_depth": data.grid_depth,
            "processing_load": data.total_thoughts as f64 / data.cells_count.max(1) as f64
        },
        "cells": data.cell_states
    })
}

async fn handle_connection(
    ws: warp::ws::WebSocket,
    colony_data: Arc<RwLock<Colony>>, //Arc<Mutex<Colony>>,
) {
    let (mut sender, mut _receiver) = ws.split();

    // Set up intervals for different types of updates
    let mut snapshot_interval = tokio::time::interval(tokio::time::Duration::from_secs(5));
    let mut update_interval = tokio::time::interval(tokio::time::Duration::from_secs(2));
    let mut heartbeat_interval = tokio::time::interval(tokio::time::Duration::from_millis(500));

    /*
    // Send initial snapshot
    let initial_snapshot = {
        let colony = colony_data.lock().unwrap();
        json!({
            "type": "snapshot",
            "timestamp": SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            "colony_stats": {
                "total_cells": colony.cells.len(),
                "average_energy": colony.get_average_energy(),
                "total_thoughts": colony.get_total_thoughts(),
                "total_plans": colony.get_total_plans(),
                "mutation_rate": colony.get_mutation_rate(),
                "cluster_count": colony.get_cluster_count()
            },
            "cells": colony.cells.iter().map(|(id, cell)| {
                json!({
                    "id": id,
                    "position": {
                        "x": cell.position.x,
                        "y": cell.position.y,
                        "z": cell.position.z
                    },
                    "energy": cell.energy,
                    "dimensions": {
                        "emergence": cell.dimensional_position.emergence,
                        "coherence": cell.dimensional_position.coherence,
                        "resilience": cell.dimensional_position.resilience,
                        "intelligence": cell.dimensional_position.intelligence,
                        "efficiency": cell.dimensional_position.efficiency,
                        "integration": cell.dimensional_position.integration
                    },
                    "thoughts": {
                        "count": cell.thoughts.len(),
                        "recent": cell.thoughts.iter().rev().take(5).map(|t| {
                            json!({
                                "id": t.id,
                                "content": t.content,
                                "relevance": t.relevance_score,
                                "confidence": t.confidence_score,
                                "timestamp": t.timestamp
                            })
                        }).collect::<Vec<_>>()
                    },
                    "plan": cell.current_plan.as_ref().map(|p| {
                        json!({
                            "id": p.id,
                            "summary": p.summary,
                            "score": p.score,
                            "status": p.status,
                            "created_at": p.created_at
                        })
                    }),
                    "research_topics": cell.research_topics.clone(),
                    "dopamine": cell.dopamine,
                    "stability": cell.stability,
                    "phase": cell.phase,
                    "lenia_state": cell.lenia_state,
                    "memory_stats": {
                        "thoughts": cell.thoughts.len(),
                        "compressed_memories": cell.compressed_memories.len(),
                        "total_size": cell.thoughts.iter().map(|t| t.content.len()).sum::<usize>() +
                                    cell.compressed_memories.iter().map(|m| m.len()).sum::<usize>()
                    }
                })
            }).collect::<Vec<_>>(),
            "runtime_stats": {
                "uptime_seconds": std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
                "memory_usage": colony.get_total_thoughts() * std::mem::size_of::<Thought>(),
                "active_plans": colony.get_total_plans(),
                "mutation_rate": colony.get_mutation_rate() * 100.0,
                "cluster_efficiency": colony.get_average_energy() / colony.cells.len() as f64,
                "dimensional_balance": {
                    "emergence": colony.cells.values().map(|c| c.dimensional_position.emergence).sum::<f64>() / colony.cells.len() as f64,
                    "coherence": colony.cells.values().map(|c| c.dimensional_position.coherence).sum::<f64>() / colony.cells.len() as f64,
                    "resilience": colony.cells.values().map(|c| c.dimensional_position.resilience).sum::<f64>() / colony.cells.len() as f64,
                    "intelligence": colony.cells.values().map(|c| c.dimensional_position.intelligence).sum::<f64>() / colony.cells.len() as f64,
                    "efficiency": colony.cells.values().map(|c| c.dimensional_position.efficiency).sum::<f64>() / colony.cells.len() as f64,
                    "integration": colony.cells.values().map(|c| c.dimensional_position.integration).sum::<f64>() / colony.cells.len() as f64
                }
            }
        })
    };

    if let Err(e) = sender.send(warp::ws::Message::text(serde_json::to_string(&initial_snapshot).unwrap())).await {
        eprintln!("Error sending initial snapshot: {}", e);
        return;
    }
    */

    loop {
        tokio::select! {
            _ = snapshot_interval.tick() => {
                let snapshot = {
                    //println!("(WS) >>> TO SEND: snapshot <<<");
                    //let colony = colony_data.lock().unwrap();
                    //println!("(WS) >>> Attempting to acquire read lock...");
                    let colony = colony_data.read().await;
                    //println!("(WS) >>> Read lock acquired.");
                    //println!("(WS) >>> DATA Unlocked <<<");
                    let mut _model = "x-ai/grok-beta".to_string();
                    let mut _max_tokens: usize = 120000;
                    let mut _framework = "OpenRouter".to_string();
                    if std::env::var("HF_API_KEY").is_ok() {
                        _framework = "HuggingFace".to_string();
                        _model = std::env::var("HF_MODEL").unwrap_or("nvidia/Llama-3.1-Nemotron-70B-Instruct-HF".to_string());
                        _max_tokens = std::env::var("HF_MAX_TOKENS").unwrap_or("128000".to_string()).parse().unwrap_or(6048);
                    } else if std::env::var("OPENAI_API_KEY").is_ok() {
                        _framework = "OpenAI".to_string();
                        _model = std::env::var("OPENAI_MODEL").unwrap_or("gpt-4o".to_string());
                        _max_tokens = std::env::var("OPENAI_MAX_TOKENS").unwrap_or("16384".to_string()).parse().unwrap_or(16384);
                        let o1_detected = _model.starts_with("o1");
                        if o1_detected {
                            _max_tokens = 32768;
                        }
                    }
                    json!({
                        "type": "snapshot",
                        "model": &_model,
                        "framework": &_framework,
                        "max_tokens": _max_tokens,
                        "timestamp": SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs(),
                        "colony_stats": {
                            "total_cells": colony.cells.len(),
                            "average_energy": colony.get_average_energy(),
                            "total_thoughts": colony.get_total_thoughts(),
                            "total_plans": colony.get_total_plans(),
                            "mutation_rate": colony.get_mutation_rate(),
                            "cluster_count": colony.get_cluster_count()
                        },
                        "cells": colony.cells.iter().map(|(id, cell)| {
                            json!({
                                "id": id,
                                "position": {
                                    "x": cell.position.x,
                                    "y": cell.position.y,
                                    "z": cell.position.z
                                },
                                "energy": cell.energy,
                                "dimensions": {
                                    "emergence": cell.dimensional_position.emergence,
                                    "coherence": cell.dimensional_position.coherence,
                                    "resilience": cell.dimensional_position.resilience,
                                    "intelligence": cell.dimensional_position.intelligence,
                                    "efficiency": cell.dimensional_position.efficiency,
                                    "integration": cell.dimensional_position.integration
                                },
                                "thoughts": {
                                    "count": cell.thoughts.len(),
                                    "recent": cell.thoughts.iter().rev().take(5).map(|t| {
                                        json!({
                                            "id": t.id,
                                            "content": t.content,
                                            "relevance": t.relevance_score,
                                            "confidence": t.confidence_score,
                                            "timestamp": t.timestamp
                                        })
                                    }).collect::<Vec<_>>()
                                },
                                "plan": cell.current_plan.as_ref().map(|p| {
                                    json!({
                                        "id": p.id,
                                        "summary": p.summary,
                                        "score": p.score,
                                        "status": p.status,
                                        "created_at": p.created_at
                                    })
                                }),
                                "memory_stats": {
                                    "thoughts": cell.thoughts.len(),
                                    "compressed_memories": cell.compressed_memories.len(),
                                    "total_size": cell.thoughts.iter().map(|t| t.content.len()).sum::<usize>() +
                                                cell.compressed_memories.iter().map(|m| m.len()).sum::<usize>()
                                }
                            })
                        }).collect::<Vec<_>>(),
                        "runtime_stats": {
                            "uptime_seconds": std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap_or_default()
                                .as_secs(),
                            "memory_usage": colony.get_total_thoughts() * std::mem::size_of::<Thought>(),
                            "active_plans": colony.get_total_plans(),
                            "mutation_rate": colony.get_mutation_rate() * 100.0,
                            "cluster_efficiency": colony.get_average_energy() / colony.cells.len() as f64,
                            "dimensional_balance": {
                                "emergence": colony.cells.values().map(|c| c.dimensional_position.emergence).sum::<f64>() / colony.cells.len() as f64,
                                "coherence": colony.cells.values().map(|c| c.dimensional_position.coherence).sum::<f64>() / colony.cells.len() as f64,
                                "resilience": colony.cells.values().map(|c| c.dimensional_position.resilience).sum::<f64>() / colony.cells.len() as f64,
                                "intelligence": colony.cells.values().map(|c| c.dimensional_position.intelligence).sum::<f64>() / colony.cells.len() as f64,
                                "efficiency": colony.cells.values().map(|c| c.dimensional_position.efficiency).sum::<f64>() / colony.cells.len() as f64,
                                "integration": colony.cells.values().map(|c| c.dimensional_position.integration).sum::<f64>() / colony.cells.len() as f64
                            }
                        }
                    })
                };

                println!("(WS) >>> Sending snapshot <<<");
                if let Err(e) = sender.send(warp::ws::Message::text(serde_json::to_string(&snapshot).unwrap())).await {
                    eprintln!("(WS) >>> Error sending snapshot: {}", e);
                    break;
                } else {
                    println!("(WS) >>> Snapshot sent <<<");
                }
            }
            /*
            _ = update_interval.tick() => {
                let update_json = {
                    let colony = colony_data.lock().unwrap();
                    serde_json::to_string(&json!({
                        "type": "update",
                        "timestamp": SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs(),
                        "colony_stats": {
                            "total_cells": colony.cells.len(),
                            "average_energy": colony.get_average_energy(),
                            "total_thoughts": colony.get_total_thoughts(),
                            "total_plans": colony.get_total_plans(),
                            "mutation_rate": colony.get_mutation_rate(),
                            "cluster_count": colony.get_cluster_count()
                        },
                        "cells": colony.cells.iter().map(|(id, cell)| {
                            json!({
                                "id": id,
                                "position": {
                                    "x": cell.position.x,
                                    "y": cell.position.y,
                                    "z": cell.position.z
                                },
                                "energy": cell.energy,
                                "dimensions": cell.dimensional_position,
                                "thoughts_count": cell.thoughts.len(),
                                "has_plan": cell.current_plan.is_some()
                            })
                        }).collect::<Vec<_>>()
                    })).unwrap()
                };

                if let Err(e) = sender.send(warp::ws::Message::text(update_json)).await {
                    eprintln!("Error sending update: {}", e);
                    break;
                }
            }
            _ = heartbeat_interval.tick() => {
                let heartbeat_data = {
                    let colony = colony_data.lock().unwrap();
                    prepare_heartbeat_data(&colony)
                };
                let heartbeat = generate_heartbeat_from_data(heartbeat_data).await;
                
                if let Err(e) = sender.send(warp::ws::Message::text(serde_json::to_string(&heartbeat).unwrap())).await {
                    eprintln!("Error sending heartbeat: {}", e);
                    break;
                }
            }
            */
        }
    }
}
