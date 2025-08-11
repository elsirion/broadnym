use std::sync::Arc;
use leptos::prelude::*;
use leptos_meta::{provide_meta_context};

use nym_client_core::client::base_client::{BaseClient, BaseClientBuilder, ClientInput};
use nym_client_core::client::base_client::storage::Ephemeral;
use nym_client_core::client::inbound_messages::InputMessage;
use nym_client_core::config::Config;
use nym_client_core::init::helpers::gateways_for_init;
use nym_client_core::init::types::GatewaySetup;
use nym_task::connections::TransmissionLane;
use nym_validator_client::nyxd::NyxdClient;
use nym_validator_client::ReqwestRpcClient;
use rand::thread_rng;
use tracing::{error, info};

fn main() {
    // set up logging
    tracing_wasm::set_as_global_default();
    console_error_panic_hook::set_once();
    provide_meta_context();

    mount_to_body(move || {
        let client = LocalResource::new(move || init_nym_client());
        let send = Action::new(move |msg: &String| async move {
            if let Some(Ok(client)) = client.get_untracked() {
                let destination = "AP5BoCNFyz9DcE8B5ERDvXQNbGvxxV2cLy2GJdtf45UK.9kWkeaUiRVcBbFo2h2GmvqUPxtznYcZBNu7fnbDJGpDo@2KuFi7CjKVh1mQrNKhdwLjhXBhVLRktvWMg5k4WXKdrX";
                client.input.send(InputMessage::Anonymous {
                    recipient: destination.parse().unwrap(),
                    data: vec![0x42],
                    reply_surbs: 0,
                    lane: TransmissionLane::General,
                    max_retransmissions: None,
                }).await.unwrap();
                panic!("Message sent to {}", destination);
            } else {
                error!("Nym client is not initialized yet.");
            }
        });
        view! {
           <body class="dark:bg-gray-900">
               <h1>Hello World</h1>

                {move || {
                    match client.get() {
                        Some(Ok(_)) => view! { <p>"Nym client initialized successfully!"</p> }.into_any(),
                        Some(Err(e)) => view! { <p>"Failed to initialize Nym client: " {e.to_string()}</p> }.into_any(),
                        None => view! { <p>"Initializing Nym client..."</p> }.into_any(),
                    }
                }}

                <button
                    class="bg-blue-500 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded"
                    on:click=move |_| {
                        send.dispatch("Hello from Leptos!".to_string());
                    }
                >
                  "Send Message"
                </button>
           </body>
        }
    })
}

struct Client {
    client: BaseClient,
    input: ClientInput,
}

async fn init_nym_client() -> Result<Arc<Client>, String> {
    let config = Config::new("", "");
    let storage = Ephemeral::default();
    let gateways = gateways_for_init(
        &mut thread_rng(),
        &config.client.nym_api_urls,
        None,
        config.debug.topology.minimum_gateway_performance,
        config.debug.topology.ignore_ingress_epoch_role,
    ).await.map_err(|e| e.to_string())?;
    let mut client = BaseClientBuilder::new(config, storage, Option::<NyxdClient<ReqwestRpcClient>>::None)
        .with_gateway_setup(GatewaySetup::New {
            specification: Default::default(),
            available_gateways: gateways,
        })
        .start_base()
        .await
        .map_err(|e| e.to_string())?;


    let destination = "AP5BoCNFyz9DcE8B5ERDvXQNbGvxxV2cLy2GJdtf45UK.9kWkeaUiRVcBbFo2h2GmvqUPxtznYcZBNu7fnbDJGpDo@2KuFi7CjKVh1mQrNKhdwLjhXBhVLRktvWMg5k4WXKdrX";

    let input = client.client_input.register_producer();

    Ok(Arc::new(Client {
        client,
        input,
    }))
}