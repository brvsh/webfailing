use std::sync::mpsc;
use std::time::Duration;

use steamworks::*;

fn main() {
    let (client, single) = Client::init_app(AppId(3146520)).unwrap();

    let matchmaking = client.matchmaking();
    let networking = client.networking();

    let (lobby_tx, lobby_rx) = mpsc::channel();
    let (join_tx, join_rx) = mpsc::channel();

    for search_filter in (0..20).rev() {
        let tx = lobby_tx.clone();
        matchmaking
            .set_lobby_list_filter(LobbyListFilter {
                string: Some(vec![
                    StringFilter(
                        LobbyKey::new("server_browser_value"), &search_filter.to_string(), StringFilterKind::Include
                    ),
                ]),
                number: None,
                near_value: None,
                open_slots: None,
                distance: Some(DistanceFilter::Worldwide),
                count: Some(u64::MAX),
            })
            .request_lobby_list(move |lobbies| {
                for lobby in lobbies.unwrap() {
                    tx.send(lobby).unwrap();
                }
            });
    }

    loop {
        single.run_callbacks();

        let tx = join_tx.clone();

        if let Ok(lobby) = lobby_rx.recv_timeout(Duration::from_millis(1000)) {
            println!(">> Lobby ID: {}", lobby.raw());
            matchmaking.join_lobby(lobby, move |result| {
                if let Ok(l) = result {
                    tx.send(lobby).unwrap()
                }
            });
        }

        if let Ok(lobby) = join_rx.recv_timeout(Duration::from_millis(1000)) {
            println!(
                "\t> Joined lobby {}! ({}/{} Players) - Type {} - Code {}",
                lobby.raw(),
                matchmaking.lobby_member_count(lobby),
                matchmaking.lobby_member_limit(lobby).unwrap(),
                matchmaking.lobby_data(lobby, "type").unwrap(),
                matchmaking.lobby_data(lobby, "code").unwrap(),
            );

            for member in matchmaking.lobby_members(lobby) {
                println!(
                    "\t\tplayer>>> {}",
                    member.raw()
                );
            }

            /*
            for peer in matchmaking.lobby_members(lobby).iter() {
                if peer.raw() != client.user().steam_id().raw() {
                    println!(
                        "\t> Send packet to {} for {}: - {}",
                        peer.raw(),
                        lobby.raw(),
                        networking.send_p2p_packet(
                            *peer,
                            SendType::Reliable,
                            "r1xwashere-\0test\0".as_bytes(),
                        )
                    );
                }
            }
            */
        }
    }
}
