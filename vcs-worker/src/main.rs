use clap::Parser;
use clap_derive::Parser;
use moor_common::tasks::WorkerError;
use moor_var::{Obj, Symbol, Var};
use rpc_async_client::{make_worker_token, worker_loop};
use rpc_common::client_args::RpcClientArgs;
use rpc_common::{WorkerToken, load_keypair};
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use tokio::select;
use tokio::signal::unix::{SignalKind, signal};
use tracing::{error, info};
use uuid::Uuid;

use moor_vcs_worker::{VcsOperation, VcsProcessor, Config};

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

    // Load configuration from environment variables
    let config = Config::from_env();
    
    moor_common::tracing::init_tracing(config.is_debug_enabled()).expect("Unable to configure logging");

    let _processor = VcsProcessor::with_config(config);

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
    let worker_type = Symbol::mk("vcs");
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

    let config = Config::from_env();
    let mut processor = VcsProcessor::with_config(config);
    let operation_name_str = operation_name.as_arc_string().to_lowercase();
    info!("VCS Worker: Processing operation: '{}' with {} arguments", operation_name_str, arguments.len());
    
    let operation = match operation_name_str.as_str() {
        "update_object" => {
            if arguments.len() < 3 {
                return Err(WorkerError::RequestError(
                    "update_object requires object_name and object_dump arguments".to_string(),
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
        
        "rename_object" => {
            if arguments.len() < 3 {
                return Err(WorkerError::RequestError(
                    "rename_object requires old_name and new_name arguments".to_string(),
                ));
            }
            let old_name = arguments[1].as_string().ok_or_else(|| {
                WorkerError::RequestError("Second argument must be a string (old_name)".to_string())
            })?;
            let new_name = arguments[2].as_string().ok_or_else(|| {
                WorkerError::RequestError("Third argument must be a string (new_name)".to_string())
            })?;
            VcsOperation::RenameObject { 
                old_name: old_name.to_string(),
                new_name: new_name.to_string(),
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
                arguments[2].as_string().unwrap_or_else(|| "vcs-worker")
            } else {
                "vcs-worker"
            };
            let author_email = if arguments.len() > 3 {
                arguments[3].as_string().unwrap_or_else(|| "vcs-worker@system")
            } else {
                "vcs-worker@system"
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
        
        "list_objects" => {
            VcsOperation::ListObjects
        }
        
        "get_objects" => {
            if arguments.len() < 2 {
                return Err(WorkerError::RequestError(
                    "get_objects requires at least one object_name argument".to_string(),
                ));
            }
            
            let mut object_names = Vec::new();
            for i in 1..arguments.len() {
                let object_name = arguments[i].as_string().ok_or_else(|| {
                    WorkerError::RequestError(format!("Argument {} must be a string (object_name)", i + 1))
                })?;
                object_names.push(object_name.to_string());
            }
            
            VcsOperation::GetObjects { object_names }
        }
        
        "get_commits" => {
            let limit = if arguments.len() > 1 {
                arguments[1].as_integer().map(|i| i as usize)
            } else {
                None
            };
            
            let offset = if arguments.len() > 2 {
                arguments[2].as_integer().map(|i| i as usize)
            } else {
                None
            };
            
            VcsOperation::GetCommits { limit, offset }
        }
        
        // Credential management operations
        "set_ssh_key" => {
            if arguments.len() < 3 {
                return Err(WorkerError::RequestError(
                    "set_ssh_key requires key_content and key_name arguments".to_string(),
                ));
            }
            let key_content = arguments[1].as_string().ok_or_else(|| {
                WorkerError::RequestError("Second argument must be a string (key_content)".to_string())
            })?;
            let key_name = arguments[2].as_string().ok_or_else(|| {
                WorkerError::RequestError("Third argument must be a string (key_name)".to_string())
            })?;
            VcsOperation::SetSshKey { 
                key_content: key_content.to_string(), 
                key_name: key_name.to_string() 
            }
        }
        
        "clear_ssh_key" => {
            VcsOperation::ClearSshKey
        }
        
        "set_git_user" => {
            if arguments.len() < 3 {
                return Err(WorkerError::RequestError(
                    "set_git_user requires name and email arguments".to_string(),
                ));
            }
            let name = arguments[1].as_string().ok_or_else(|| {
                WorkerError::RequestError("Second argument must be a string (name)".to_string())
            })?;
            let email = arguments[2].as_string().ok_or_else(|| {
                WorkerError::RequestError("Third argument must be a string (email)".to_string())
            })?;
            VcsOperation::SetGitUser { 
                name: name.to_string(), 
                email: email.to_string() 
            }
        }
        
        "update_ignored_properties" => {
            if arguments.len() < 3 {
                return Err(WorkerError::RequestError(
                    "update_ignored_properties requires object_name and at least one property".to_string(),
                ));
            }
            let object_name = arguments[1].as_string().ok_or_else(|| {
                WorkerError::RequestError("Second argument must be a string (object_name)".to_string())
            })?;
            let mut properties = Vec::new();
            for i in 2..arguments.len() {
                let property = arguments[i].as_string().ok_or_else(|| {
                    WorkerError::RequestError(format!("Argument {} must be a string (property_name)", i + 1))
                })?;
                properties.push(property.to_string());
            }
            VcsOperation::UpdateIgnoredProperties { 
                object_name: object_name.to_string(), 
                properties 
            }
        }
        
        "update_ignored_verbs" => {
            if arguments.len() < 3 {
                return Err(WorkerError::RequestError(
                    "update_ignored_verbs requires object_name and at least one verb".to_string(),
                ));
            }
            let object_name = arguments[1].as_string().ok_or_else(|| {
                WorkerError::RequestError("Second argument must be a string (object_name)".to_string())
            })?;
            let mut verbs = Vec::new();
            for i in 2..arguments.len() {
                let verb = arguments[i].as_string().ok_or_else(|| {
                    WorkerError::RequestError(format!("Argument {} must be a string (verb_name)", i + 1))
                })?;
                verbs.push(verb.to_string());
            }
            VcsOperation::UpdateIgnoredVerbs { 
                object_name: object_name.to_string(), 
                verbs 
            }
        }
        
        "test_ssh" => {
            VcsOperation::TestSshConnection
        }
        
        "pull" => {
            let dry_run = if arguments.len() > 1 {
                arguments[1].as_bool().unwrap_or(false)
            } else {
                false
            };
            VcsOperation::Pull { dry_run }
        }
        
        "reset" => {
            info!("VCS Worker: Creating Reset operation");
            VcsOperation::Reset
        }
        
        _ => {
            return Err(WorkerError::RequestError(format!(
                "Unknown operation: {}",
                operation_name.as_arc_string()
            )));
        }
    };

    info!("VCS Worker: About to process operation: {:?}", operation);
    let result = processor.process_operation(operation);
    
    match result {
        Ok(vars) => {
            info!("VCS operation succeeded");
            Ok(vars)
        }
        Err(e) => {
            error!("VCS operation failed: {}", e);
            Err(e)
        }
    }
}

