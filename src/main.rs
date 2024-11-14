use std::sync::mpsc;
use std::time::Duration;

use steamworks::*;

fn main() {
    let (client, single) = Client::init_app(AppId(3146520)).unwrap();

    let matchmaking = client.matchmaking();
    let networking = client.networking();

    let (lobby_tx, lobby_rx) = mpsc::channel();
    let (join_tx, join_rx) = mpsc::channel();

    matchmaking
        .set_lobby_list_filter(LobbyListFilter {
            /*
            string: Some(vec![
                StringFilter(
                    LobbyKey::new("code"), &lobby_code, StringFilterKind::Include
                ),
            ]),
            */
            string: None,
            number: None,
            near_value: None,
            open_slots: None,
            distance: Some(DistanceFilter::Worldwide),
            count: None,
        })
        .request_lobby_list(move |lobbies| {
            for lobby in lobbies.unwrap() {
                let tx = lobby_tx.clone();
                tx.send(lobby).unwrap();
            }
        });

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
                "\t> Joined lobby {}! ({}/{} Players)",
                lobby.raw(),
                matchmaking.lobby_member_count(lobby),
                matchmaking.lobby_member_limit(lobby).unwrap()
            );
        }
    }
}
