use std::io::{self, BufRead, Write};
use std::path::PathBuf;

use contextclean_core::{
    build_report, clean_text, read_source_with_options, render_report, render_result, CleanMode,
    CleanOptions, FitModel, OutputFormat, ReadOptions, ReportOptions,
};
use serde_json::{json, Value};

use crate::support::CliSupportError;

const MCP_PROTOCOL_VERSION: &str = "2025-11-25";

pub fn run_mcp_server() -> Result<(), CliSupportError> {
    let stdin = io::stdin();
    let mut stdout = io::stdout().lock();

    for line in stdin.lock().lines() {
        let line = line.map_err(|error| CliSupportError::Render(error.to_string()))?;
        if line.trim().is_empty() {
            continue;
        }

        let request: Value = match serde_json::from_str(&line) {
            Ok(request) => request,
            Err(error) => {
                write_response(&mut stdout, parse_error(error.to_string()))?;
                continue;
            }
        };

        if request.get("id").is_none() {
            continue;
        }

        let response = handle_request(&request);
        write_response(&mut stdout, response)?;

        if request.get("method").and_then(Value::as_str) == Some("shutdown") {
            break;
        }
    }

    Ok(())
}

fn handle_request(request: &Value) -> Value {
    let id = request.get("id").cloned().unwrap_or(Value::Null);
    let Some(method) = request.get("method").and_then(Value::as_str) else {
        return error_response(id, -32600, "invalid request");
    };

    match method {
        "initialize" => success_response(id, initialize_result()),
        "ping" => success_response(id, json!({})),
        "tools/list" => success_response(id, tools_list_result()),
        "tools/call" => match call_tool(request.get("params").unwrap_or(&Value::Null)) {
            Ok(result) => success_response(id, result),
            Err(message) => error_response(id, -32602, &message),
        },
        "shutdown" => success_response(id, Value::Null),
        _ => error_response(id, -32601, "method not found"),
    }
}

fn call_tool(params: &Value) -> Result<Value, String> {
    let name = params
        .get("name")
        .and_then(Value::as_str)
        .ok_or_else(|| "tools/call requires params.name".to_string())?;
    let arguments = params.get("arguments").unwrap_or(&Value::Null);

    match name {
        "contextclean_clean" | "ctxclean_clean" => {
            render_clean_tool(arguments).map(tool_text_result)
        }
        "contextclean_report" | "ctxclean_report" => {
            render_report_tool(arguments).map(tool_text_result)
        }
        _ => Err(format!("unknown tool: {name}")),
    }
}

fn render_clean_tool(arguments: &Value) -> Result<String, String> {
    let source = source_from_arguments(arguments)?;
    let options = options_from_arguments(arguments, source.name.clone())?;
    let mut result = clean_text(&source.content, &options);
    result.warnings.extend(source.warnings);
    render_result(&result, options.format).map_err(|error| error.to_string())
}

fn render_report_tool(arguments: &Value) -> Result<String, String> {
    let source = source_from_arguments(arguments)?;
    let options = options_from_arguments(arguments, source.name.clone())?;
    let mut result = clean_text(&source.content, &options);
    result.warnings.extend(source.warnings);
    let report = build_report(
        &result,
        &ReportOptions {
            source_arg: source.name.clone(),
            mode: options.mode,
            format: options.format,
            max_tokens: arguments
                .get("maxTokens")
                .or_else(|| arguments.get("max_tokens"))
                .and_then(Value::as_u64)
                .map(|value| value as usize),
            strip_comments: options.strip_comments,
            include_sensitive: arguments
                .get("includeSensitive")
                .or_else(|| arguments.get("include_sensitive"))
                .and_then(Value::as_bool)
                .unwrap_or(false),
        },
    );
    render_report(&report, options.format).map_err(|error| error.to_string())
}

fn source_from_arguments(arguments: &Value) -> Result<McpSource, String> {
    let text = arguments
        .get("text")
        .or_else(|| arguments.get("input"))
        .and_then(Value::as_str);
    let path = arguments.get("path").and_then(Value::as_str);

    match (text, path) {
        (Some(_), Some(_)) | (None, None) => {
            return Err("tool arguments require exactly one of text/input or path".to_string())
        }
        _ => {}
    }

    if let Some(input) = text {
        return Ok(McpSource {
            name: arguments
                .get("sourceName")
                .or_else(|| arguments.get("source"))
                .and_then(Value::as_str)
                .map(ToOwned::to_owned)
                .or_else(|| Some("mcp-input".to_string())),
            content: input.to_string(),
            warnings: Vec::new(),
        });
    }

    if let Some(path) = path {
        if path == "-" {
            return Err(
                "MCP path input cannot be '-' because stdin is the protocol stream".to_string(),
            );
        }
        let include_sensitive = arguments
            .get("includeSensitive")
            .or_else(|| arguments.get("include_sensitive"))
            .and_then(Value::as_bool)
            .unwrap_or(false);
        let source = read_source_with_options(
            Some(&PathBuf::from(path)),
            &ReadOptions { include_sensitive },
        )
        .map_err(|error| error.to_string())?;
        return Ok(McpSource {
            name: source.name,
            content: source.content,
            warnings: source.warnings,
        });
    }

    unreachable!("source arguments were validated above")
}

