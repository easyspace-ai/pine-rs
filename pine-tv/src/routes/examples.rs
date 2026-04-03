//! Example scripts API for pine-tv
//!
//! Provides endpoints to list and retrieve Pine Script example files
//! from the static/examples directory.

use axum::{
    extract::Path,
    http::StatusCode,
    response::Json,
    routing::{get, Router},
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Category of example scripts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExampleCategory {
    pub name: String,
    pub description: String,
    pub examples: Vec<ExampleInfo>,
}

/// Information about a single example script
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExampleInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: String,
}

/// Full example with code content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExampleContent {
    pub id: String,
    pub name: String,
    pub category: String,
    pub code: String,
}

/// List all available example scripts
pub async fn list_examples() -> Result<Json<Vec<ExampleCategory>>, StatusCode> {
    let mut categories = Vec::new();

    // Define categories with their descriptions
    let category_configs: Vec<(&str, &str)> = vec![
        ("ta", "Technical Analysis - Indicators and overlays"),
        ("math", "Math functions - Calculations and operations"),
        ("array", "Array operations - Collection handling"),
        ("map", "Map operations - Key-value storage"),
        ("color", "Color operations - Visual styling"),
        ("str", "String operations - Text manipulation"),
        ("input", "Input functions - User parameters"),
        ("control", "Control flow - If/for/while/switch"),
        ("udf", "User Defined Functions - Custom logic"),
        ("strategy", "Strategy - Trading strategies"),
    ];

    for (dir_name, description) in category_configs {
        match scan_category(dir_name, description).await {
            Ok(category) if !category.examples.is_empty() => {
                categories.push(category);
            }
            _ => continue,
        }
    }

    Ok(Json(categories))
}

/// Scan a category directory for example files
async fn scan_category(name: &str, description: &str) -> Result<ExampleCategory, std::io::Error> {
    let mut examples = Vec::new();
    // Try multiple paths to find examples directory
    let possible_paths = [
        PathBuf::from("pine-tv/static/examples").join(name),
        PathBuf::from("static/examples").join(name),
    ];

    let dir_path = possible_paths
        .iter()
        .find(|p| p.exists())
        .cloned()
        .unwrap_or_else(|| possible_paths[0].clone());

    if !dir_path.exists() {
        return Ok(ExampleCategory {
            name: name.to_string(),
            description: description.to_string(),
            examples,
        });
    }

    let mut entries = tokio::fs::read_dir(&dir_path).await?;
    let mut paths: Vec<PathBuf> = Vec::new();

    // Collect all entries
    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("pine") {
            paths.push(path);
        }
    }

    // Sort by file name
    paths.sort_by(|a, b| {
        a.file_name()
            .unwrap_or_default()
            .cmp(b.file_name().unwrap_or_default())
    });

    for path in paths {
        let file_name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();

        let id = format!("{}/{}", name, file_name);
        let display_name = format_display_name(&file_name);

        // Try to extract description from the file content
        let desc = extract_description(&path)
            .await
            .unwrap_or_else(|| format!("{} example", display_name));

        examples.push(ExampleInfo {
            id,
            name: display_name,
            description: desc,
            category: name.to_string(),
        });
    }

    Ok(ExampleCategory {
        name: name.to_string(),
        description: description.to_string(),
        examples,
    })
}

/// Format a file name like "basic_strategy" to "Basic Strategy"
fn format_display_name(file_name: &str) -> String {
    file_name
        .split('_')
        .map(|w| {
            let mut chars = w.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => {
                    let first_upper: String = first.to_uppercase().collect();
                    let rest: String = chars.as_str().to_lowercase();
                    format!("{}{}", first_upper, rest)
                }
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Extract description from the first line of a pine script
async fn extract_description(path: &PathBuf) -> Option<String> {
    let content = tokio::fs::read_to_string(path).await.ok()?;
    let first_line = content.lines().next()?;

    // Parse indicator/strategy title from //@version=X
    // Look for indicator("Title") or strategy("Title")
    if let Some(start) = first_line.find("indicator(") {
        let rest = &first_line[start + 10..];
        if let Some(end) = rest.find('"') {
            let after_first_quote = &rest[end + 1..];
            if let Some(end_quote) = after_first_quote.find('"') {
                return Some(after_first_quote[..end_quote].to_string());
            }
        }
    }

    if let Some(start) = first_line.find("strategy(") {
        let rest = &first_line[start + 9..];
        if let Some(end) = rest.find('"') {
            let after_first_quote = &rest[end + 1..];
            if let Some(end_quote) = after_first_quote.find('"') {
                return Some(after_first_quote[..end_quote].to_string());
            }
        }
    }

    None
}

/// Get a specific example by category and name
pub async fn get_example(
    Path((category, name)): Path<(String, String)>,
) -> Result<Json<ExampleContent>, StatusCode> {
    // Try multiple paths to find examples directory
    let possible_base_paths = [
        PathBuf::from("pine-tv/static/examples"),
        PathBuf::from("static/examples"),
    ];

    let file_path = possible_base_paths
        .iter()
        .map(|p| p.join(&category).join(format!("{}.pine", name)))
        .find(|p| p.exists())
        .ok_or(StatusCode::NOT_FOUND)?;

    if !file_path.exists() {
        return Err(StatusCode::NOT_FOUND);
    }

    let code = tokio::fs::read_to_string(&file_path)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let display_name = format_display_name(&name);

    Ok(Json(ExampleContent {
        id: format!("{}/{}", category, name),
        name: display_name,
        category,
        code,
    }))
}

/// Get example router
pub fn router() -> Router {
    Router::new()
        .route("/api/examples", get(list_examples))
        .route("/api/examples/:category/:name", get(get_example))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Method, Request};
    use tower::ServiceExt;

    #[tokio::test]
    async fn get_example_route_matches() {
        let app = router();
        let req = Request::builder()
            .method(Method::GET)
            .uri("/api/examples/ta/highest_lowest")
            .body(Body::empty())
            .unwrap();
        let response = app.oneshot(req).await.unwrap();
        assert_eq!(
            response.status(),
            StatusCode::OK,
            "GET /api/examples/:category/:name should return example file"
        );
    }

    #[test]
    fn test_display_name_formatting() {
        let name = "basic_strategy";
        let display = format_display_name(name);
        assert_eq!(display, "Basic Strategy");

        let name2 = "rsi_strategy";
        let display2 = format_display_name(name2);
        assert_eq!(display2, "Rsi Strategy");

        let name3 = "sma";
        let display3 = format_display_name(name3);
        assert_eq!(display3, "Sma");
    }
}
