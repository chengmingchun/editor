// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

use serde::{Deserialize, Serialize};
use tauri::{generate_handler, Manager, State, WindowBuilder, WindowUrl};

// ==================== Data Structures ====================

/// CodeReview 意见结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewComment {
    pub id: String,
    pub author: String,
    pub content: String,
    pub file_path: Option<String>,
    pub line_number: Option<u32>,
    pub severity: String, // critical, warning, suggestion
    pub created_at: String,
}

/// RAG 处理后的 Review 数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RAGReviewPair {
    pub error_logic: String,
    pub fix_suggestion: String,
    pub source_comment: ReviewComment,
}

/// 效能指标数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricData {
    pub date: String,
    pub ai_generate_time_minutes: f64,
    pub ai_lines_of_code: u32,
    pub manual_lines_of_code: u32,
    pub review_comments_count: u32,
    pub resolved_comments_count: u32,
}

/// 应用状态
pub struct AppState {
    pub documents_dir: Mutex<PathBuf>,
    pub review_data: Mutex<Vec<ReviewComment>>,
    pub rag_pairs: Mutex<Vec<RAGReviewPair>>,
    pub metrics: Mutex<Vec<MetricData>>,
}

impl AppState {
    fn new() -> Self {
        let documents_dir = dirs::document_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("AI_Flow_Studio");
        
        // 确保目录存在
        let _ = fs::create_dir_all(&documents_dir);
        
        Self {
            documents_dir: Mutex::new(documents_dir),
            review_data: Mutex::new(Vec::new()),
            rag_pairs: Mutex::new(Vec::new()),
            metrics: Mutex::new(Vec::new()),
        }
    }
}

// ==================== Document Commands ====================

/// 保存文档到本地
#[tauri::command]
fn save_document(
    filename: String,
    content: String,
    state: State<AppState>,
) -> Result<String, String> {
    let dir = state.documents_dir.lock().map_err(|e| e.to_string())?;
    let file_path = dir.join(format!("{}.md", filename));
    
    fs::write(&file_path, content).map_err(|e| format!("保存失败: {}", e))?;
    
    Ok(format!("文档已保存至: {}", file_path.display()))
}

/// 加载本地文档
#[tauri::command]
fn load_document(
    filename: String,
    state: State<AppState>,
) -> Result<String, String> {
    let dir = state.documents_dir.lock().map_err(|e| e.to_string())?;
    let file_path = dir.join(format!("{}.md", filename));
    
    let content = fs::read_to_string(&file_path)
        .map_err(|e| format!("读取失败: {}", e))?;
    
    Ok(content)
}

/// 获取所有已保存的文档列表
#[tauri::command]
fn list_documents(state: State<AppState>) -> Result<Vec<String>, String> {
    let dir = state.documents_dir.lock().map_err(|e| e.to_string())?;
    
    let mut files = Vec::new();
    if let Ok(entries) = fs::read_dir(&*dir) {
        for entry in entries.flatten() {
            if let Some(name) = entry.file_name().to_str() {
                if name.ends_with(".md") {
                    files.push(name.trim_end_matches(".md").to_string());
                }
            }
        }
    }
    
    Ok(files)
}

// ==================== CodeHub Webview Commands ====================

/// 创建 CodeHub Webview 窗口
#[tauri::command]
async fn open_codehub_window(app_handle: tauri::AppHandle, mr_url: String) -> Result<String, String> {
    let window = WindowBuilder::new(
        &app_handle,
        "codehub",
        WindowUrl::External(mr_url.parse().map_err(|e| format!("URL解析错误: {:?}", e))?),
    )
    .title("CodeHub - MR Review")
    .inner_size(1400.0, 900.0)
    .center()
    .build()
    .map_err(|e| format!("创建窗口失败: {}", e))?;

    Ok("CodeHub 窗口已打开".to_string())
}

