use serde::Serialize;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Serialize)]
pub struct XlsxMarkdownResult {
    source_path: String,
    output_path: String,
    source_file_name: String,
    output_file_name: String,
    markdown: String,
}

#[tauri::command]
pub fn convert_xlsx_spec_to_markdown(
    input_path: String,
    output_path: Option<String>,
) -> Result<XlsxMarkdownResult, String> {
    let input = PathBuf::from(input_path.trim());
    if input.as_os_str().is_empty() {
        return Err("Please select an Excel workbook before converting.".to_string());
    }

    if !input.exists() {
        return Err(format!("Excel workbook not found: {}", input.display()));
    }

    if !matches!(input.extension().and_then(|value| value.to_str()), Some("xlsx")) {
        return Err("Only .xlsx workbooks are supported.".to_string());
    }

    let output = output_path
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| input.with_extension("md"));

    let script = repository_root()?.join("scripts").join("xlsx_spec_to_markdown.py");
    if !script.exists() {
        return Err(format!("Converter script not found: {}", script.display()));
    }

    run_converter(&script, &input, &output)?;

    let markdown = std::fs::read_to_string(&output)
        .map_err(|error| format!("Markdown was created but could not be read: {error}"))?;

    Ok(XlsxMarkdownResult {
        source_file_name: file_name(&input),
        output_file_name: file_name(&output),
        source_path: input.display().to_string(),
        output_path: output.display().to_string(),
        markdown,
    })
}

fn repository_root() -> Result<PathBuf, String> {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .map(Path::to_path_buf)
        .ok_or_else(|| "Could not resolve repository root.".to_string())
}

fn run_converter(script: &Path, input: &Path, output: &Path) -> Result<(), String> {
    let candidates: &[(&str, &[&str])] = if cfg!(target_os = "windows") {
        &[("python", &[]), ("py", &["-3"]), ("python3", &[])]
    } else {
        &[("python3", &[]), ("python", &[])]
    };

    let mut errors = Vec::new();

    for (program, prefix_args) in candidates {
        let mut command = Command::new(program);
        command.args(*prefix_args);
        command.arg(script).arg(input).arg("-o").arg(output);

        match command.output() {
            Ok(result) if result.status.success() => return Ok(()),
            Ok(result) => {
                let stderr = String::from_utf8_lossy(&result.stderr).trim().to_string();
                let stdout = String::from_utf8_lossy(&result.stdout).trim().to_string();
                let detail = if !stderr.is_empty() { stderr } else { stdout };
                errors.push(format!("{program}: {detail}"));
            }
            Err(error) => errors.push(format!("{program}: {error}")),
        }
    }

    Err(format!(
        "Could not run Python converter. Install Python 3 or check the converter script. Attempts: {}",
        errors.join(" | ")
    ))
}

fn file_name(path: &Path) -> String {
    path.file_name()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
        .to_string()
}
