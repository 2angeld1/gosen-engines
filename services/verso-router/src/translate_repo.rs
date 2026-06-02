use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::Semaphore;
use walkdir::WalkDir;

#[derive(Debug)]
pub struct RepoRequest {
    pub repo_url: String,
    pub source_lang: String,
    pub target_lang: String,
    pub branch: Option<String>,
    pub gemini_key: Option<String>,
    pub cohere_key: Option<String>,
    pub gemini_model: Option<String>,
    pub cohere_model: Option<String>,
    pub db: crate::db::DbPool,
}

#[derive(Debug, Serialize)]
pub struct FileResult {
    pub path: String,
    pub translated: bool,
    pub error: Option<String>,
    pub method: Option<String>,
    pub lines_input: usize,
    pub lines_output: usize,
}

#[derive(Debug, Serialize)]
pub struct RepoResponse {
    pub total_files: usize,
    pub translated_files: usize,
    pub failed_files: usize,
    pub files: Vec<FileResult>,
    pub errors: Vec<String>,
}

fn get_extensions(lang: &str) -> &[&str] {
    match lang {
        "php" => &["php", "phtml", "php3", "php4", "php5", "php7", "php8"],
        "python" => &["py", "pyw"],
        "javascript" => &["js", "jsx", "mjs", "cjs"],
        "typescript" => &["ts", "tsx"],
        "rust" => &["rs"],
        "go" => &["go"],
        "java" => &["java"],
        "csharp" => &["cs"],
        "ruby" => &["rb"],
        "cpp" => &["cpp", "cc", "cxx", "hpp", "hxx", "c++"],
        "kotlin" => &["kt", "kts"],
        "cobol" => &["cob", "cbl", "cpy"],
        _ => &[],
    }
}

async fn clone_repo(url: &str, dest: &Path, branch: Option<&str>) -> Result<(), String> {
    let mut cmd = std::process::Command::new("git");
    cmd.arg("clone")
        .arg("--depth")
        .arg("1")
        .arg(url)
        .arg(dest);

    if let Some(b) = branch {
        cmd.arg("--branch").arg(b);
    }

    let output = cmd.output().map_err(|e| format!("git clone failed: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("git clone error: {}", stderr.trim()));
    }

    Ok(())
}

fn find_source_files(root: &Path, exts: &[&str]) -> Vec<PathBuf> {
    WalkDir::new(root)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter(|e| {
            e.path().extension()
                .and_then(|ext| ext.to_str())
                .map_or(false, |ext| exts.contains(&ext))
        })
        .filter(|e| {
            let p = e.path();
            !p.components().any(|c| {
                c.as_os_str().to_str().map_or(false, |s| {
                    s.starts_with('.') || s == "node_modules" || s == "vendor"
                        || s == "__pycache__" || s == "target"
                })
            })
        })
        .map(|e| e.path().to_path_buf())
        .collect()
}

pub async fn run(req: RepoRequest) -> Result<RepoResponse, String> {
    let source_lang = crate::languages::normalize(&req.source_lang);
    let target_lang = crate::languages::normalize(&req.target_lang);
    let exts = get_extensions(&source_lang);

    if exts.is_empty() {
        return Err(format!("Unsupported source language: {}", req.source_lang));
    }

    tracing::info!(url = %req.repo_url, "cloning repo");
    let tmp = tempfile::tempdir().map_err(|e| format!("tempdir: {}", e))?;
    let repo_path = tmp.path().join("repo");

    clone_repo(&req.repo_url, &repo_path, req.branch.as_deref()).await?;

    let files = find_source_files(&repo_path, exts);
    let total = files.len();

    tracing::info!(count = total, "source files found");

    if total == 0 {
        return Ok(RepoResponse {
            total_files: 0,
            translated_files: 0,
            failed_files: 0,
            files: vec![],
            errors: vec![],
        });
    }

    let semaphore = Arc::new(Semaphore::new(4));
    let mut handles = Vec::new();

    for file_path in files {
        let source_lang = source_lang.clone();
        let target_lang = target_lang.clone();
        let req_gemini_key = req.gemini_key.clone();
        let req_cohere_key = req.cohere_key.clone();
        let req_gemini_model = req.gemini_model.clone();
        let req_cohere_model = req.cohere_model.clone();
        let db_pool = req.db.clone();
        let permit = Arc::clone(&semaphore);

        let handle = tokio::spawn(async move {
            let _permit = permit.acquire_owned().await.unwrap();
            translate_file(
                &file_path,
                &source_lang,
                &target_lang,
                req_gemini_key.as_deref(),
                req_cohere_key.as_deref(),
                req_gemini_model.as_deref(),
                req_cohere_model.as_deref(),
                &db_pool,
            )
            .await
        });

        handles.push(handle);
    }

    let mut files_result = Vec::new();
    let mut errors = Vec::new();
    let mut translated = 0;
    let mut failed = 0;

    for handle in handles {
        match handle.await {
            Ok(result) => {
                let translated_count = &mut translated;
                let failed_count = &mut failed;
                if result.translated {
                    *translated_count += 1;
                } else {
                    *failed_count += 1;
                }
                if let Some(ref err) = result.error {
                    errors.push(format!("{}: {}", result.path, err));
                }
                files_result.push(result);
            }
            Err(e) => {
                failed += 1;
                errors.push(format!("task panic: {}", e));
            }
        }
    }

    Ok(RepoResponse {
        total_files: total,
        translated_files: translated,
        failed_files: failed,
        files: files_result,
        errors,
    })
}

async fn translate_file(
    path: &Path,
    source_lang: &str,
    target_lang: &str,
    gemini_key: Option<&str>,
    cohere_key: Option<&str>,
    gemini_model: Option<&str>,
    cohere_model: Option<&str>,
    db: &crate::db::DbPool,
) -> FileResult {
    let rel_path = path.to_string_lossy().to_string();
    let source = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            return FileResult {
                path: rel_path,
                translated: false,
                error: Some(format!("read: {}", e)),
                method: None,
                lines_input: 0,
                lines_output: 0,
            };
        }
    };

    let lines_input = source.lines().count();

    let req = crate::translate::Request {
        source: source.clone(),
        source_lang: source_lang.to_string(),
        target_lang: target_lang.to_string(),
        source_version: None,
        target_version: None,
        gemini_key: gemini_key.map(|s| s.to_string()),
        cohere_key: cohere_key.map(|s| s.to_string()),
        gemini_model: gemini_model.map(|s| s.to_string()),
        cohere_model: cohere_model.map(|s| s.to_string()),
        db: db.clone(),
    };

    tracing::info!(path = %rel_path, "translating file");
    match crate::translate::run(req).await {
        Ok(resp) => {
            tracing::info!(path = %rel_path, method = %resp.method, "file translated");
            FileResult {
                path: rel_path,
                translated: true,
                error: None,
                method: Some(resp.method),
                lines_input,
                lines_output: resp.lines_output,
            }
        }
        Err(e) => {
            tracing::warn!(path = %rel_path, error = %e, "file translation failed");
            FileResult {
                path: rel_path,
                translated: false,
                error: Some(e),
                method: None,
                lines_input,
                lines_output: 0,
            }
        }
    }
}
