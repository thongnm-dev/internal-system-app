//! Service nén file/folder thành ZIP (mã hoá AES-256 tuỳ chọn) rồi cắt thành
//! các phần `.001/.002/...` theo giới hạn dung lượng để gửi kèm email.
//!
//! Quy tắc quan trọng: sau khi nén xong, nếu dung lượng file zip **không vượt
//! quá** giới hạn đã nhập thì KHÔNG cắt — giữ nguyên 1 file zip duy nhất.

use crate::app::error::AppError;
use crate::app::result::AppResult;
use crate::models::file_split::{FileSplitOptions, FileSplitPart, FileSplitResult, FileSplitSource};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::{Path, PathBuf};
use zip::write::FileOptions;
use zip::{AesMode, CompressionMethod, ZipWriter};

/// Kích thước buffer khi copy/cắt file (1 MiB).
const COPY_BUFFER: usize = 1024 * 1024;

/// Nén các nguồn đã chọn rồi cắt nếu vượt giới hạn.
pub fn compress_and_split(options: FileSplitOptions) -> AppResult<FileSplitResult> {
    if options.sources.is_empty() {
        return Err(AppError::new("Chưa chọn nguồn nào để nén."));
    }

    let output_dir = PathBuf::from(options.output_dir.trim());
    if output_dir.as_os_str().is_empty() {
        return Err(AppError::new("Vui lòng chọn thư mục xuất."));
    }
    std::fs::create_dir_all(&output_dir).map_err(|e| {
        AppError::new(format!("Không tạo được thư mục xuất: {e}"))
    })?;

    for source in &options.sources {
        if !Path::new(&source.path).exists() {
            return Err(AppError::new(format!("Không tìm thấy nguồn: {}", source.path)));
        }
    }

    let encrypted = !options.password.trim().is_empty();

    // Xác định giới hạn cắt (byte). Khi gộp 1 file thì không cần giới hạn.
    let part_bytes: Option<u64> = if options.single_archive {
        None
    } else {
        match options.limit_mb {
            Some(mb) if mb > 0.0 => Some((mb * 1024.0 * 1024.0).round() as u64),
            _ => {
                return Err(AppError::new(
                    "Vui lòng nhập giới hạn dung lượng mỗi phần (MB) khi không gộp 1 file.",
                ))
            }
        }
    };

    // Gom nguồn thành các nhóm: 1 nhóm (gộp) hoặc mỗi nguồn 1 nhóm.
    let groups: Vec<(String, Vec<&FileSplitSource>)> = if options.single_archive {
        vec![(normalized_archive_name(&options.archive_name), options.sources.iter().collect())]
    } else {
        let mut used: HashMap<String, u32> = HashMap::new();
        options
            .sources
            .iter()
            .map(|s| (dedup_base_name(&s.name, &mut used), vec![s]))
            .collect()
    };

    let mut files: Vec<FileSplitPart> = Vec::new();
    let mut total_bytes: u64 = 0;
    let mut split_count: usize = 0;
    let archive_count = groups.len();

    for (base_name, sources) in groups {
        let zip_path = output_dir.join(format!("{base_name}.zip"));
        build_zip(&zip_path, &sources, &options.password)?;

        let zip_size = std::fs::metadata(&zip_path)?.len();

        // Tự động giữ 1 file khi zip không vượt quá giới hạn.
        let should_split = matches!(part_bytes, Some(limit) if zip_size > limit);

        if should_split {
            let limit = part_bytes.expect("đã kiểm tra ở matches!");
            let parts = split_archive(&zip_path, limit)?;
            std::fs::remove_file(&zip_path).ok();
            split_count += 1;
            for part in parts {
                let size = std::fs::metadata(&part)?.len();
                total_bytes += size;
                files.push(FileSplitPart { name: file_name(&part), size_bytes: size });
            }
        } else {
            total_bytes += zip_size;
            files.push(FileSplitPart { name: file_name(&zip_path), size_bytes: zip_size });
        }
    }

    Ok(FileSplitResult {
        output_dir: output_dir.display().to_string(),
        files,
        total_bytes,
        archive_count,
        split_count,
        encrypted,
    })
}

/// Chuẩn hoá tên file zip: bỏ đuôi `.zip` nếu có, fallback `attachment`.
fn normalized_archive_name(name: &str) -> String {
    let trimmed = name.trim().trim_end_matches(".zip").trim_end_matches(".ZIP");
    if trimmed.is_empty() {
        "attachment".to_string()
    } else {
        trimmed.to_string()
    }
}

/// Sinh tên cơ sở (không đuôi) cho chế độ mỗi-nguồn-1-zip, tránh trùng (`_2`, `_3`).
fn dedup_base_name(source_name: &str, used: &mut HashMap<String, u32>) -> String {
    let base = Path::new(source_name)
        .file_stem()
        .and_then(|s| s.to_str())
        .filter(|s| !s.is_empty())
        .unwrap_or(source_name)
        .to_string();
    let count = used.entry(base.clone()).or_insert(0);
    *count += 1;
    if *count == 1 {
        base
    } else {
        format!("{base}_{}", *count)
    }
}

