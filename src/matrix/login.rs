use matrix_sdk::{
    events::AnyToDeviceEvent, Client, ClientConfig, JsonStore, LoopCtrl, SyncSettings,
};
use url::Url;

pub async fn login(homeserver: &str, user: &str, password: &str) -> Result<(), matrix_sdk::Error> {
    let homeserver_url = Url::parse(homeserver).unwrap();

    let store = JsonStore::open("./config.json").unwrap();
    let client_config = ClientConfig::new().state_store(Box::new(store));

    let client = Client::new_with_config(homeserver_url, client_config)?;

    client.login(user, password, None, Some("desktop")).await?;

    let client_ref = &client;
    client
        .sync_with_callback(SyncSettings::new(), |response| async move {
            let client = &client_ref;

            for event in &response.to_device.events {
                let e = event
                    .deserialize()
                    .expect("Can't deserialize to-device event");

                match e {
                    AnyToDeviceEvent::KeyVerificationStart(e) => {
                        let sas = client
                            .get_verification(&e.content.transaction_id)
                            .await
                            .expect("Sas object wasn't created");
                        println!(
                            "Starting verification with {} {}",
                            &sas.other_device().user_id(),
                            &sas.other_device().device_id()
                        );
                        sas.accept().await.unwrap();
                    }

                    AnyToDeviceEvent::KeyVerificationKey(e) => {
                        let sas = client
                            .get_verification(&e.content.transaction_id)
                            .await
                            .expect("Sas object wasn't created");

                        //tokio::spawn(wait_for_confirmation((*client).clone(), sas));
                    }

                    AnyToDeviceEvent::KeyVerificationMac(e) => {
                        let sas = client
                            .get_verification(&e.content.transaction_id)
                            .await
                            .expect("Sas object wasn't created");

                        if sas.is_done() {}
                    }

                    _ => (),
                }
            }

            LoopCtrl::Continue
        })
        .await;

    Ok(())
}
