use std::{error::Error, time::Duration};
use anyhow::{Result};
use libp2p::{
    futures::{stream::FusedStream, StreamExt}, identify, identity::Keypair, noise, ping, rendezvous::{self, client::Event}, swarm::{NetworkBehaviour, SwarmEvent}, tcp, yamux, Multiaddr, Swarm
};
use libp2p::identify::Behaviour;
use flexi_logger::Logger;

#[derive(NetworkBehaviour)]
struct ClientBehaviour {
    identify: identify::Behaviour,
    rendezvous: rendezvous::client::Behaviour,
    ping: ping::Behaviour,
}

#[derive(NetworkBehaviour)]
struct ServerBehaviour {
    identify: identify::Behaviour,
    rendezvous: rendezvous::server::Behaviour,
    ping: ping::Behaviour,
}

fn create_client(keypair: Keypair) -> Result<Swarm<ClientBehaviour>> {
    let swarm = libp2p::SwarmBuilder::with_existing_identity(keypair)
        .with_tokio()
        .with_tcp(
            tcp::Config::default(),
            noise::Config::new,
            yamux::Config::default,
        )?
        .with_behaviour(|key| {
            ClientBehaviour {
                identify: identify::Behaviour::new(identify::Config::new(
                "/ipfs/id/1.0.0".to_string(),
                key.public(),
                )),
                rendezvous: rendezvous::client::Behaviour::new(key.clone()),
                ping: ping::Behaviour::new(ping::Config::new().with_interval(Duration::from_secs(1))),
            }
        })?
        .with_swarm_config(|cfg| cfg.with_idle_connection_timeout(Duration::from_secs(u64::MAX)))
        .build();

    Ok(swarm)
}

