use clap::Parser;
use clap_derive::Parser;
use moor_common::tasks::WorkerError;
use moor_var::{Obj, Symbol, Var, v_str};
use rpc_async_client::{make_worker_token, worker_loop};
use rpc_common::client_args::RpcClientArgs;
use rpc_common::{WorkerToken, load_keypair};
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use tokio::select;
use tokio::signal::unix::{SignalKind, signal};
use tracing::{error, info};
use uuid::Uuid;

use moor_vms_worker::{VcsOperation, VcsResult, VcsProcessor};

// TODO: timeouts, and generally more error handling
#[derive(Parser, Debug)]
struct Args {
    #[command(flatten)]
    client_args: RpcClientArgs,

    #[arg(long, help = "Enable debug logging", default_value = "false")]
    debug: bool,
}

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<(), eyre::Error> {
    color_eyre::install()?;
    let args: Args = Args::parse();

    moor_common::tracing::init_tracing(args.debug).expect("Unable to configure logging");

    let mut processor = VcsProcessor::new();
    processor.initialize_repository();

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

    let worker_response_rpc_addr = args.client_args.workers_response_address.clone();
    let worker_request_rpc_addr = args.client_args.workers_request_address.clone();
    let worker_type = Symbol::mk("vms");
    let ks = kill_switch.clone();
    let perform_func = Arc::new(process_vcs_request);
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
        }
    }
    info!("Done");
    Ok(())
}

async fn process_vcs_request(
    _token: WorkerToken,
    _request_id: Uuid,
    _worker_type: Symbol,
    _perms: Obj,
    arguments: Vec<Var>,
    _timeout: Option<std::time::Duration>,
) -> Result<Vec<Var>, WorkerError> {
    if arguments.is_empty() {
        return Err(WorkerError::RequestError(
            "At least one argument (operation) is required".to_string(),
        ));
    }

    // First argument should be the operation name
    let operation_name = arguments[0].as_symbol().map_err(|_| {
        WorkerError::RequestError("First argument must be a symbol (operation name)".to_string())
    })?;

    let mut processor = VcsProcessor::new();
    let operation = match operation_name.as_arc_string().to_lowercase().as_str() {
        "add_object" => {
            if arguments.len() < 3 {
                return Err(WorkerError::RequestError(
                    "add_object requires object_name and object_dump arguments".to_string(),
                ));
            }
            
            let object_name = arguments[1].as_string().ok_or_else(|| {
                WorkerError::RequestError("Second argument must be a string (object_name)".to_string())
            })?;

            // let object_dump = arguments[2].as_string().ok_or_else(|| {
            //     WorkerError::RequestError("Third argument must be a string (object_dump)".to_string())
            // })?;
            let object_dump = if let Some(list) = arguments[2].as_list() {
                let mut lines = Vec::new();
                for item in list.iter() {
                    let Some(s) = item.as_string() else {
                        return Err(WorkerError::RequestError(
                            "Each element of object_dump must be a string".to_string(),
                        ));
                    };
                    lines.push(s.to_string()); // clone into owned String
                }
                lines.join("\n")
            } else {
                return Err(WorkerError::RequestError(
                    "Third argument must be a list of strings (object_dump)".to_string(),
                ));
            };

            VcsOperation::AddOrUpdateObject { 
                object_dump: object_dump.to_string(), 
                object_name: object_name.to_string(),
            }
        }
        
        "delete_object" => {
            if arguments.len() < 2 {
                return Err(WorkerError::RequestError(
                    "delete_object requires object_name argument".to_string(),
                ));
            }
            let object_name = arguments[1].as_string().ok_or_else(|| {
                WorkerError::RequestError("Second argument must be a string (object_name)".to_string())
            })?;
            VcsOperation::DeleteObject { 
                object_name: object_name.to_string(),
            }
        }
        
        "commit" => {
            if arguments.len() < 2 {
                return Err(WorkerError::RequestError(
                    "commit requires commit_message argument".to_string(),
                ));
            }
            let message = arguments[1].as_string().ok_or_else(|| {
                WorkerError::RequestError("Second argument must be a string (commit_message)".to_string())
            })?;
            let author_name = if arguments.len() > 2 {
                arguments[2].as_string().unwrap_or_else(|| "vms-worker")
            } else {
                "vms-worker"
            };
            let author_email = if arguments.len() > 3 {
                arguments[3].as_string().unwrap_or_else(|| "vms-worker@system")
            } else {
                "vms-worker@system"
            };
            VcsOperation::Commit { 
                message: message.to_string(), 
                author_name: author_name.to_string(), 
                author_email: author_email.to_string() 
            }
        }
        
        "status" => {
            VcsOperation::Status
        }
        
        _ => {
            return Err(WorkerError::RequestError(format!(
                "Unknown operation: {}",
                operation_name.as_arc_string()
            )));
        }
    };

    let result = processor.process_operation(operation);
    
    match result {
        VcsResult::Success { message } => {
            info!("VCS operation succeeded: {}", message);
            Ok(vec![v_str(&message)])
        }
        VcsResult::SuccessWithData { message, data } => {
            info!("VCS operation succeeded with data: {}", message);
            let mut result = vec![v_str(&message)];
            for item in data {
                result.push(v_str(&item));
            }
            Ok(result)
        }
        VcsResult::Error { message } => {
            error!("VCS operation failed: {}", message);
            Err(WorkerError::RequestError(message))
        }
    }
}

