use actix_web::{web, App, HttpServer, HttpResponse, Result as ActixResult};
use serde::{Deserialize, Serialize};
use std::process::{Command, Stdio};
use std::io::Write;
use std::fs::OpenOptions;
use std::path::PathBuf;
use chrono::Local;
use tokio::process::Command as TokioCommand;

#[derive(Deserialize)]
struct ExecuteRequest {
    command: String,
    #[serde(default = "default_timeout")]
    #[allow(dead_code)]
    timeout: u64,
}

fn default_timeout() -> u64 {
    30
}

#[derive(Serialize)]
struct ExecuteResponse {
    success: bool,
    command: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    stdout: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stderr: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    return_code: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    executed: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

#[derive(Serialize)]
struct AsyncExecuteResponse {
    success: bool,
    message: Option<String>,
    command: String,
    pid: u32,
    started_at: String,
    status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

#[derive(Serialize)]
struct HealthResponse {
    status: String,
    platform: String,
}

#[derive(Serialize)]
struct HomeResponse {
    message: String,
    endpoints: std::collections::HashMap<String, String>,
}

fn get_log_file_path() -> PathBuf {
    let exe_path = std::env::current_exe()
        .unwrap_or_else(|_| PathBuf::from("."));
    let default_path = PathBuf::from(".");
    let log_dir = exe_path.parent().unwrap_or(&default_path);
    log_dir.join("app_error.log")
}

fn log_error(endpoint: &str, error_msg: &str, command: Option<&str>) {
    let log_file = get_log_file_path();
    let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");
    
    let mut file = match OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_file) {
        Ok(f) => f,
        Err(_) => {
            eprintln!("Failed to open log file: {:?}", log_file);
            return;
        }
    };
    
    let log_entry = if let Some(cmd) = command {
        format!("{} - ERROR - {} - {} - Command: {}\n", timestamp, endpoint, error_msg, cmd)
    } else {
        format!("{} - ERROR - {} - {}\n", timestamp, endpoint, error_msg)
    };
    
    if let Err(e) = file.write_all(log_entry.as_bytes()) {
        eprintln!("Failed to write to log file: {}", e);
    }
}

fn log_error_with_traceback(endpoint: &str, error_msg: &str, traceback: &str, command: Option<&str>) {
    let log_file = get_log_file_path();
    let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");
    
    let mut file = match OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_file) {
        Ok(f) => f,
        Err(_) => {
            eprintln!("Failed to open log file: {:?}", log_file);
            return;
        }
    };
    
    let log_entry = if let Some(cmd) = command {
        format!("{} - ERROR - {} - {} - Command: {}\nTraceback: {}\n", 
                timestamp, endpoint, error_msg, cmd, traceback)
    } else {
        format!("{} - ERROR - {} - {}\nTraceback: {}\n", 
                timestamp, endpoint, error_msg, traceback)
    };
    
    if let Err(e) = file.write_all(log_entry.as_bytes()) {
        eprintln!("Failed to write to log file: {}", e);
    }
}

async fn home() -> ActixResult<HttpResponse> {
    let mut endpoints = std::collections::HashMap::new();
    endpoints.insert("/execute".to_string(), "POST - Execute a command and wait for response".to_string());
    endpoints.insert("/execute-async".to_string(), "POST - Execute a command asynchronously (fire and forget)".to_string());
    endpoints.insert("/health".to_string(), "GET - Check API health".to_string());
    
    Ok(HttpResponse::Ok().json(HomeResponse {
        message: "Machine Agent API".to_string(),
        endpoints,
    }))
}

async fn health() -> ActixResult<HttpResponse> {
    Ok(HttpResponse::Ok().json(HealthResponse {
        status: "healthy".to_string(),
        platform: std::env::consts::OS.to_string(),
    }))
}

async fn execute_command(req: web::Json<ExecuteRequest>) -> ActixResult<HttpResponse> {
    let command = req.command.trim();
    
    if command.is_empty() {
        let error_msg = "Command must be a non-empty string";
        log_error("/execute", error_msg, Some(command));
        return Ok(HttpResponse::BadRequest().json(ExecuteResponse {
            success: false,
            command: command.to_string(),
            stdout: None,
            stderr: None,
            return_code: None,
            executed: None,
            error: Some(error_msg.to_string()),
        }));
    }
    
    // Execute the command
    let output = if cfg!(target_os = "windows") {
        Command::new("cmd")
            .args(["/C", command])
            .current_dir(std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))
            .output()
    } else {
        Command::new("sh")
            .arg("-c")
            .arg(command)
            .current_dir(std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))
            .output()
    };
    
    match output {
        Ok(result) => {
            let stdout = String::from_utf8_lossy(&result.stdout).to_string();
            let stderr = String::from_utf8_lossy(&result.stderr).to_string();
            let return_code = result.status.code();
            
            Ok(HttpResponse::Ok().json(ExecuteResponse {
                success: true,
                command: command.to_string(),
                stdout: Some(stdout),
                stderr: Some(stderr),
                return_code,
                executed: Some(true),
                error: None,
            }))
        }
        Err(e) => {
            let error_msg = format!("Command execution failed: {}", e);
            log_error_with_traceback("/execute", &error_msg, &format!("{:?}", e), Some(command));
            Ok(HttpResponse::InternalServerError().json(ExecuteResponse {
                success: false,
                command: command.to_string(),
                stdout: None,
                stderr: None,
                return_code: None,
                executed: None,
                error: Some(e.to_string()),
            }))
        }
    }
}