/// 关闭 CodeHub Webview 窗口
#[tauri::command]
async fn close_codehub_window(app_handle: tauri::AppHandle) -> Result<(), String> {
    if let Some(window) = app_handle.get_window("codehub") {
        window.close().map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// 从 Webview 抓取检视意见
#[tauri::command]
async fn capture_review_comments(
    app_handle: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<Vec<ReviewComment>, String> {
    use tauri::Manager;
    
    // 获取 CodeHub 窗口
    let window = app_handle.get_window("codehub").ok_or("CodeHub 窗口未打开")?;
    
    // 创建一个一次性监听器来接收 JavaScript 发送的结果
    let (tx, rx) = std::sync::mpsc::channel::<String>();
    let tx = std::sync::Mutex::new(Some(tx));
    
    // 设置事件监听器
    let app_handle_for_event = app_handle.clone();
    let _id = app_handle_for_event.listen_global("__capture_comments_result__", move |event| {
        if let Some(payload) = event.payload() {
            if let Some(sender) = tx.lock().unwrap().take() {
                let _ = sender.send(payload.to_string());
            }
        }
    });
    
    // 执行 JavaScript 抓取评论并通过事件发送回 Rust
    let script = r#"
        (function() {
            const comments = [];
            const commentElements = document.querySelectorAll('.comment, .review-comment, [data-testid="comment"], .note-body, .diff-comment');
            
            commentElements.forEach((el, index) => {
                const authorEl = el.querySelector('.author, .username, [data-testid="author"], .user-avatar');
                const contentEl = el.querySelector('.comment-content, .note-text, p, .markdown-body');
                const fileEl = el.closest('.file-holder, .diff-file')?.querySelector('.file-title, .filename');
                const lineEl = el.closest('.line, .diff-line')?.querySelector('.line-number');
                
                if (contentEl) {
                    comments.push({
                        id: 'comment_' + index + '_' + Date.now(),
                        author: authorEl ? authorEl.textContent.trim() : 'Unknown',
                        content: contentEl.textContent.trim(),
                        file_path: fileEl ? fileEl.textContent.trim() : null,
                        line_number: lineEl ? parseInt(lineEl.textContent) : null,
                        severity: el.classList.contains('critical') ? 'critical' : 
                                  el.classList.contains('warning') ? 'warning' : 'suggestion',
                        created_at: new Date().toISOString()
                    });
                }
            });
            
            window.__TAURI__.event.emit('__capture_comments_result__', JSON.stringify(comments));
        })()
    "#;

    window
        .eval(script)
        .map_err(|e| format!("执行脚本失败: {}", e))?;
    
    // 等待 JavaScript 发送结果（最多等待 5 秒）
    let result = rx.recv_timeout(std::time::Duration::from_secs(5))
        .map_err(|_| "获取评论超时或失败")?;

    let comments: Vec<ReviewComment> = serde_json::from_str(&result)
        .map_err(|e| format!("解析评论失败: {}", e))?;

    // 先获取结果，然后再 lock state 存储到应用状态
    let mut review_data = state.review_data.lock().map_err(|e| e.to_string())?;
    review_data.extend(comments.clone());

    Ok(comments)
}

/// 手动添加检视意见
#[tauri::command]
fn add_review_comment(
    comment: ReviewComment,
    state: State<AppState>,
) -> Result<(), String> {
    let mut review_data = state.review_data.lock().map_err(|e| e.to_string())?;
    review_data.push(comment);
    Ok(())
}

/// 获取所有检视意见
#[tauri::command]
fn get_review_comments(state: State<AppState>) -> Result<Vec<ReviewComment>, String> {
    let review_data = state.review_data.lock().map_err(|e| e.to_string())?;
    Ok(review_data.clone())
}

/// 清空检视意见
#[tauri::command]
fn clear_review_comments(state: State<AppState>) -> Result<(), String> {
    let mut review_data = state.review_data.lock().map_err(|e| e.to_string())?;
    review_data.clear();
    Ok(())
}

// ==================== RAG Processing Commands ====================

/// 处理检视意见为 RAG 格式
#[tauri::command]
fn process_rag_pairs(state: State<AppState>) -> Result<Vec<RAGReviewPair>, String> {
    let review_data = state.review_data.lock().map_err(|e| e.to_string())?;
    let mut rag_pairs = state.rag_pairs.lock().map_err(|e| e.to_string())?;
    
    rag_pairs.clear();
    
    for comment in review_data.iter() {
        // 智能解析评论内容，提取错误逻辑和修复建议
        let (error_logic, fix_suggestion) = parse_comment_to_rag(&comment.content);
        
        rag_pairs.push(RAGReviewPair {
            error_logic,
            fix_suggestion,
            source_comment: comment.clone(),
        });
    }
    
    Ok(rag_pairs.clone())
}

/// 解析评论内容为 RAG 格式
fn parse_comment_to_rag(content: &str) -> (String, String) {
    // 简单的启发式解析逻辑
    let lines: Vec<&str> = content.lines().collect();
    
    let mut error_logic = String::new();
    let mut fix_suggestion = String::new();
    
    let mut in_error_section = true;
    
    for line in lines {
        let line_lower = line.to_lowercase();
        
        if line_lower.contains("建议") || line_lower.contains("fix") || line_lower.contains("should") {
            in_error_section = false;
        }
        
        if in_error_section {
            if !error_logic.is_empty() {
                error_logic.push(' ');
            }
            error_logic.push_str(line.trim());
        } else {
            if !fix_suggestion.is_empty() {
                fix_suggestion.push(' ');
            }
            fix_suggestion.push_str(line.trim());
        }
    }
    
    // 如果没有明确的分隔，默认前半部分是错误描述
    if fix_suggestion.is_empty() && error_logic.len() > 50 {
        let mid = error_logic.len() / 2;
        let split_pos = error_logic[mid..].find(' ').map(|p| mid + p).unwrap_or(mid);
        fix_suggestion = error_logic.split_off(split_pos);
    }
    
    if error_logic.is_empty() {
        error_logic = content.to_string();
    }
    if fix_suggestion.is_empty() {
        fix_suggestion = "请根据上述问题检查并修复代码".to_string();
    }
    
    (error_logic, fix_suggestion)
}

/// 获取 RAG 对
#[tauri::command]
fn get_rag_pairs(state: State<AppState>) -> Result<Vec<RAGReviewPair>, String> {
    let rag_pairs = state.rag_pairs.lock().map_err(|e| e.to_string())?;
    Ok(rag_pairs.clone())
}

/// 导出 RAG 数据为 JSON
#[tauri::command]
fn export_rag_data(state: State<AppState>) -> Result<String, String> {
    let dir = state.documents_dir.lock().map_err(|e| e.to_string())?;
    let rag_pairs = state.rag_pairs.lock().map_err(|e| e.to_string())?;
    
    let json_content = serde_json::to_string_pretty(&*rag_pairs)
        .map_err(|e| format!("序列化失败: {}", e))?;
    
    let file_path = dir.join("rag_review_data.json");
    fs::write(&file_path, json_content).map_err(|e| format!("写入失败: {}", e))?;
    
    Ok(file_path.to_string_lossy().to_string())
}

// ==================== Metrics Commands ====================

/// 添加效能指标
#[tauri::command]
fn add_metric(metric: MetricData, state: State<AppState>) -> Result<(), String> {
    let mut metrics = state.metrics.lock().map_err(|e| e.to_string())?;
    metrics.push(metric);
    Ok(())
}

/// 获取所有效能指标
#[tauri::command]
fn get_metrics(state: State<AppState>) -> Result<Vec<MetricData>, String> {
    let metrics = state.metrics.lock().map_err(|e| e.to_string())?;
    Ok(metrics.clone())
}

/// 获取效能统计摘要
#[tauri::command]
fn get_metrics_summary(state: State<AppState>) -> Result<HashMap<String, f64>, String> {
    let metrics = state.metrics.lock().map_err(|e| e.to_string())?;
    
    let mut summary = HashMap::new();
    
    if metrics.is_empty() {
        summary.insert("total_sessions".to_string(), 0.0);
        summary.insert("avg_ai_time".to_string(), 0.0);
        summary.insert("total_ai_lines".to_string(), 0.0);
        summary.insert("total_manual_lines".to_string(), 0.0);
        summary.insert("ai_efficiency".to_string(), 0.0);
        summary.insert("total_reviews".to_string(), 0.0);
        summary.insert("resolved_rate".to_string(), 0.0);
        return Ok(summary);
    }
    
    let total_sessions = metrics.len() as f64;
    let avg_ai_time: f64 = metrics.iter().map(|m| m.ai_generate_time_minutes).sum::<f64>() / total_sessions;
    let total_ai_lines: f64 = metrics.iter().map(|m| m.ai_lines_of_code as f64).sum();
    let total_manual_lines: f64 = metrics.iter().map(|m| m.manual_lines_of_code as f64).sum();
    let total_reviews: f64 = metrics.iter().map(|m| m.review_comments_count as f64).sum();
    let total_resolved: f64 = metrics.iter().map(|m| m.resolved_comments_count as f64).sum();
    
    let ai_efficiency = if total_manual_lines > 0.0 {
        (total_ai_lines / (total_ai_lines + total_manual_lines)) * 100.0
    } else {
        100.0
    };
    
    let resolved_rate = if total_reviews > 0.0 {
        (total_resolved / total_reviews) * 100.0
    } else {
        0.0
    };
    
    summary.insert("total_sessions".to_string(), total_sessions);
    summary.insert("avg_ai_time".to_string(), avg_ai_time);
    summary.insert("total_ai_lines".to_string(), total_ai_lines);
    summary.insert("total_manual_lines".to_string(), total_manual_lines);
    summary.insert("ai_efficiency".to_string(), ai_efficiency);
    summary.insert("total_reviews".to_string(), total_reviews);
    summary.insert("resolved_rate".to_string(), resolved_rate);
    
    Ok(summary)
}

/// 清空效能数据
#[tauri::command]
fn clear_metrics(state: State<AppState>) -> Result<(), String> {
    let mut metrics = state.metrics.lock().map_err(|e| e.to_string())?;
    metrics.clear();
    Ok(())
}

// ==================== Remote Template Commands ====================

/// 模板数据结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Template {
    pub id: String,
    pub name: String,
    pub description: String,
    pub content: String,
}

/// 从远程接口获取模板列表
#[tauri::command]
async fn fetch_remote_templates(url: String) -> Result<Vec<Template>, String> {
    // 使用 reqwest 或其他 HTTP 客户端获取远程模板
    // 这里先返回模拟数据用于演示
    
    // 模拟网络延迟
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    
    // 如果是特定 URL，返回演示模板
    if url.contains("demo") || url.contains("example") {
        let templates = vec![
            Template {
                id: "demo-api-design".to_string(),
                name: "API 设计规范".to_string(),
                description: "RESTful API 设计规范模板".to_string(),
                content: "# API 设计规范\n\n## 设计原则\n1. 使用名词复数形式表示资源\n2. 使用 HTTP 方法表示操作类型\n3. 使用状态码表示请求结果".to_string(),
            },
            Template {
                id: "demo-db-design".to_string(),
                name: "数据库设计规范".to_string(),
                description: "数据库表结构设计模板".to_string(),
                content: "# 数据库设计规范\n\n## 命名规范\n- 表名：小写下划线，复数形式\n- 字段名：小写下划线\n- 索引名：idx_表名_字段名".to_string(),
            },
            Template {
                id: "demo-prd-template".to_string(),
                name: "PRD 文档模板".to_string(),
                description: "产品需求文档标准模板".to_string(),
                content: "# PRD 文档\n\n## 1. 背景\n描述项目背景和目标\n\n## 2. 需求描述\n详细描述功能需求\n\n## 3. 验收标准\n- [ ] 标准1\n- [ ] 标准2".to_string(),
            },
        ];
        return Ok(templates);
    }
    
    // 实际项目中，这里应该使用 HTTP 客户端获取远程数据
    // 例如：
    // let response = reqwest::get(&url).await.map_err(|e| e.to_string())?;
    // let templates: Vec<Template> = response.json().await.map_err(|e| e.to_string())?;
    
    Err("暂不支持该地址，请使用包含 'demo' 或 'example' 的 URL 查看演示".to_string())
}

// ==================== AI PlantUML Commands ====================

/// 根据描述生成 PlantUML 代码
#[tauri::command]
async fn generate_plantuml(description: String) -> Result<String, String> {
    // 模拟 AI 生成延迟
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    
    // 根据描述关键词生成不同类型的图表
    let desc_lower = description.to_lowercase();
    
    let plantuml = if desc_lower.contains("登录") || desc_lower.contains("认证") || desc_lower.contains("login") {
        r#"@startuml
!theme plain
skinparam backgroundColor #FEFEFE

actor 用户
participant "前端" as FE
participant "认证服务" as Auth
database "用户数据库" as DB

用户 -> FE: 输入账号密码
FE -> Auth: 发送登录请求
Auth -> DB: 查询用户信息
DB --> Auth: 返回用户数据
Auth -> Auth: 验证密码

alt 验证成功
    Auth --> FE: 返回 Token
    FE --> 用户: 登录成功
else 验证失败
    Auth --> FE: 返回错误信息
    FE --> 用户: 显示错误提示
end

@enduml"#.to_string()
    } else if desc_lower.contains("订单") || desc_lower.contains("下单") || desc_lower.contains("order") {
        r#"@startuml
!theme plain
skinparam backgroundColor #FEFEFE

actor 用户
participant "订单服务" as Order
participant "库存服务" as Stock
participant "支付服务" as Pay
database "订单数据库" as DB

用户 -> Order: 创建订单
Order -> Stock: 检查库存
Stock --> Order: 库存充足
Order -> DB: 保存订单(待支付)
Order --> 用户: 订单创建成功

用户 -> Pay: 发起支付
Pay -> DB: 更新订单状态
DB --> Pay: 更新成功
Pay --> 用户: 支付成功

@enduml"#.to_string()
    } else if desc_lower.contains("类") || desc_lower.contains("class") {
        r#"@startuml
!theme plain
skinparam backgroundColor #FEFEFE

class User {
  - id: Long
  - username: String
  - email: String
  + login(): Boolean
  + logout(): void
}

class Order {
  - id: Long
  - userId: Long
  - totalAmount: BigDecimal
  - status: OrderStatus
  + pay(): Boolean
  + cancel(): Boolean
}

class Product {
  - id: Long
  - name: String
  - price: BigDecimal
  - stock: Integer
  + checkStock(): Boolean
}

User "1" --> "*" Order : 拥有
Order "*" --> "1" Product : 包含

@enduml"#.to_string()
    } else {
        // 默认流程图
        r#"@startuml
!theme plain
skinparam backgroundColor #FEFEFE

title 业务流程图

|用户|
:start;
:输入需求;

|系统|
:处理请求;
if (验证通过?) then (是)
  :执行业务逻辑;
  :返回成功结果;
else (否)
  :返回错误信息;
endif

|用户|
:查看结果;
:结束;

stop

@enduml"#.to_string()
    };
    
    Ok(plantuml)
}

// ==================== Main ====================

fn main() {
    tauri::Builder::default()
        .manage(AppState::new())
        .invoke_handler(generate_handler![
            // Document commands
            save_document,
            load_document,
            list_documents,
            // CodeHub commands
            open_codehub_window,
            close_codehub_window,
            capture_review_comments,
            add_review_comment,
            get_review_comments,
            clear_review_comments,
            // RAG commands
            process_rag_pairs,
            get_rag_pairs,
            export_rag_data,
            // Metrics commands
            add_metric,
            get_metrics,
            get_metrics_summary,
            clear_metrics,
            // Template commands
            fetch_remote_templates,
            // AI commands
            generate_plantuml,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