fn options_from_arguments(
    arguments: &Value,
    source_name: Option<String>,
) -> Result<CleanOptions, String> {
    let mode = match arguments
        .get("mode")
        .and_then(Value::as_str)
        .unwrap_or("standard")
    {
        "light" => CleanMode::Light,
        "standard" => CleanMode::Standard,
        "aggressive" => CleanMode::Aggressive,
        other => return Err(format!("unsupported mode: {other}")),
    };
    let format = match arguments
        .get("format")
        .and_then(Value::as_str)
        .unwrap_or("markdown")
    {
        "text" => OutputFormat::Text,
        "markdown" => OutputFormat::Markdown,
        "json" => OutputFormat::Json,
        other => return Err(format!("unsupported format: {other}")),
    };
    let fit = match arguments.get("fit").and_then(Value::as_str) {
        Some("gpt-4.1") => Some(FitModel::Gpt41),
        Some("claude-sonnet") => Some(FitModel::ClaudeSonnet),
        Some("gemini-pro") => Some(FitModel::GeminiPro),
        Some(other) => return Err(format!("unsupported fit: {other}")),
        None => None,
    };
    let max_tokens = arguments
        .get("maxTokens")
        .or_else(|| arguments.get("max_tokens"))
        .and_then(Value::as_u64)
        .map(|value| value as usize)
        .or_else(|| fit.map(FitModel::max_tokens));

    Ok(CleanOptions {
        mode,
        format,
        max_tokens,
        fit,
        strip_comments: arguments
            .get("stripComments")
            .or_else(|| arguments.get("strip_comments"))
            .and_then(Value::as_bool)
            .unwrap_or(false),
        redact_secrets: arguments
            .get("redactSecrets")
            .or_else(|| arguments.get("redact_secrets"))
            .and_then(Value::as_bool)
            .unwrap_or(true),
        source_name,
    })
}

fn initialize_result() -> Value {
    json!({
        "protocolVersion": MCP_PROTOCOL_VERSION,
        "serverInfo": {
            "name": "contextclean",
            "title": "ContextClean MCP Server",
            "version": env!("CARGO_PKG_VERSION")
        },
        "capabilities": {
            "tools": {
                "listChanged": false
            }
        },
        "instructions": "Use contextclean_clean to clean text or files before giving context to an AI model; use contextclean_report to explain token savings and noise sources."
    })
}

fn tools_list_result() -> Value {
    json!({
        "tools": [
            {
                "name": "contextclean_clean",
                "description": "Clean noisy text, HTML, logs, or local files into model-ready context.",
                "inputSchema": tool_schema()
            },
            {
                "name": "contextclean_report",
                "description": "Return token savings, noise sources, removed sections, and a recommended command.",
                "inputSchema": tool_schema()
            }
        ]
    })
}

fn tool_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "text": { "type": "string", "description": "Raw text/HTML/log content to clean." },
            "input": { "type": "string", "description": "Alias for text." },
            "path": { "type": "string", "description": "Local file or directory path to read." },
            "sourceName": { "type": "string" },
            "mode": { "type": "string", "enum": ["light", "standard", "aggressive"] },
            "format": { "type": "string", "enum": ["text", "markdown", "json"] },
            "maxTokens": { "type": "integer", "minimum": 5 },
            "fit": { "type": "string", "enum": ["gpt-4.1", "claude-sonnet", "gemini-pro"] },
            "stripComments": { "type": "boolean" },
            "redactSecrets": { "type": "boolean" },
            "includeSensitive": { "type": "boolean" }
        },
        "oneOf": [
            { "required": ["text"] },
            { "required": ["input"] },
            { "required": ["path"] }
        ]
    })
}

fn tool_text_result(text: String) -> Value {
    json!({
        "content": [
            {
                "type": "text",
                "text": text
            }
        ],
        "isError": false
    })
}

fn success_response(id: Value, result: Value) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": result
    })
}

fn error_response(id: Value, code: i64, message: &str) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "error": {
            "code": code,
            "message": message
        }
    })
}

fn parse_error(message: String) -> Value {
    error_response(Value::Null, -32700, &message)
}

fn write_response(writer: &mut impl Write, response: Value) -> Result<(), CliSupportError> {
    writeln!(writer, "{response}").map_err(|error| CliSupportError::Render(error.to_string()))?;
    writer
        .flush()
        .map_err(|error| CliSupportError::Render(error.to_string()))
}

struct McpSource {
    name: Option<String>,
    content: String,
    warnings: Vec<contextclean_core::Warning>,
}
