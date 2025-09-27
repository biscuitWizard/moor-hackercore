use std::sync::mpsc;
use std::thread;
use std::path::PathBuf;
use tracing::info;
use crate::config::Config;
use super::repository::GitRepository;

/// Git operation commands that can be sent through the channel
#[derive(Debug)]
pub enum GitCommand {
    // File operations
    AddFile { path: PathBuf, response: mpsc::Sender<Result<(), String>> },
    RemoveFile { path: PathBuf, response: mpsc::Sender<Result<(), String>> },
    AddAllChanges { response: mpsc::Sender<Result<(), String>> },
    WriteFile { path: PathBuf, content: String, response: mpsc::Sender<Result<(), String>> },
    ReadFile { path: PathBuf, response: mpsc::Sender<Result<String, String>> },
    FileExists { path: PathBuf, response: mpsc::Sender<bool> },
    RenameFile { old_path: PathBuf, new_path: PathBuf, response: mpsc::Sender<Result<(), String>> },
    
    // Commit operations
    Commit { message: String, author_name: String, author_email: String, response: mpsc::Sender<Result<(), String>> },
    GetCommits { limit: Option<usize>, offset: Option<usize>, response: mpsc::Sender<Result<Vec<crate::vcs::types::CommitInfo>, String>> },
    GetLastCommitInfo { response: mpsc::Sender<Result<Option<crate::vcs::types::CommitInfo>, String>> },
    GetCommitsAheadBehind { local_branch: String, remote_branch: String, response: mpsc::Sender<Result<(usize, usize), String>> },
    GetCommitsBetween { from: String, to: String, response: mpsc::Sender<Result<Vec<crate::vcs::types::CommitInfo>, String>> },
    GetCommitChanges { commit_id: String, response: mpsc::Sender<Result<Vec<crate::vcs::types::CommitChange>, String>> },
    GetFileContentAtCommit { commit_id: String, file_path: String, response: mpsc::Sender<Result<String, String>> },
    RebaseOnto { upstream_branch: String, response: mpsc::Sender<Result<(), String>> },
    RollbackLastCommit { response: mpsc::Sender<Result<(), String>> },
    
    // Remote operations
    Push { response: mpsc::Sender<Result<(), String>> },
    TestSshConnection { response: mpsc::Sender<Result<(), String>> },
    FetchRemote { response: mpsc::Sender<Result<(), String>> },
    GetCurrentBranch { response: mpsc::Sender<Result<Option<String>, String>> },
    GetUpstreamInfo { response: mpsc::Sender<Result<Option<String>, String>> },
    
    // Status operations
    HasChanges { response: mpsc::Sender<Result<bool, String>> },
    GetStatus { response: mpsc::Sender<Result<Vec<String>, String>> },
    ResetWorkingTree { response: mpsc::Sender<Result<(), String>> },
    
    // Configuration
    ConfigureGitUser { response: mpsc::Sender<Result<(), String>> },
    
    // Shutdown
    Shutdown,
}

/// Channel-based git operations handler
pub struct GitChannel {
    sender: mpsc::Sender<GitCommand>,
    handle: Option<thread::JoinHandle<()>>,
}

impl GitChannel {
    /// Create a new git channel with a repository
    pub fn new(repo: GitRepository) -> Self {
        let (sender, receiver) = mpsc::channel();
        
        let handle = thread::spawn(move || {
            Self::git_worker(receiver, repo);
        });
        
        Self {
            sender,
            handle: Some(handle),
        }
    }
    
    /// Create a new git channel that will initialize the repository
    pub fn new_with_config(config: Config) -> Result<Self, Box<dyn std::error::Error>> {
        let repo = GitRepository::init(config.repository_path(), config.clone())?;
        Ok(Self::new(repo))
    }
    
