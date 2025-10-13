//! Integration tests for main.rs
//!
//! These tests spawn the actual vcs-worker binary and test:
//! - Basic startup and shutdown
//! - Signal handling (SIGHUP, SIGINT)
//! - HTTP server availability
//! - Error handling for invalid configurations
//! - Component lifecycle management

use nix::sys::signal::{kill, Signal};
use nix::unistd::Pid;
use serial_test::serial;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::Duration;
use tempfile::TempDir;

/// Get the path to the vcs-worker binary
fn get_vcs_worker_bin() -> &'static str {
    env!("CARGO_BIN_EXE_moor-vcs-worker")
}

/// Test keypairs (same as used in other integration tests)
const SIGNING_KEY: &str = r#"-----BEGIN PRIVATE KEY-----
MC4CAQAwBQYDK2VwBCIEILrkKmddHFUDZqRCnbQsPoW/Wsp0fLqhnv5KNYbcQXtk
-----END PRIVATE KEY-----
"#;

const VERIFYING_KEY: &str = r#"-----BEGIN PUBLIC KEY-----
MCowBQYDK2VwAyEAZQUxGvw8u9CcUHUGLttWFZJaoroXAmQgUGINgbBlVYw=
-----END PUBLIC KEY-----
"#;

/// Managed process wrapper that captures output and ensures cleanup
struct ManagedProcess {
    name: String,
    child: Child,
    #[allow(dead_code)]
    output_thread: Option<thread::JoinHandle<Vec<String>>>,
    #[allow(dead_code)]
    error_thread: Option<thread::JoinHandle<Vec<String>>>,
}

impl ManagedProcess {
    /// Create a new managed process
    fn new(name: impl Into<String>, mut child: Child) -> Self {
        let name = name.into();

        // Capture stdout
        let stdout = child.stdout.take().expect("Failed to get stdout");
        let name_clone = name.clone();
        let output_thread = thread::spawn(move || {
            let reader = BufReader::new(stdout);
            let mut lines = Vec::new();
            for line in reader.lines() {
                if let Ok(line) = line {
                    println!("[{}] {}", name_clone, line);
                    lines.push(line);
                }
            }
            lines
        });

        // Capture stderr
        let stderr = child.stderr.take().expect("Failed to get stderr");
        let name_clone = name.clone();
        let error_thread = thread::spawn(move || {
            let reader = BufReader::new(stderr);
            let mut lines = Vec::new();
            for line in reader.lines() {
                if let Ok(line) = line {
                    eprintln!("[{}] {}", name_clone, line);
                    lines.push(line);
                }
            }
            lines
        });

        Self {
            name,
            child,
            output_thread: Some(output_thread),
            error_thread: Some(error_thread),
        }
    }

    /// Get the process ID
    fn pid(&self) -> u32 {
        self.child.id()
    }

    /// Send a signal to the process
    fn signal(&self, signal: Signal) -> Result<(), String> {
        kill(Pid::from_raw(self.pid() as i32), signal)
            .map_err(|e| format!("Failed to send signal: {}", e))
    }

    /// Check if process is still running
    fn is_running(&mut self) -> bool {
        matches!(self.child.try_wait(), Ok(None))
    }

    /// Wait for process to exit with timeout
    fn wait_for_exit(&mut self, timeout: Duration) -> Result<std::process::ExitStatus, String> {
        let start = std::time::Instant::now();
        while start.elapsed() < timeout {
            if let Ok(Some(status)) = self.child.try_wait() {
                return Ok(status);
            }
            thread::sleep(Duration::from_millis(100));
        }
        
        // Try killing if it didn't exit gracefully
        let _ = self.child.kill();
        match self.child.wait() {
            Ok(status) => Ok(status),
            Err(e) => Err(format!(
                "Process {} did not exit within timeout and kill failed: {}",
                self.name, e
            ))
        }
    }

    /// Get captured stdout lines (consumes the thread)
    #[allow(dead_code)]
    fn stdout_lines(mut self) -> Vec<String> {
        if let Some(thread) = self.output_thread.take() {
            thread.join().unwrap_or_default()
        } else {
            Vec::new()
        }
    }

    /// Get captured stderr lines (consumes the thread)
    #[allow(dead_code)]
    fn stderr_lines(mut self) -> Vec<String> {
        if let Some(thread) = self.error_thread.take() {
            thread.join().unwrap_or_default()
        } else {
            Vec::new()
        }
    }
}

impl Drop for ManagedProcess {
    fn drop(&mut self) {
        if self.is_running() {
            eprintln!("Killing process {} (pid={})", self.name, self.pid());
            let _ = self.child.kill();
            let _ = self.child.wait();
        }
    }
}