async fn execute_command_async(req: web::Json<ExecuteRequest>) -> ActixResult<HttpResponse> {
    let command = req.command.trim();
    
    if command.is_empty() {
        let error_msg = "Command must be a non-empty string";
        log_error("/execute-async", error_msg, Some(command));
        return Ok(HttpResponse::BadRequest().json(AsyncExecuteResponse {
            success: false,
            message: None,
            command: command.to_string(),
            pid: 0,
            started_at: String::new(),
            status: String::new(),
            error: Some(error_msg.to_string()),
        }));
    }
    
    // Execute the command asynchronously
    let current_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let cmd = if cfg!(target_os = "windows") {
        TokioCommand::new("cmd")
            .args(["/C", command])
            .current_dir(&current_dir)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
    } else {
        TokioCommand::new("sh")
            .arg("-c")
            .arg(command)
            .current_dir(&current_dir)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
    };
    
    match cmd {
        Ok(mut child) => {
            let pid = child.id().unwrap_or(0);
            let started_at = Local::now().to_rfc3339();
            
            // Detach the process - don't wait for it
            tokio::spawn(async move {
                let _ = child.wait().await;
            });
            
            Ok(HttpResponse::Ok().json(AsyncExecuteResponse {
                success: true,
                message: Some("Command started successfully".to_string()),
                command: command.to_string(),
                pid,
                started_at,
                status: "running".to_string(),
                error: None,
            }))
        }
        Err(e) => {
            let error_msg = format!("Failed to start command: {}", e);
            log_error_with_traceback("/execute-async", &error_msg, &format!("{:?}", e), Some(command));
            Ok(HttpResponse::InternalServerError().json(AsyncExecuteResponse {
                success: false,
                message: None,
                command: command.to_string(),
                pid: 0,
                started_at: String::new(),
                status: String::new(),
                error: Some(e.to_string()),
            }))
        }
    }
}

fn print_logo() {
    println!("\n");
    println!("╔══════════════════════════════════════════════════════════╗");
    println!("║                                                          ║");
    println!("║         ███╗   ███╗ █████╗  ██████╗██╗  ██╗              ║");
    println!("║         ████╗ ████║██╔══██╗██╔════╝██║  ██║              ║");
    println!("║         ██╔████╔██║███████║██║     ███████║              ║");
    println!("║         ██║╚██╔╝██║██╔══██║██║     ██╔══██║              ║");
    println!("║         ██║ ╚═╝ ██║██║  ██║╚██████╗██║  ██║              ║");
    println!("║         ╚═╝     ╚═╝╚═╝  ╚═╝ ╚═════╝╚═╝  ╚═╝              ║");
    println!("║                                                          ║");
    println!("║              █████╗  ██████╗ ███████╗███╗   ██╗████████╗  ║");
    println!("║             ██╔══██╗██╔════╝ ██╔════╝████╗  ██║╚══██╔══╝  ║");
    println!("║             ███████║██║  ███╗█████╗  ██╔██╗ ██║   ██║     ║");
    println!("║             ██╔══██║██║   ██║██╔══╝  ██║╚██╗██║   ██║     ║");
    println!("║             ██║  ██║╚██████╔╝███████╗██║ ╚████║   ██║     ║");
    println!("║             ╚═╝  ╚═╝ ╚═════╝ ╚══════╝╚═╝  ╚═══╝   ╚═╝     ║");
    println!("║                                                           ║");
    println!("║                    Server Starting...            v.1.0 AV ║");
    println!("╚═══════════════════════════════════════════════════════════╝");
    println!("\n");
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    print_logo();
    println!("Error logs will be written to: app_error.log");
    
    HttpServer::new(|| {
        App::new()
            .route("/", web::get().to(home))
            .route("/health", web::get().to(health))
            .route("/execute", web::post().to(execute_command))
            .route("/execute-async", web::post().to(execute_command_async))
    })
    .bind("0.0.0.0:6565")?
    .run()
    .await
}