/// Tạo file zip từ danh sách nguồn. Mã hoá AES-256 nếu `password` khác rỗng.
fn build_zip(zip_path: &Path, sources: &[&FileSplitSource], password: &str) -> AppResult<()> {
    let file = File::create(zip_path)
        .map_err(|e| AppError::new(format!("Không tạo được file zip: {e}")))?;
    let mut zip = ZipWriter::new(BufWriter::new(file));

    // Tuỳ chọn cho entry: deflate + (tuỳ chọn) AES-256. Lifetime của password
    // được suy ra từ &password ở đây nên phải giữ password sống trong hàm này.
    let base = FileOptions::<()>::default().compression_method(CompressionMethod::Deflated);
    let options = if password.is_empty() {
        base
    } else {
        base.with_aes_encryption(AesMode::Aes256, password)
    };

    for source in sources {
        let path = PathBuf::from(&source.path);
        // Prefix entry = tên nguồn (file/folder), giữ cấu trúc thư mục con.
        let parent = path.parent().map(Path::to_path_buf).unwrap_or_default();
        if path.is_dir() {
            add_dir(&mut zip, &parent, &path, options)?;
        } else {
            add_file(&mut zip, &parent, &path, options)?;
        }
    }

    zip.finish().map_err(|e| AppError::new(format!("Lỗi khi đóng file zip: {e}")))?;
    Ok(())
}

/// Thêm đệ quy một thư mục vào zip (giữ nguyên cây thư mục con, kể cả folder rỗng).
fn add_dir(
    zip: &mut ZipWriter<BufWriter<File>>,
    base_parent: &Path,
    dir: &Path,
    options: FileOptions<()>,
) -> AppResult<()> {
    let mut entries: Vec<PathBuf> = std::fs::read_dir(dir)?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .collect();
    entries.sort();

    if entries.is_empty() {
        // Folder rỗng: vẫn ghi entry thư mục để giữ cấu trúc.
        zip.add_directory(entry_name(base_parent, dir), options)
            .map_err(|e| AppError::new(format!("Lỗi ghi thư mục vào zip: {e}")))?;
        return Ok(());
    }

    for path in entries {
        if path.is_dir() {
            add_dir(zip, base_parent, &path, options)?;
        } else {
            add_file(zip, base_parent, &path, options)?;
        }
    }
    Ok(())
}

/// Thêm một file vào zip với tên entry tính tương đối theo `base_parent`.
fn add_file(
    zip: &mut ZipWriter<BufWriter<File>>,
    base_parent: &Path,
    path: &Path,
    options: FileOptions<()>,
) -> AppResult<()> {
    zip.start_file(entry_name(base_parent, path), options)
        .map_err(|e| AppError::new(format!("Lỗi ghi file vào zip: {e}")))?;
    let mut input = BufReader::new(File::open(path)?);
    std::io::copy(&mut input, zip)?;
    Ok(())
}

/// Tên entry trong zip: đường dẫn tương đối so với `base_parent`, dùng dấu `/`.
fn entry_name(base_parent: &Path, path: &Path) -> String {
    let relative = path.strip_prefix(base_parent).unwrap_or(path);
    relative
        .components()
        .map(|c| c.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}

/// Cắt file zip thành các phần `.001/.002/...` mỗi phần tối đa `part_bytes` byte.
fn split_archive(zip_path: &Path, part_bytes: u64) -> AppResult<Vec<PathBuf>> {
    let zip_name = file_name(zip_path);
    let mut input = BufReader::new(File::open(zip_path)?);
    let mut buffer = vec![0u8; COPY_BUFFER];
    let mut parts: Vec<PathBuf> = Vec::new();
    let mut index: u32 = 1;

    loop {
        let part_path = zip_path.with_file_name(format!("{zip_name}.{index:03}"));
        let mut out = BufWriter::new(File::create(&part_path)?);
        let mut written: u64 = 0;

        while written < part_bytes {
            let want = std::cmp::min(buffer.len() as u64, part_bytes - written) as usize;
            let read = input.read(&mut buffer[..want])?;
            if read == 0 {
                break;
            }
            out.write_all(&buffer[..read])?;
            written += read as u64;
        }
        out.flush()?;
        drop(out);

        if written == 0 {
            // Không còn dữ liệu (trường hợp tổng chia hết): xoá phần rỗng thừa.
            std::fs::remove_file(&part_path).ok();
            break;
        }

        parts.push(part_path);
        index += 1;

        if written < part_bytes {
            break; // đã tới EOF ở phần này
        }
    }

    Ok(parts)
}

/// Lấy tên file (không có thư mục) từ đường dẫn.
fn file_name(path: &Path) -> String {
    path.file_name()
        .and_then(|v| v.to_str())
        .unwrap_or_default()
        .to_string()
}
