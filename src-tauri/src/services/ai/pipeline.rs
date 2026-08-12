use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PipelineKind {
    Text,
    Pdf,
    Image,
    Audio,
    Video,
    Unsupported,
}

const MAX_CHARS: usize = 32_000;
const HEAD_SIZE: usize = 20_000;
const TAIL_SIZE: usize = 12_000;

/// Guesses which AI pipeline a file extension belongs to.
pub fn classify(extension: &str) -> PipelineKind {
    match extension.trim_start_matches('.').to_lowercase().as_str() {
        "pdf" => PipelineKind::Pdf,
        "jpg" | "jpeg" | "png" | "gif" | "bmp" | "webp" | "tiff" | "tif" | "heic" | "heif"
        | "svg" | "avif" => PipelineKind::Image,
        "mp3" | "wav" | "flac" | "ogg" | "m4a" | "aac" | "wma" | "opus" => PipelineKind::Audio,
        "mp4" | "mkv" | "avi" | "mov" | "webm" | "wmv" | "flv" | "m4v" => PipelineKind::Video,
        "txt" | "md" | "markdown" | "log" | "json" | "jsonl" | "yml" | "yaml" | "xml" | "html"
        | "htm" | "csv" | "tsv" | "toml" | "ini" | "cfg" | "conf" | "py" | "js" | "jsx" | "ts"
        | "tsx" | "rs" | "java" | "c" | "h" | "cpp" | "hpp" | "cc" | "cxx" | "go" | "rb"
        | "php" | "sh" | "bat" | "cmd" | "ps1" | "css" | "scss" | "sql" | "doc" | "docx" | "rtf"
        | "tex" | "bib" => PipelineKind::Text,
        _ => PipelineKind::Unsupported,
    }
}

/// Pulls readable text out of a file, handling PDFs and Word documents.
pub fn extract_text(path: &Path, kind: PipelineKind) -> Result<String, String> {
    match kind {
        PipelineKind::Pdf => pdf_extract::extract_text(path).map_err(|e| e.to_string()),
        PipelineKind::Text => match path.extension().and_then(|e| e.to_str()) {
            Some(ext) if ext.eq_ignore_ascii_case("docx") => extract_docx(path),
            _ => {
                let bytes = fs::read(path).map_err(|e| e.to_string())?;
                Ok(String::from_utf8_lossy(&bytes).into_owned())
            }
        },
        _ => Ok(String::new()),
    }
}

/// Trims long text down to a manageable size by keeping the start and end.
pub fn truncate_head_tail(text: &str) -> String {
    if text.chars().count() <= MAX_CHARS {
        return text.to_string();
    }
    let head: String = text.chars().take(HEAD_SIZE).collect();
    let tail: String = text
        .chars()
        .rev()
        .take(TAIL_SIZE)
        .collect::<Vec<char>>()
        .into_iter()
        .rev()
        .collect();
    format!("{head}\n\n'TEXT TRUNCATED FOR BREVITY'\n\n{tail}")
}

/// Reads a Word document and flattens its text into a single string.
fn extract_docx(path: &Path) -> Result<String, String> {
    let bytes = fs::read(path).map_err(|e| e.to_string())?;
    let document = docx_rs::read_docx(&bytes).map_err(|e| format!("{e:?}"))?;
    let mut out = String::new();
    append_document_children(&document.document.children, &mut out);
    Ok(out)
}

/// Walks the top-level Word document parts, grabbing text from paragraphs and tables.
fn append_document_children(children: &[docx_rs::DocumentChild], out: &mut String) {
    for child in children {
        match child {
            docx_rs::DocumentChild::Paragraph(paragraph) => append_paragraph(paragraph, out),
            docx_rs::DocumentChild::Table(table) => append_table(table, out),
            _ => {}
        }
    }
}

/// Digs through a Word table row by row, pulling text out of every cell.
fn append_table(table: &docx_rs::Table, out: &mut String) {
    for row_child in &table.rows {
        let docx_rs::TableChild::TableRow(row) = row_child;
        for cell_child in &row.cells {
            let docx_rs::TableRowChild::TableCell(cell) = cell_child;
            for content in &cell.children {
                match content {
                    docx_rs::TableCellContent::Paragraph(paragraph) => {
                        append_paragraph(paragraph, out)
                    }
                    docx_rs::TableCellContent::Table(inner) => append_table(inner, out),
                    _ => {}
                }
            }
        }
    }
}

/// Copies a paragraph's runs into the buffer, ending with a newline.
fn append_paragraph(paragraph: &docx_rs::Paragraph, out: &mut String) {
    for child in &paragraph.children {
        if let docx_rs::ParagraphChild::Run(run) = child {
            append_run(run, out);
        }
    }
    out.push('\n');
}

/// Copies a run's text into the buffer, translating tabs and breaks into whitespace.
fn append_run(run: &docx_rs::Run, out: &mut String) {
    for child in &run.children {
        match child {
            docx_rs::RunChild::Text(text) => out.push_str(&text.text),
            docx_rs::RunChild::Tab(_) => out.push('\t'),
            docx_rs::RunChild::Break(_) | docx_rs::RunChild::CarriageReturn(_) => {
                out.push('\n')
            }
            _ => {}
        }
    }
}