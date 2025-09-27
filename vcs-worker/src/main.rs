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

use moor_vcs_worker::{VcsOperation, VcsProcessor, Config, arg_validation::ArgValidation};

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
    info!("Main: Using repository path: {:?}", config.repository_path());
    info!("Main: Using objects directory: {}", config.objects_directory());
    let mut processor = VcsProcessor::with_config(config);
    let operation_name_str = operation_name.as_arc_string().to_lowercase();
    info!("VCS Worker: Processing operation: '{}' with {} arguments", operation_name_str, arguments.len());
    
    let operation = match operation_name_str.as_str() {
        "update_object" => {
            ArgValidation::require_args(&arguments, 3, "update_object")?;
            
            let object_name = ArgValidation::extract_string(&arguments, 1, "object_name")?;

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
            ArgValidation::require_args(&arguments, 2, "delete_object")?;
            let object_name = ArgValidation::extract_string(&arguments, 1, "object_name")?;
            VcsOperation::DeleteObject { 
                object_name,
            }
        }
        
        "rename_object" => {
            ArgValidation::require_args(&arguments, 3, "rename_object")?;
            let old_name = ArgValidation::extract_string(&arguments, 1, "old_name")?;
            let new_name = ArgValidation::extract_string(&arguments, 2, "new_name")?;
            VcsOperation::RenameObject { 
                old_name,
                new_name,
            }
        }
        
        "commit" => {
            ArgValidation::require_args(&arguments, 2, "commit")?;
            let message = ArgValidation::extract_string(&arguments, 1, "commit_message")?;
            let author_name = if arguments.len() > 2 {
                arguments[2].as_string().unwrap_or_else(|| "vcs-worker").to_string()
            } else {
                "vcs-worker".to_string()
            };
            let author_email = if arguments.len() > 3 {
                arguments[3].as_string().unwrap_or_else(|| "vcs-worker@system").to_string()
            } else {
                "vcs-worker@system".to_string()
            };
            VcsOperation::Commit { 
                message, 
                author_name, 
                author_email
            }
        }
        
        "status" => {
            VcsOperation::Status
        }
        
        "list_objects" => {
            VcsOperation::ListObjects
        }
        
        "get_objects" => {
            ArgValidation::require_args(&arguments, 2, "get_objects")?;
            let object_names = ArgValidation::extract_string_list(&arguments, 1, "object_name")?;
            VcsOperation::GetObjects { object_names }
        }
        
        "get_commits" => {
            let limit = ArgValidation::extract_int_or_default(&arguments, 1, None);
            let offset = ArgValidation::extract_int_or_default(&arguments, 2, None);
            VcsOperation::GetCommits { limit, offset }
        }
        
        // Credential management operations
        "set_ssh_key" => {
            ArgValidation::require_args(&arguments, 3, "set_ssh_key")?;
            let key_content = ArgValidation::extract_string(&arguments, 1, "key_content")?;
            let key_name = ArgValidation::extract_string(&arguments, 2, "key_name")?;
            VcsOperation::SetSshKey { 
                key_content, 
                key_name
            }
        }
        
        "clear_ssh_key" => {
            VcsOperation::ClearSshKey
        }
        
        "set_git_user" => {
            ArgValidation::require_args(&arguments, 3, "set_git_user")?;
            let name = ArgValidation::extract_string(&arguments, 1, "name")?;
            let email = ArgValidation::extract_string(&arguments, 2, "email")?;
            VcsOperation::SetGitUser { 
                name, 
                email
            }
        }
        
        "update_ignored_properties" => {
            ArgValidation::require_args(&arguments, 3, "update_ignored_properties")?;
            let object_name = ArgValidation::extract_string(&arguments, 1, "object_name")?;
            let properties = ArgValidation::extract_string_list(&arguments, 2, "property_name")?;
            VcsOperation::UpdateIgnoredProperties { 
                object_name, 
                properties 
            }
        }
        
        "update_ignored_verbs" => {
            ArgValidation::require_args(&arguments, 3, "update_ignored_verbs")?;
            let object_name = ArgValidation::extract_string(&arguments, 1, "object_name")?;
            let verbs = ArgValidation::extract_string_list(&arguments, 2, "verb_name")?;
            VcsOperation::UpdateIgnoredVerbs { 
                object_name, 
                verbs 
            }
        }
        
        "test_ssh" => {
            VcsOperation::TestSshConnection
        }
        
        "pull" => {
            let dry_run = ArgValidation::extract_bool_or_default(&arguments, 1, false);
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