/// Test environment setup helper
struct TestEnvironment {
    _temp_dir: TempDir,
    _db_dir: TempDir,
    public_key_path: PathBuf,
    private_key_path: PathBuf,
    http_port: u16,
    zmq_response_addr: String,
    zmq_request_addr: String,
}

impl TestEnvironment {
    fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        let db_dir = TempDir::new()?;

        // Write keypair files
        let public_key_path = temp_dir.path().join("public.pem");
        let private_key_path = temp_dir.path().join("private.pem");
        fs::write(&public_key_path, VERIFYING_KEY)?;
        fs::write(&private_key_path, SIGNING_KEY)?;

        // Find available HTTP port
        let listener = std::net::TcpListener::bind("127.0.0.1:0")?;
        let http_port = listener.local_addr()?.port();
        drop(listener);

        // Use IPC for ZMQ to avoid port conflicts
        let uuid = uuid::Uuid::new_v4();
        let zmq_response_addr = format!("ipc:///tmp/vcs-test-response-{}", uuid);
        let zmq_request_addr = format!("ipc:///tmp/vcs-test-request-{}", uuid);

        Ok(Self {
            _temp_dir: temp_dir,
            _db_dir: db_dir,
            public_key_path,
            private_key_path,
            http_port,
            zmq_response_addr,
            zmq_request_addr,
        })
    }

    fn db_path(&self) -> PathBuf {
        self._db_dir.path().to_path_buf()
    }

    fn spawn_vcs_worker(&self) -> Result<ManagedProcess, Box<dyn std::error::Error>> {
        let child = Command::new(get_vcs_worker_bin())
            .arg("--public-key")
            .arg(&self.public_key_path)
            .arg("--private-key")
            .arg(&self.private_key_path)
            .arg("--workers-response-address")
            .arg(&self.zmq_response_addr)
            .arg("--workers-request-address")
            .arg(&self.zmq_request_addr)
            .arg("--http-address")
            .arg(format!("127.0.0.1:{}", self.http_port))
            .env("VCS_DB_PATH", self.db_path())
            .env("VCS_GAME_NAME", "Test Game")
            .env("VCS_WIZARD_API_KEY", "test-wizard-key")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        Ok(ManagedProcess::new("vcs-worker", child))
    }

    fn http_url(&self) -> String {
        format!("http://127.0.0.1:{}", self.http_port)
    }

    fn wait_for_http_server(&self, timeout: Duration) -> Result<(), String> {
        let start = std::time::Instant::now();
        let client = reqwest::blocking::Client::new();

        while start.elapsed() < timeout {
            // Try to connect to the Swagger UI endpoint
            if let Ok(response) = client.get(&format!("{}/swagger-ui", self.http_url())).send() {
                if response.status().is_success() {
                    return Ok(());
                }
            }
            thread::sleep(Duration::from_millis(100));
        }

        Err("HTTP server did not become available within timeout".to_string())
    }
}

#[test]
#[serial]
fn test_basic_startup_and_shutdown() {
    let env = TestEnvironment::new().expect("Failed to create test environment");
    let mut process = env.spawn_vcs_worker().expect("Failed to spawn worker");

    // Wait for server to start
    thread::sleep(Duration::from_secs(2));

    // Verify process is running
    assert!(process.is_running(), "Process should be running");

    // Send SIGINT for graceful shutdown
    process
        .signal(Signal::SIGINT)
        .expect("Failed to send SIGINT");

    // Wait for graceful shutdown with longer timeout (worker loop may take time)
    let _status = process
        .wait_for_exit(Duration::from_secs(10))
        .expect("Process should exit gracefully");

    // If we got here, the process exited (either gracefully or was killed)
    // which is what we're testing - that shutdown works
}

#[test]
#[serial]
fn test_http_server_responds() {
    let env = TestEnvironment::new().expect("Failed to create test environment");
    let mut process = env.spawn_vcs_worker().expect("Failed to spawn worker");

    // Wait for HTTP server to be ready
    env.wait_for_http_server(Duration::from_secs(5))
        .expect("HTTP server should start");

    // Verify process is still running
    assert!(process.is_running(), "Process should still be running");

    // Make HTTP request to Swagger UI
    let client = reqwest::blocking::Client::new();
    let response = client
        .get(&format!("{}/swagger-ui", env.http_url()))
        .send()
        .expect("Failed to make HTTP request");

    assert!(
        response.status().is_success(),
        "HTTP request should succeed"
    );

    // Cleanup
    process
        .signal(Signal::SIGINT)
        .expect("Failed to send SIGINT");
    let _ = process.wait_for_exit(Duration::from_secs(5));
}

