use std::sync::Arc;
use leptos::prelude::*;
use leptos_meta::{provide_meta_context};
use common::{Network, TransactionRequest};

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

fn main() {
    // set up logging
    tracing_wasm::set_as_global_default();
    console_error_panic_hook::set_once();
    provide_meta_context();

    mount_to_body(move || {
        let client = LocalResource::new(move || init_nym_client());
        let (nym_address, set_nym_address) = signal(String::new());
        let (tx_hex, set_tx_hex) = signal(String::new());
        let (network, set_network) = signal(Network::Mainnet);
        let (status, set_status) = signal(String::new());
        let (is_sending, set_is_sending) = signal(false);
        
        let send_transaction = Action::new(move |(nym_addr, tx_hex, network): &(String, String, Network)| {
            let nym_addr = nym_addr.clone();
            let tx_hex = tx_hex.clone();
            let network = network.clone();
            
            async move {
                if let Some(Ok(client)) = client.get_untracked() {
                    let tx_request = TransactionRequest {
                        tx_hex,
                        network,
                    };
                    
                    let message = bincode::serialize(&tx_request)
                        .map_err(|e| format!("Failed to serialize request: {}", e))?;
                    
                    client.input.send(InputMessage::Anonymous {
                        recipient: nym_addr.parse().map_err(|e| format!("Invalid Nym address: {:?}", e))?,
                        data: message,
                        reply_surbs: 0,
                        lane: TransmissionLane::General,
                        max_retransmissions: None,
                    }).await.map_err(|e| format!("Failed to send message: {}", e))?;
                    
                    Ok::<(), String>(())
                } else {
                    Err("Nym client is not initialized yet.".to_string())
                }
            }
        });
        
        let submit_tx = move |_| {
            let nym_addr = nym_address.get();
            let tx = tx_hex.get();
            
            if nym_addr.is_empty() || tx.is_empty() {
                set_status.set("Please fill in all fields".to_string());
                return;
            }
            
            set_is_sending.set(true);
            set_status.set("Sending transaction...".to_string());
            
            send_transaction.dispatch((nym_addr, tx, network.get()));
        };
        
        Effect::new(move || {
            if let Some(result) = send_transaction.value().get() {
                match result {
                    Ok(_) => {
                        set_status.set("Transaction sent successfully!".to_string());
                        set_tx_hex.set(String::new());
                    }
                    Err(e) => {
                        set_status.set(format!("Error: {}", e));
                    }
                }
                set_is_sending.set(false);
            }
        });
        
        view! {
            <body class="bg-gray-50 dark:bg-gray-900 min-h-screen">
                <div class="max-w-2xl mx-auto px-4 py-8">
                    <h1 class="text-3xl font-bold text-gray-900 dark:text-gray-100 mb-2">
                        "Bitcoin Transaction Broadcaster"
                    </h1>
                    <p class="text-gray-600 dark:text-gray-400 mb-8">
                        "Send Bitcoin transactions anonymously through the Nym mixnet"
                    </p>

                    {move || {
                        match client.get() {
                            Some(Ok(_)) => {
                                view! {
                                    <div class="mb-6 p-4 bg-green-100 dark:bg-green-900 border border-green-400 dark:border-green-600 text-green-700 dark:text-green-300 rounded-lg">
                                        "Nym client initialized successfully!"
                                    </div>
                                }
                                    .into_any()
                            }
                            Some(Err(e)) => {
                                view! {
                                    <div class="mb-6 p-4 bg-red-100 dark:bg-red-900 border border-red-400 dark:border-red-600 text-red-700 dark:text-red-300 rounded-lg">
                                        "Failed to initialize Nym client: " {e.to_string()}
                                    </div>
                                }
                                    .into_any()
                            }
                            None => {
                                view! {
                                    <div class="mb-6 p-4 bg-blue-100 dark:bg-blue-900 border border-blue-400 dark:border-blue-600 text-blue-700 dark:text-blue-300 rounded-lg">
                                        "Initializing Nym client..."
                                    </div>
                                }
                                    .into_any()
                            }
                        }
                    }}

                    <div class="bg-white dark:bg-gray-800 rounded-lg shadow-lg p-6">
                        <div class="mb-6">
                            <label
                                for="nym-address"
                                class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2"
                            >
                                "Server Nym Address:"
                            </label>
                            <input
                                id="nym-address"
                                type="text"
                                placeholder="Enter the server's Nym address"
                                class="w-full px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-md focus:ring-2 focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:text-gray-100 dark:placeholder-gray-400 disabled:bg-gray-100 dark:disabled:bg-gray-900 disabled:cursor-not-allowed"
                                prop:value=move || nym_address.get()
                                on:input=move |ev| set_nym_address.set(event_target_value(&ev))
                                prop:disabled=move || is_sending.get()
                            />
                        </div>

                        <div class="mb-6">
                            <label
                                for="tx-hex"
                                class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2"
                            >
                                "Transaction Hex:"
                            </label>
                            <textarea
                                id="tx-hex"
                                placeholder="Paste your raw Bitcoin transaction hex here"
                                rows="6"
                                class="w-full px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-md focus:ring-2 focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:text-gray-100 dark:placeholder-gray-400 font-mono text-sm disabled:bg-gray-100 dark:disabled:bg-gray-900 disabled:cursor-not-allowed"
                                prop:value=move || tx_hex.get()
                                on:input=move |ev| set_tx_hex.set(event_target_value(&ev))
                                prop:disabled=move || is_sending.get()
                            />
                        </div>

                        <div class="mb-6">
                            <label
                                for="network"
                                class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2"
                            >
                                "Network:"
                            </label>
                            <select
                                id="network"
                                class="w-full px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-md focus:ring-2 focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:text-gray-100 disabled:bg-gray-100 dark:disabled:bg-gray-900 disabled:cursor-not-allowed"
                                on:change=move |ev| {
                                    let value = event_target_value(&ev);
                                    set_network
                                        .set(
                                            match value.as_str() {
                                                "testnet" => Network::Testnet,
                                                _ => Network::Mainnet,
                                            },
                                        );
                                }
                                prop:disabled=move || is_sending.get()
                            >
                                <option value="mainnet">"Mainnet"</option>
                                <option value="testnet">"Testnet"</option>
                            </select>
                        </div>

                        <button
                            class="w-full py-3 px-4 bg-blue-600 hover:bg-blue-700 disabled:bg-gray-400 dark:disabled:bg-gray-600 text-white font-medium rounded-md transition duration-200 disabled:cursor-not-allowed"
                            on:click=submit_tx
                            prop:disabled=move || is_sending.get() || client.get().is_none()
                        >
                            {move || {
                                if is_sending.get() { "Sending..." } else { "Submit Transaction" }
                            }}
                        </button>

                        {move || {
                            if !status.get().is_empty() {
                                let (bg_class, border_class, text_class) = if status
                                    .get()
                                    .contains("Error")
                                {
                                    (
                                        "bg-red-100 dark:bg-red-900",
                                        "border-red-400 dark:border-red-600",
                                        "text-red-700 dark:text-red-300",
                                    )
                                } else if status.get().contains("successfully") {
                                    (
                                        "bg-green-100 dark:bg-green-900",
                                        "border-green-400 dark:border-green-600",
                                        "text-green-700 dark:text-green-300",
                                    )
                                } else {
                                    (
                                        "bg-blue-100 dark:bg-blue-900",
                                        "border-blue-400 dark:border-blue-600",
                                        "text-blue-700 dark:text-blue-300",
                                    )
                                };
                                view! {
                                    <div class=format!(
                                        "mt-6 p-4 {} border {} {} rounded-lg",
                                        bg_class,
                                        border_class,
                                        text_class,
                                    )>{status.get()}</div>
                                }
                                    .into_any()
                            } else {
                                view! {}.into_any()
                            }
                        }}
                    </div>
                </div>
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

    let input = client.client_input.register_producer();

    Ok(Arc::new(Client {
        client,
        input,
    }))
}