use clap::Parser;
use clap_derive::Parser;
use rpc_async_client::{make_worker_token, worker_loop};
use rpc_common::client_args::RpcClientArgs;
use rpc_common::load_keypair;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use tokio::select;
use tokio::signal::unix::{SignalKind, signal};
use tracing::{error, info};
use uuid::Uuid;

mod operations;
mod router;
mod config;
mod util;
mod database;
mod providers;
mod types;
mod object_diff;

use operations::create_default_registry;
use router::{start_http_server, create_rpc_handler};

// TODO: timeouts, and generally more error handling
#[derive(Parser, Debug)]
struct Args {
    #[command(flatten)]
    client_args: RpcClientArgs,

    #[arg(long, help = "Enable debug logging", default_value = "false")]
    debug: bool,

    #[arg(long, help = "HTTP listen address", default_value = "0.0.0.0:3000")]
    http_address: String,
}

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<(), eyre::Error> {
    color_eyre::install()?;
    let args: Args = Args::parse();

    moor_common::tracing::init_tracing(args.debug).expect("Unable to configure logging");

    let mut hup_signal = match signal(SignalKind::hangup()) {
        Ok(signal) => signal,
        Err(e) => {
            error!("Unable to register HUP signal handler: {}", e);
            std::process::exit(1);
        }
    };
    let mut stop_signal = match signal(SignalKind::interrupt()) {
        Ok(signal) => signal,
        Err(e) => {
            error!("Unable to register STOP signal handler: {}", e);
            std::process::exit(1);
        }
    };

    let kill_switch = Arc::new(AtomicBool::new(false));

    let (private_key, _public_key) =
        match load_keypair(&args.client_args.public_key, &args.client_args.private_key) {
            Ok(keypair) => keypair,
            Err(e) => {
                error!(
                    "Unable to load keypair from public and private key files: {}",
                    e
                );
                std::process::exit(1);
            }
        };
    let my_id = Uuid::new_v4();
    let worker_token = make_worker_token(&private_key, my_id);

    // Create operation registry and register operations
    let (registry, _objects_tree) = create_default_registry()
        .map_err(|e| eyre::eyre!("Failed to create default registry: {}", e))?;
    let registry = Arc::new(registry);
    info!("Registered operations: {:?}", registry.list_operations());

    let worker_response_rpc_addr = args.client_args.workers_response_address.clone();
    let worker_request_rpc_addr = args.client_args.workers_request_address.clone();
    let worker_type = moor_var::Symbol::mk("vcs");
    let ks = kill_switch.clone();
    let perform_func = Arc::new(create_rpc_handler(registry.clone()));
    
    // Start RPC worker loop
    let worker_loop_thread = tokio::spawn(async move {
        if let Err(e) = worker_loop(
            &ks,
            my_id,
            &worker_token,
            &worker_response_rpc_addr,
            &worker_request_rpc_addr,
            worker_type,
            perform_func,
        )
        .await
        {
            error!("Worker loop for {my_id} exited with error: {}", e);
            ks.store(true, std::sync::atomic::Ordering::Relaxed);
        }
    });

    // Start HTTP server
    let http_address = args.http_address.clone();
    let http_registry = registry.clone();
    let http_server_thread = tokio::spawn(async move {
        if let Err(e) = start_http_server(&http_address, http_registry).await {
            error!("HTTP server error: {}", e);
        }
    });

    select! {
        _ = hup_signal.recv() => {
            info!("Received HUP signal, reloading configuration is not supported yet");
        },
        _ = stop_signal.recv() => {
            info!("Received STOP signal, shutting down...");
            kill_switch.store(true, std::sync::atomic::Ordering::Relaxed);
        },
        _ = worker_loop_thread => {
            info!("Worker loop thread exited");
            kill_switch.store(true, std::sync::atomic::Ordering::Relaxed);
        },
        _ = http_server_thread => {
            info!("HTTP server thread exited");
            kill_switch.store(true, std::sync::atomic::Ordering::Relaxed);
        }
    }
    info!("Done");
    Ok(())
}