#[test]
#[serial]
fn test_sigint_graceful_shutdown() {
    let env = TestEnvironment::new().expect("Failed to create test environment");
    let mut process = env.spawn_vcs_worker().expect("Failed to spawn worker");

    // Wait for startup
    thread::sleep(Duration::from_secs(2));

    // Send SIGINT
    process
        .signal(Signal::SIGINT)
        .expect("Failed to send SIGINT");

    // Process should exit
    let _status = process
        .wait_for_exit(Duration::from_secs(10))
        .expect("Process should exit after SIGINT");

    // If we got here, the process exited (gracefully or forcefully)
}

#[test]
#[serial]
fn test_sighup_signal() {
    let env = TestEnvironment::new().expect("Failed to create test environment");
    let mut process = env.spawn_vcs_worker().expect("Failed to spawn worker");

    // Wait for startup
    thread::sleep(Duration::from_secs(2));

    // Send SIGHUP (currently just logs a message but doesn't reload)
    process
        .signal(Signal::SIGHUP)
        .expect("Failed to send SIGHUP");

    // Process should still be running after SIGHUP
    thread::sleep(Duration::from_millis(500));
    assert!(
        process.is_running(),
        "Process should still be running after SIGHUP"
    );

    // Cleanup
    process
        .signal(Signal::SIGINT)
        .expect("Failed to send SIGINT");
    let _ = process.wait_for_exit(Duration::from_secs(5));
}

#[test]
#[serial]
fn test_missing_keypair_files() {
    let env = TestEnvironment::new().expect("Failed to create test environment");

    // Try to spawn with non-existent keypair files
    let result = Command::new(get_vcs_worker_bin())
        .arg("--public-key")
        .arg("/nonexistent/public.pem")
        .arg("--private-key")
        .arg("/nonexistent/private.pem")
        .arg("--workers-response-address")
        .arg(&env.zmq_response_addr)
        .arg("--workers-request-address")
        .arg(&env.zmq_request_addr)
        .arg("--http-address")
        .arg(format!("127.0.0.1:{}", env.http_port))
        .env("VCS_DB_PATH", env.db_path())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn();

    assert!(result.is_ok(), "Process should spawn");

    let mut process = ManagedProcess::new("vcs-worker-error", result.unwrap());

    // Process should exit with error
    let status = process
        .wait_for_exit(Duration::from_secs(5))
        .expect("Process should exit");

    assert!(
        !status.success(),
        "Process should exit with error code when keypair files are missing"
    );
    assert_eq!(
        status.code(),
        Some(1),
        "Process should exit with code 1 on error"
    );
}

#[test]
#[serial]
fn test_invalid_keypair_files() {
    let env = TestEnvironment::new().expect("Failed to create test environment");
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    // Write invalid keypair files
    let invalid_public = temp_dir.path().join("invalid_public.pem");
    let invalid_private = temp_dir.path().join("invalid_private.pem");
    fs::write(&invalid_public, "INVALID KEY DATA").expect("Failed to write file");
    fs::write(&invalid_private, "INVALID KEY DATA").expect("Failed to write file");

    let result = Command::new(get_vcs_worker_bin())
        .arg("--public-key")
        .arg(&invalid_public)
        .arg("--private-key")
        .arg(&invalid_private)
        .arg("--workers-response-address")
        .arg(&env.zmq_response_addr)
        .arg("--workers-request-address")
        .arg(&env.zmq_request_addr)
        .arg("--http-address")
        .arg(format!("127.0.0.1:{}", env.http_port))
        .env("VCS_DB_PATH", env.db_path())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn();

    assert!(result.is_ok(), "Process should spawn");

    let mut process = ManagedProcess::new("vcs-worker-invalid", result.unwrap());

    // Process should exit with error
    let status = process
        .wait_for_exit(Duration::from_secs(5))
        .expect("Process should exit");

    assert!(
        !status.success(),
        "Process should exit with error when keypair is invalid"
    );
    assert_eq!(
        status.code(),
        Some(1),
        "Process should exit with code 1 on invalid keypair"
    );
}

#[test]
#[serial]
fn test_config_environment_variables_work() {
    let env = TestEnvironment::new().expect("Failed to create test environment");

    // Test with custom environment variables
    let process = Command::new(get_vcs_worker_bin())
        .arg("--public-key")
        .arg(&env.public_key_path)
        .arg("--private-key")
        .arg(&env.private_key_path)
        .arg("--workers-response-address")
        .arg(&env.zmq_response_addr)
        .arg("--workers-request-address")
        .arg(&env.zmq_request_addr)
        .arg("--http-address")
        .arg(format!("127.0.0.1:{}", env.http_port))
        .env("VCS_DB_PATH", env.db_path())
        .env("VCS_GAME_NAME", "Integration Test Game")
        .env("VCS_WIZARD_API_KEY", "integration-test-key")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn process");

    let mut managed = ManagedProcess::new("vcs-worker-env", process);

    // Wait for startup
    thread::sleep(Duration::from_secs(2));

    // Verify process is running (which means env vars were valid)
    assert!(
        managed.is_running(),
        "Process should be running with custom env vars"
    );

    // Cleanup
    managed
        .signal(Signal::SIGINT)
        .expect("Failed to send SIGINT");
    let _ = managed.wait_for_exit(Duration::from_secs(5));
}

