use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use matrix_sdk::{
    events::{
        room::message::MessageEventContent, AnySyncMessageEvent, AnySyncRoomEvent, AnyToDeviceEvent,
    },
    Client, ClientConfig, LoopCtrl, SyncSettings,
};
use url::Url;

use crate::session::{write_session, Session};

pub async fn restore_login(
    saved_session: Session,
) -> Result<(Client, matrix_sdk::Session), matrix_sdk::Error> {
    let homeserver_url = Url::parse(&saved_session.homeserver).unwrap();

    let client_config = ClientConfig::new().store_path("./data/config");

    let client = Client::new_with_config(homeserver_url, client_config)?;
    client.restore_login(saved_session.clone().into()).await?;
    client.sync_once(SyncSettings::new()).await?;

    return Ok((client, saved_session.into()));
}

pub async fn login(
    homeserver: &str,
    user: &str,
    password: &str,
) -> Result<(Client, Session), matrix_sdk::Error> {
    let homeserver_url = Url::parse(homeserver).unwrap();

    let client_config = ClientConfig::new().store_path("./data/config");

    let client = Client::new_with_config(homeserver_url, client_config)?;

    let response = client.login(user, password, None, Some("desktop")).await?;
    let session = Session {
        access_token: response.access_token,
        user_id: response.user_id,
        device_id: response.device_id,
        homeserver: String::from(homeserver),
    };

    println!("Login: {:#?}", client);

    //let client_ref = &client;
    //let initial_sync = Arc::new(AtomicBool::from(true));
    //let initial_ref = &initial_sync;

    client.sync_once(SyncSettings::new()).await?;

    //client
    //    .sync_with_callback(SyncSettings::new(), |response| async move {
    //        let client = &client_ref;
    //        let initial = &initial_ref;

    //        for event in &response.to_device.events {
    //            match event {
    //                AnyToDeviceEvent::KeyVerificationStart(e) => {
    //                    let sas = client
    //                        .get_verification(&e.content.transaction_id)
    //                        .await
    //                        .expect("Sas object wasn't created");
    //                    println!(
    //                        "Starting verification with {} {}",
    //                        &sas.other_device().user_id(),
    //                        &sas.other_device().device_id()
    //                    );
    //                    sas.accept().await.unwrap();
    //                }

    //                AnyToDeviceEvent::KeyVerificationKey(e) => {
    //                    let sas = client
    //                        .get_verification(&e.content.transaction_id)
    //                        .await
    //                        .expect("Sas object wasn't created");

    //                    //tokio::spawn(wait_for_confirmation((*client).clone(), sas));
    //                }

    //                AnyToDeviceEvent::KeyVerificationMac(e) => {
    //                    let sas = client
    //                        .get_verification(&e.content.transaction_id)
    //                        .await
    //                        .expect("Sas object wasn't created");

    //                    if sas.is_done() {}
    //                }

    //                _ => (),
    //            }
    //        }

    //        if !initial.load(Ordering::SeqCst) {
    //            for (_room_id, room_info) in response.rooms.join {
    //                for event in room_info.timeline.events {
    //                    if let AnySyncRoomEvent::Message(event) = event {
    //                        match event {
    //                            AnySyncMessageEvent::RoomMessage(m) => {
    //                                if let MessageEventContent::VerificationRequest(_) = &m.content
    //                                {
    //                                    let request = client
    //                                        .get_verification_request(&m.event_id)
    //                                        .await
    //                                        .expect("Request object wasn't created");

    //                                    request
    //                                        .accept()
    //                                        .await
    //                                        .expect("Can't accept verification request");
    //                                }
    //                            }
    //                            AnySyncMessageEvent::KeyVerificationKey(e) => {
    //                                let sas = client
    //                                    .get_verification(&e.content.relation.event_id.as_str())
    //                                    .await
    //                                    .expect("Sas object wasn't created");

    //                                //tokio::spawn(wait_for_confirmation((*client).clone(), sas));
    //                            }
    //                            AnySyncMessageEvent::KeyVerificationMac(e) => {
    //                                let sas = client
    //                                    .get_verification(&e.content.relation.event_id.as_str())
    //                                    .await
    //                                    .expect("Sas object wasn't created");

    //                                if sas.is_done() {
    //                                    println!("{:#?}", sas);
    //                                }
    //                            }
    //                            _ => (),
    //                        }
    //                    }
    //                }
    //            }
    //        }

    //        initial.store(false, Ordering::SeqCst);

    //        LoopCtrl::Continue
    //    })
    //    .await;

    write_session(&session)?;
    Ok((client, session))
}