    /// Git worker thread that processes commands
    fn git_worker(receiver: mpsc::Receiver<GitCommand>, repo: GitRepository) {
        info!("Git worker thread started");
        
        while let Ok(command) = receiver.recv() {
            match command {
                GitCommand::AddFile { path, response } => {
                    let result = repo.add_file(&path).map_err(|e| e.to_string());
                    let _ = response.send(result);
                }
                
                GitCommand::RemoveFile { path, response } => {
                    let result = repo.remove_file(&path).map_err(|e| e.to_string());
                    let _ = response.send(result);
                }
                
                GitCommand::AddAllChanges { response } => {
                    let result = repo.add_all_changes().map_err(|e| e.to_string());
                    let _ = response.send(result);
                }
                
                GitCommand::WriteFile { path, content, response } => {
                    let result = repo.write_file(&path, &content).map_err(|e| e.to_string());
                    let _ = response.send(result);
                }
                
                GitCommand::ReadFile { path, response } => {
                    let result = repo.read_file(&path).map_err(|e| e.to_string());
                    let _ = response.send(result);
                }
                
                GitCommand::FileExists { path, response } => {
                    let exists = repo.file_exists(&path);
                    let _ = response.send(exists);
                }
                
                GitCommand::RenameFile { old_path, new_path, response } => {
                    let result = repo.rename_file(&old_path, &new_path).map_err(|e| e.to_string());
                    let _ = response.send(result);
                }
                
                GitCommand::Commit { message, author_name, author_email, response } => {
                    let result = repo.commit(&message, &author_name, &author_email).map(|_| ()).map_err(|e| e.to_string());
                    let _ = response.send(result);
                }
                
                GitCommand::GetCommits { limit, offset, response } => {
                    let result = repo.get_commits(limit, offset).map_err(|e| e.to_string());
                    let _ = response.send(result);
                }
                
                GitCommand::GetLastCommitInfo { response } => {
                    let result = repo.get_last_commit_info().map_err(|e| e.to_string());
                    let _ = response.send(result);
                }
                
                GitCommand::GetCommitsAheadBehind { local_branch, remote_branch, response } => {
                    let result = repo.get_commits_ahead_behind(&local_branch, &remote_branch).map_err(|e| e.to_string());
                    let _ = response.send(result);
                }
                
                GitCommand::GetCommitsBetween { from, to, response } => {
                    let result = repo.get_commits_between(&from, &to).map_err(|e| e.to_string());
                    let _ = response.send(result);
                }
                
                GitCommand::GetCommitChanges { commit_id, response } => {
                    let result = repo.get_commit_changes(&commit_id).map_err(|e| e.to_string());
                    let _ = response.send(result);
                }
                
                GitCommand::GetFileContentAtCommit { commit_id, file_path, response } => {
                    let result = repo.get_file_content_at_commit(&commit_id, &file_path).map_err(|e| e.to_string());
                    let _ = response.send(result);
                }
                
                GitCommand::RebaseOnto { upstream_branch, response } => {
                    let result = repo.rebase_onto(&upstream_branch).map_err(|e| e.to_string());
                    let _ = response.send(result);
                }
                
                GitCommand::RollbackLastCommit { response } => {
                    let result = repo.rollback_last_commit().map_err(|e| e.to_string());
                    let _ = response.send(result);
                }
                
                GitCommand::Push { response } => {
                    let result = repo.push().map_err(|e| e.to_string());
                    let _ = response.send(result);
                }
                
                GitCommand::TestSshConnection { response } => {
                    let result = repo.test_ssh_connection().map_err(|e| e.to_string());
                    let _ = response.send(result);
                }
                
                GitCommand::FetchRemote { response } => {
                    let result = repo.fetch_remote().map_err(|e| e.to_string());
                    let _ = response.send(result);
                }
                
                GitCommand::GetCurrentBranch { response } => {
                    let result = repo.get_current_branch().map_err(|e| e.to_string());
                    let _ = response.send(result);
                }
                
                GitCommand::GetUpstreamInfo { response } => {
                    let result = repo.get_upstream_info().map_err(|e| e.to_string());
                    let _ = response.send(result);
                }
                
                GitCommand::HasChanges { response } => {
                    let result = repo.has_changes().map_err(|e| e.to_string());
                    let _ = response.send(result);
                }
                
                GitCommand::GetStatus { response } => {
                    let result = repo.status().map_err(|e| e.to_string());
                    let _ = response.send(result);
                }
                
                GitCommand::ResetWorkingTree { response } => {
                    let result = repo.reset_working_tree().map_err(|e| e.to_string());
                    let _ = response.send(result);
                }
                
                GitCommand::ConfigureGitUser { response } => {
                    let result = repo.configure_git_user().map_err(|e| e.to_string());
                    let _ = response.send(result);
                }
                
                GitCommand::Shutdown => {
                    info!("Git worker thread shutting down");
                    break;
                }
            }
        }
        
        info!("Git worker thread finished");
    }
    
    // File operations
    pub fn add_file(&self, path: PathBuf) -> Result<(), String> {
        let (response_sender, response_receiver) = mpsc::channel();
        self.sender.send(GitCommand::AddFile { path, response: response_sender })
            .map_err(|_| "Failed to send command to git worker".to_string())?;
        response_receiver.recv().map_err(|_| "Failed to receive response from git worker".to_string())?
    }
    
    pub fn remove_file(&self, path: PathBuf) -> Result<(), String> {
        let (response_sender, response_receiver) = mpsc::channel();
        self.sender.send(GitCommand::RemoveFile { path, response: response_sender })
            .map_err(|_| "Failed to send command to git worker".to_string())?;
        response_receiver.recv().map_err(|_| "Failed to receive response from git worker".to_string())?
    }
    