fn create_server(keypair: Keypair) -> Result<Swarm<ServerBehaviour>> {
    let swarm = libp2p::SwarmBuilder::with_existing_identity(keypair)
        .with_tokio()
        .with_tcp(
            tcp::Config::default(),
            noise::Config::new,
            yamux::Config::default,
        )?
        .with_behaviour(|key| {
            ServerBehaviour {
                identify: identify::Behaviour::new(identify::Config::new(
                "/ipfs/id/1.0.0".to_string(),
                key.public(),
                )),
                rendezvous: rendezvous::server::Behaviour::new(rendezvous::server::Config::default()),
                ping: ping::Behaviour::new(ping::Config::new().with_interval(Duration::from_secs(1))),
            }
        })?
        .with_swarm_config(|cfg| cfg.with_idle_connection_timeout(Duration::from_secs(10)))
        // .with_swarm_config(|cfg| cfg.with_idle_connection_timeout(Duration::from_secs(u64::MAX)))
        .build();

    Ok(swarm)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    Logger::try_with_str("info")?.start()?;
    //    SwarmBuilder::with_existing_identity(libp2p_identity::Keypair::generate_ed25519())
    // let keypair: libp2p::identity::Keypair = libp2p::identity::Keypair::ed25519_from_bytes([0; 32]).unwrap();
    let keypair = libp2p::identity::Keypair::generate_ed25519();
    let mut swarm = create_client(keypair.clone())?;
    // 12D3KooWDpJ7As7BWAwRMfu1VU2WCqNjvq387JEYKDBj4kx6nXTN
    // println!("{}", keypair.public().to_peer_id());

    let rendezvous_point_address = "/ip4/127.0.0.1/tcp/62649".parse::<Multiaddr>().unwrap();
    let external_address = "/ip4/127.0.0.1/tcp/0".parse::<Multiaddr>().unwrap();
    // swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;
    let rendezvous_point: libp2p::PeerId = "12D3KooWDpJ7As7BWAwRMfu1VU2WCqNjvq387JEYKDBj4kx6nXTN"
        .parse()
        .unwrap();
    // let external_address = "/ip4/127.0.0.1/tcp/0".parse::<Multiaddr>().unwrap();

    swarm.add_external_address(external_address.clone());
    swarm.dial(rendezvous_point_address.clone())?;

    let mut should_become_server = false;

    loop {
        match swarm.select_next_some().await {
            SwarmEvent::OutgoingConnectionError { connection_id, peer_id, error } => {
                println!("OutgoingConnectionError connection_id {}", connection_id);
                // let close_connection = swarm.close_connection(connection_id);
                // println!("close_connection: {}", close_connection);
                should_become_server = true;
                break;

                // swarm.behaviour_mut().rendezvous.
            }
            SwarmEvent::ConnectionClosed { peer_id, cause, .. } => {
                println!("Disconnected from {} {:?}", peer_id, cause);
            }
            SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                if let Err(error) = swarm.behaviour_mut().rendezvous.register(
                    rendezvous::Namespace::from_static("rendezvous"),
                    rendezvous_point,
                    None,
                ) {
                    println!("Failed to register: {error}");
                    return Ok(());
                }
                
                println!("Connection established with rendezvous point {}", peer_id);
            }
            SwarmEvent::Behaviour(ClientBehaviourEvent::Rendezvous(
                rendezvous::client::Event::Registered {
                    namespace,
                    ttl,
                    rendezvous_node,
                },
            )) => {
                println!(
                    "Registered for namespace '{}' at rendezvous point {} for the next {} seconds",
                    namespace,
                    rendezvous_node,
                    ttl
                );
            }
            SwarmEvent::Behaviour(ClientBehaviourEvent::Ping(event)) => {
                println!("Ping {:?}", event);
            }
            SwarmEvent::Behaviour(ClientBehaviourEvent::Identify(event)) => {
                println!("Identify {:?}", event);
            }
            SwarmEvent::Behaviour(ClientBehaviourEvent::Rendezvous(Event::RegisterFailed { rendezvous_node, namespace, error })) => {
                println!("Client: RegisterFailed {} {:?}", rendezvous_node, error);
            }
            other => {
                println!("Client: Unhandled {:?}", other);
            }
        }
    }

    if should_become_server {
        println!("becoming a server");
        let mut swarm = create_server(keypair)?;
        swarm.listen_on(rendezvous_point_address.clone())?;

        loop {
            match swarm.select_next_some().await {
                SwarmEvent::NewListenAddr { address, .. } => {
                    println!("Listening on {}", address);
                }
                SwarmEvent::IncomingConnection { connection_id, .. } => {
                    println!("Server: IncomingConnection to {}", connection_id);
                }
                SwarmEvent::OutgoingConnectionError { connection_id, peer_id, error } => {
                    println!("OutgoingConnectionError connection_id {}", connection_id);
                    
                }
                SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                    println!("Server: Connected to {}", peer_id);
                }
                SwarmEvent::ConnectionClosed { peer_id, cause, .. } => {
                    println!("Disconnected from {} {:?}", peer_id, cause);
                }
                SwarmEvent::Behaviour(ServerBehaviourEvent::Rendezvous(
                    rendezvous::server::Event::PeerRegistered { peer, registration },
                )) => {
                    println!(
                        "Peer {} registered for namespace '{}'",
                        peer,
                        registration.namespace
                    );
                }
                SwarmEvent::Behaviour(ServerBehaviourEvent::Rendezvous(
                    rendezvous::server::Event::DiscoverServed {
                        enquirer,
                        registrations,
                    },
                )) => {
                    println!(
                        "Served peer {} with {} registrations",
                        enquirer,
                        registrations.len()
                    );
                }
                SwarmEvent::Behaviour(ServerBehaviourEvent::Ping(event)) => {
                    println!("Ping {:?}", event);
                }
                SwarmEvent::Behaviour(ServerBehaviourEvent::Identify(event)) => {
                    println!("Identify {:?}", event);
                }
                other => {
                    println!("Server: Unhandled {:?}", other);
                }
            }
        }
    }

    Ok(())
}