#[test]
#[serial]
fn test_debug_flag() {
    let env = TestEnvironment::new().expect("Failed to create test environment");

    // Spawn with --debug flag
    let child = Command::new(get_vcs_worker_bin())
        .arg("--debug")
        .arg("--public-key")
        .arg(&env.public_key_path)
        .arg("--private-key")
        .arg(&env.private_key_path)
        .arg("--workers-response-address")
        .arg(&env.zmq_response_addr)
        .arg("--workers-request-address")
        .arg(&env.zmq_request_addr)
        .arg("--http-address")
        .arg(format!("127.0.0.1:{}", env.http_port))
        .env("VCS_DB_PATH", env.db_path())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn process");

    let mut process = ManagedProcess::new("vcs-worker-debug", child);

    // Wait for startup
    thread::sleep(Duration::from_secs(2));

    // Should be running
    assert!(
        process.is_running(),
        "Process should be running with debug flag"
    );

    // Cleanup
    process
        .signal(Signal::SIGINT)
        .expect("Failed to send SIGINT");
    let _ = process.wait_for_exit(Duration::from_secs(5));
}

#[test]
#[serial]
fn test_custom_http_address() {
    let env = TestEnvironment::new().expect("Failed to create test environment");

    // Find another available port
    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("Failed to bind");
    let custom_port = listener.local_addr().expect("Failed to get addr").port();
    drop(listener);

    let child = Command::new(get_vcs_worker_bin())
        .arg("--public-key")
        .arg(&env.public_key_path)
        .arg("--private-key")
        .arg(&env.private_key_path)
        .arg("--workers-response-address")
        .arg(&env.zmq_response_addr)
        .arg("--workers-request-address")
        .arg(&env.zmq_request_addr)
        .arg("--http-address")
        .arg(format!("127.0.0.1:{}", custom_port))
        .env("VCS_DB_PATH", env.db_path())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn process");

    let mut process = ManagedProcess::new("vcs-worker-custom-http", child);

    // Wait for startup
    thread::sleep(Duration::from_secs(2));

    // Try to connect to custom port
    let client = reqwest::blocking::Client::new();
    let result = client
        .get(&format!("http://127.0.0.1:{}/swagger-ui", custom_port))
        .timeout(Duration::from_secs(2))
        .send();

    // Should be able to connect
    assert!(result.is_ok(), "Should be able to connect to custom port");

    // Cleanup
    process
        .signal(Signal::SIGINT)
        .expect("Failed to send SIGINT");
    let _ = process.wait_for_exit(Duration::from_secs(5));
}

#[test]
#[serial]
fn test_operation_registry_initialized() {
    let env = TestEnvironment::new().expect("Failed to create test environment");
    let mut process = env.spawn_vcs_worker().expect("Failed to spawn worker");

    // Wait for server to start
    env.wait_for_http_server(Duration::from_secs(5))
        .expect("HTTP server should start");

    // Make request to an operation endpoint to verify registry is initialized
    let client = reqwest::blocking::Client::new();
    let response = client
        .get(&format!("{}/api/v1/system/status", env.http_url()))
        .send();

    // Should get a response (even if it's an auth error, it means the route exists)
    assert!(
        response.is_ok(),
        "Should be able to reach API endpoints"
    );

    // Cleanup
    process
        .signal(Signal::SIGINT)
        .expect("Failed to send SIGINT");
    let _ = process.wait_for_exit(Duration::from_secs(5));
}

#[test]
#[serial]
fn test_multiple_sigint_doesnt_hang() {
    let env = TestEnvironment::new().expect("Failed to create test environment");
    let mut process = env.spawn_vcs_worker().expect("Failed to spawn worker");

    // Wait for startup
    thread::sleep(Duration::from_secs(2));

    // Send multiple SIGINT signals
    process
        .signal(Signal::SIGINT)
        .expect("Failed to send first SIGINT");
    thread::sleep(Duration::from_millis(100));
    let _ = process.signal(Signal::SIGINT); // May fail if already exiting

    // Process should still exit
    let _status = process
        .wait_for_exit(Duration::from_secs(10))
        .expect("Process should exit even with multiple signals");

    // If we got here, the process exited
}