    pub fn add_all_changes(&self) -> Result<(), String> {
        let (response_sender, response_receiver) = mpsc::channel();
        self.sender.send(GitCommand::AddAllChanges { response: response_sender })
            .map_err(|_| "Failed to send command to git worker".to_string())?;
        response_receiver.recv().map_err(|_| "Failed to receive response from git worker".to_string())?
    }
    
    pub fn write_file(&self, path: PathBuf, content: String) -> Result<(), String> {
        let (response_sender, response_receiver) = mpsc::channel();
        self.sender.send(GitCommand::WriteFile { path, content, response: response_sender })
            .map_err(|_| "Failed to send command to git worker".to_string())?;
        response_receiver.recv().map_err(|_| "Failed to receive response from git worker".to_string())?
    }
    
    pub fn read_file(&self, path: PathBuf) -> Result<String, String> {
        let (response_sender, response_receiver) = mpsc::channel();
        self.sender.send(GitCommand::ReadFile { path, response: response_sender })
            .map_err(|_| "Failed to send command to git worker".to_string())?;
        response_receiver.recv().map_err(|_| "Failed to receive response from git worker".to_string())?
    }
    
    pub fn file_exists(&self, path: PathBuf) -> bool {
        let (response_sender, response_receiver) = mpsc::channel();
        if self.sender.send(GitCommand::FileExists { path, response: response_sender }).is_ok() {
            response_receiver.recv().unwrap_or(false)
        } else {
            false
        }
    }
    
    pub fn rename_file(&self, old_path: PathBuf, new_path: PathBuf) -> Result<(), String> {
        let (response_sender, response_receiver) = mpsc::channel();
        self.sender.send(GitCommand::RenameFile { old_path, new_path, response: response_sender })
            .map_err(|_| "Failed to send command to git worker".to_string())?;
        response_receiver.recv().map_err(|_| "Failed to receive response from git worker".to_string())?
    }
    
    // Commit operations
    pub fn commit(&self, message: String, author_name: String, author_email: String) -> Result<(), String> {
        let (response_sender, response_receiver) = mpsc::channel();
        self.sender.send(GitCommand::Commit { message, author_name, author_email, response: response_sender })
            .map_err(|_| "Failed to send command to git worker".to_string())?;
        response_receiver.recv().map_err(|_| "Failed to receive response from git worker".to_string())?
    }
    
    pub fn get_commits(&self, limit: Option<usize>, offset: Option<usize>) -> Result<Vec<crate::vcs::types::CommitInfo>, String> {
        let (response_sender, response_receiver) = mpsc::channel();
        self.sender.send(GitCommand::GetCommits { limit, offset, response: response_sender })
            .map_err(|_| "Failed to send command to git worker".to_string())?;
        response_receiver.recv().map_err(|_| "Failed to receive response from git worker".to_string())?
    }
    
    pub fn get_last_commit_info(&self) -> Result<Option<crate::vcs::types::CommitInfo>, String> {
        let (response_sender, response_receiver) = mpsc::channel();
        self.sender.send(GitCommand::GetLastCommitInfo { response: response_sender })
            .map_err(|_| "Failed to send command to git worker".to_string())?;
        response_receiver.recv().map_err(|_| "Failed to receive response from git worker".to_string())?
    }
    
    pub fn get_commits_ahead_behind(&self, local_branch: String, remote_branch: String) -> Result<(usize, usize), String> {
        let (response_sender, response_receiver) = mpsc::channel();
        self.sender.send(GitCommand::GetCommitsAheadBehind { local_branch, remote_branch, response: response_sender })
            .map_err(|_| "Failed to send command to git worker".to_string())?;
        response_receiver.recv().map_err(|_| "Failed to receive response from git worker".to_string())?
    }
    
    pub fn get_commits_between(&self, from: String, to: String) -> Result<Vec<crate::vcs::types::CommitInfo>, String> {
        let (response_sender, response_receiver) = mpsc::channel();
        self.sender.send(GitCommand::GetCommitsBetween { from, to, response: response_sender })
            .map_err(|_| "Failed to send command to git worker".to_string())?;
        response_receiver.recv().map_err(|_| "Failed to receive response from git worker".to_string())?
    }
    
    pub fn get_commit_changes(&self, commit_id: String) -> Result<Vec<crate::vcs::types::CommitChange>, String> {
        let (response_sender, response_receiver) = mpsc::channel();
        self.sender.send(GitCommand::GetCommitChanges { commit_id, response: response_sender })
            .map_err(|_| "Failed to send command to git worker".to_string())?;
        response_receiver.recv().map_err(|_| "Failed to receive response from git worker".to_string())?
    }
    
    pub fn get_file_content_at_commit(&self, commit_id: String, file_path: String) -> Result<String, String> {
        let (response_sender, response_receiver) = mpsc::channel();
        self.sender.send(GitCommand::GetFileContentAtCommit { commit_id, file_path, response: response_sender })
            .map_err(|_| "Failed to send command to git worker".to_string())?;
        response_receiver.recv().map_err(|_| "Failed to receive response from git worker".to_string())?
    }
    
