use crate::app::result::AppResult;
use crate::services::xlsx_markdown::models::XlsxMarkdownResult;
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn convert(input_path: String, output_path: Option<String>) -> AppResult<XlsxMarkdownResult> {
    let input = PathBuf::from(input_path.trim());
    if input.as_os_str().is_empty() {
        return Err(crate::app::error::AppError::new(
            "Please select an Excel workbook before converting.",
        ));
    }

    if !input.exists() {
        return Err(crate::app::error::AppError::new(format!(
            "Excel workbook not found: {}",
            input.display()
        )));
    }

    if !matches!(
        input.extension().and_then(|value| value.to_str()),
        Some("xlsx")
    ) {
        return Err(crate::app::error::AppError::new(
            "Only .xlsx workbooks are supported.",
        ));
    }

    let output = output_path
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| input.with_extension("md"));

    let script = repository_root()?.join("scripts").join("xlsx_spec_to_markdown.py");
    if !script.exists() {
        return Err(crate::app::error::AppError::new(format!(
            "Converter script not found: {}",
            script.display()
        )));
    }

    run_converter(&script, &input, &output)?;

    let markdown = std::fs::read_to_string(&output).map_err(|error| {
        crate::app::error::AppError::new(format!(
            "Markdown was created but could not be read: {error}"
        ))
    })?;

    Ok(XlsxMarkdownResult {
        source_file_name: file_name(&input),
        output_file_name: file_name(&output),
        source_path: input.display().to_string(),
        output_path: output.display().to_string(),
        markdown,
    })
}

fn repository_root() -> AppResult<PathBuf> {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .map(Path::to_path_buf)
        .ok_or_else(|| {
            crate::app::error::AppError::new("Could not resolve repository root.")
        })
}

fn run_converter(script: &Path, input: &Path, output: &Path) -> AppResult<()> {
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

    Err(crate::app::error::AppError::new(format!(
        "Could not run Python converter. Install Python 3 or check the converter script. Attempts: {}",
        errors.join(" | ")
    )))
}

fn file_name(path: &Path) -> String {
    path.file_name()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
        .to_string()
}