    pub fn rebase_onto(&self, upstream_branch: String) -> Result<(), String> {
        let (response_sender, response_receiver) = mpsc::channel();
        self.sender.send(GitCommand::RebaseOnto { upstream_branch, response: response_sender })
            .map_err(|_| "Failed to send command to git worker".to_string())?;
        response_receiver.recv().map_err(|_| "Failed to receive response from git worker".to_string())?
    }
    
    pub fn rollback_last_commit(&self) -> Result<(), String> {
        let (response_sender, response_receiver) = mpsc::channel();
        self.sender.send(GitCommand::RollbackLastCommit { response: response_sender })
            .map_err(|_| "Failed to send command to git worker".to_string())?;
        response_receiver.recv().map_err(|_| "Failed to receive response from git worker".to_string())?
    }
    
    // Remote operations
    pub fn push(&self) -> Result<(), String> {
        let (response_sender, response_receiver) = mpsc::channel();
        self.sender.send(GitCommand::Push { response: response_sender })
            .map_err(|_| "Failed to send command to git worker".to_string())?;
        response_receiver.recv().map_err(|_| "Failed to receive response from git worker".to_string())?
    }
    
    pub fn test_ssh_connection(&self) -> Result<(), String> {
        let (response_sender, response_receiver) = mpsc::channel();
        self.sender.send(GitCommand::TestSshConnection { response: response_sender })
            .map_err(|_| "Failed to send command to git worker".to_string())?;
        response_receiver.recv().map_err(|_| "Failed to receive response from git worker".to_string())?
    }
    
    pub fn fetch_remote(&self) -> Result<(), String> {
        let (response_sender, response_receiver) = mpsc::channel();
        self.sender.send(GitCommand::FetchRemote { response: response_sender })
            .map_err(|_| "Failed to send command to git worker".to_string())?;
        response_receiver.recv().map_err(|_| "Failed to receive response from git worker".to_string())?
    }
    
    pub fn get_current_branch(&self) -> Result<Option<String>, String> {
        let (response_sender, response_receiver) = mpsc::channel();
        self.sender.send(GitCommand::GetCurrentBranch { response: response_sender })
            .map_err(|_| "Failed to send command to git worker".to_string())?;
        response_receiver.recv().map_err(|_| "Failed to receive response from git worker".to_string())?
    }
    
    pub fn get_upstream_info(&self) -> Result<Option<String>, String> {
        let (response_sender, response_receiver) = mpsc::channel();
        self.sender.send(GitCommand::GetUpstreamInfo { response: response_sender })
            .map_err(|_| "Failed to send command to git worker".to_string())?;
        response_receiver.recv().map_err(|_| "Failed to receive response from git worker".to_string())?
    }
    
    // Status operations
    pub fn has_changes(&self) -> Result<bool, String> {
        let (response_sender, response_receiver) = mpsc::channel();
        self.sender.send(GitCommand::HasChanges { response: response_sender })
            .map_err(|_| "Failed to send command to git worker".to_string())?;
        response_receiver.recv().map_err(|_| "Failed to receive response from git worker".to_string())?
    }
    
    pub fn get_status(&self) -> Result<Vec<String>, String> {
        let (response_sender, response_receiver) = mpsc::channel();
        self.sender.send(GitCommand::GetStatus { response: response_sender })
            .map_err(|_| "Failed to send command to git worker".to_string())?;
        response_receiver.recv().map_err(|_| "Failed to receive response from git worker".to_string())?
    }
    
    pub fn reset_working_tree(&self) -> Result<(), String> {
        let (response_sender, response_receiver) = mpsc::channel();
        self.sender.send(GitCommand::ResetWorkingTree { response: response_sender })
            .map_err(|_| "Failed to send command to git worker".to_string())?;
        response_receiver.recv().map_err(|_| "Failed to receive response from git worker".to_string())?
    }
    
    // Configuration
    pub fn configure_git_user(&self) -> Result<(), String> {
        let (response_sender, response_receiver) = mpsc::channel();
        self.sender.send(GitCommand::ConfigureGitUser { response: response_sender })
            .map_err(|_| "Failed to send command to git worker".to_string())?;
        response_receiver.recv().map_err(|_| "Failed to receive response from git worker".to_string())?
    }
    
    /// Shutdown the git worker thread
    pub fn shutdown(self) -> Result<(), String> {
        self.sender.send(GitCommand::Shutdown)
            .map_err(|_| "Failed to send shutdown command to git worker".to_string())?;
        
        if let Some(handle) = self.handle {
            handle.join().map_err(|_| "Failed to join git worker thread".to_string())?;
        }
        
        Ok(())
    }
}
