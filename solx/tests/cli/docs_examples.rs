//!
//! Checks that every example in the CLI user guide produces the output the
//! guide shows. The guide itself is the test's input: a ```bash block whose
//! next code block is a ```text block forms a case — the commands are run
//! exactly as documented, and the output block is checked against what they
//! actually print.
//!
//! On mismatch, rerun with `SOLX_DOCS_BLESS=1` to rewrite the stale blocks
//! in place from the actual output, then review the documentation diff.
//!

use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

use pulldown_cmark::CodeBlockKind;
use pulldown_cmark::Event;
use pulldown_cmark::Parser;
use pulldown_cmark::Tag;
use pulldown_cmark::TagEnd;

/// The documented page, relative to this crate's manifest directory.
const DOC_PATH: &str = "../docs/src/user-guide/02-command-line-interface.md";

/// The directory with the input files the documented commands reference.
/// `Simple.sol` must not change: its keccak256 appears verbatim in the
/// documented metadata example.
const FIXTURES_PATH: &str = "../docs/examples";

///
/// A fenced code block of the documentation.
///
struct CodeBlock {
    /// The fence info string, e.g. `bash` or `text`.
    info: String,
    /// The block body.
    content: String,
    /// The body's line range within the documentation file.
    lines: std::ops::Range<usize>,
    /// The index of the enclosing heading.
    section: usize,
    /// The enclosing heading text, for messages.
    heading: String,
}

///
/// Extracts the fenced code blocks with their enclosing headings.
///
fn parse_doc(text: &str) -> Vec<CodeBlock> {
    let line_starts: Vec<usize> = std::iter::once(0)
        .chain(
            text.char_indices()
                .filter(|(_, c)| *c == '\n')
                .map(|(i, _)| i + 1),
        )
        .collect();
    let line_of = |byte: usize| line_starts.partition_point(|start| *start <= byte) - 1;

    let mut blocks = Vec::new();
    let mut section = 0;
    let mut heading = String::new();
    let mut in_heading = false;
    let mut heading_text = String::new();
    let mut heading_code: Option<String> = None;
    let mut block: Option<(String, String, Option<std::ops::Range<usize>>)> = None;

    for (event, range) in Parser::new(text).into_offset_iter() {
        match event {
            Event::Start(Tag::Heading { .. }) => {
                in_heading = true;
                heading_text.clear();
                heading_code = None;
            }
            Event::End(TagEnd::Heading(_)) => {
                in_heading = false;
                section += 1;
                // The option name is the heading's first code span, e.g.
                // `--yul` in "### `--yul` (or `--strict-assembly`)".
                heading = heading_code.take().unwrap_or_else(|| heading_text.clone());
            }
            Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(info))) => {
                block = Some((info.to_string(), String::new(), None));
            }
            Event::End(TagEnd::CodeBlock) => {
                if let Some((info, content, Some(bytes))) = block.take() {
                    blocks.push(CodeBlock {
                        info,
                        content,
                        lines: line_of(bytes.start)..line_of(bytes.end - 1) + 1,
                        section,
                        heading: heading.clone(),
                    });
                }
            }
            Event::Text(part) if block.is_some() => {
                let (_, content, bytes) = block.as_mut().expect("Always exists");
                content.push_str(part.as_ref());
                *bytes = Some(match bytes.take() {
                    Some(existing) => existing.start.min(range.start)..existing.end.max(range.end),
                    None => range,
                });
            }
            Event::Text(part) if in_heading => heading_text.push_str(part.as_ref()),
            Event::Code(code) if in_heading => {
                heading_code.get_or_insert_with(|| code.to_string());
            }
            _ => {}
        }
    }
    blocks
}

///
/// Splits a documented command line into tokens, honoring single and double
/// quotes (the guide quotes arguments, e.g. `solx 'Simple.sol' --bin`).
///
fn split_words(line: &str) -> Vec<String> {
    let mut words = Vec::new();
    let mut word = String::new();
    let mut in_word = false;
    let mut quote: Option<char> = None;
    for character in line.chars() {
        match quote {
            Some(closing) if character == closing => quote = None,
            Some(_) => word.push(character),
            None if character == '\'' || character == '"' => {
                quote = Some(character);
                in_word = true;
            }
            None if character.is_whitespace() => {
                if in_word {
                    words.push(std::mem::take(&mut word));
                    in_word = false;
                }
            }
            None => {
                word.push(character);
                in_word = true;
            }
        }
    }
    if in_word {
        words.push(word);
    }
    words
}

///
/// Whether a token is a `NAME=value` environment variable assignment.
///
fn env_assignment(token: &str) -> Option<(&str, &str)> {
    let (name, value) = token.split_once('=')?;
    let mut characters = name.chars();
    let first = characters.next()?;
    if !first.is_ascii_alphabetic() && first != '_' {
        return None;
    }
    if characters.all(|c| c.is_ascii_alphanumeric() || c == '_') {
        Some((name, value))
    } else {
        None
    }
}

///
/// Runs the documented commands of one case in `directory` and returns their
/// combined output. Supports `solx …` (the binary under test, stdout and
/// stderr combined, non-zero exits included) and `ls '<dir>'` (a sorted
/// listing, so the documented sessions run identically on all platforms),
/// with optional leading `NAME='value'` assignments.
///
fn run_case(commands: &str, directory: &Path) -> Result<String, String> {
    for artifacts in ["build", "debug"] {
        let _ = std::fs::remove_dir_all(directory.join(artifacts));
    }

    let mut output = String::new();
    for line in commands.lines().filter(|line| !line.trim().is_empty()) {
        let words = split_words(line);
        let mut env_vars = Vec::new();
        let mut command = words.as_slice();
        while let Some((name, value)) = command.first().and_then(|token| env_assignment(token)) {
            env_vars.push((name.to_owned(), value.to_owned()));
            command = &command[1..];
        }
        let command_words: Vec<&str> = command.iter().map(String::as_str).collect();
        match command_words.as_slice() {
            ["solx", args @ ..] => {
                let mut process =
                    Command::new(assert_cmd::cargo::cargo_bin!(env!("CARGO_PKG_NAME")));
                process.current_dir(directory).args(args);
                for (name, value) in env_vars.iter() {
                    process.env(name, value);
                }
                let result = process.output().map_err(|error| error.to_string())?;
                output.push_str(String::from_utf8_lossy(result.stdout.as_slice()).as_ref());
                output.push_str(String::from_utf8_lossy(result.stderr.as_slice()).as_ref());
            }
            ["ls", path] => {
                let mut names: Vec<String> = std::fs::read_dir(directory.join(path))
                    .map_err(|error| format!("ls {path}: {error}"))?
                    .map(|entry| {
                        entry
                            .expect("Always valid")
                            .file_name()
                            .to_string_lossy()
                            .into_owned()
                    })
                    .collect();
                names.sort();
                for name in names.into_iter() {
                    output.push_str(name.as_str());
                    output.push('\n');
                }
            }
            _ => return Err(format!("unsupported documented command: {line:?}")),
        }
    }
    Ok(output.replace('\r', ""))
}

///
/// Expands tabs to 8-column stops, matching how the documentation renders
/// compiler output.
///
fn expand_tabs(line: &str) -> String {
    let mut expanded = String::with_capacity(line.len());
    for character in line.chars() {
        if character == '\t' {
            expanded.push(' ');
            while !expanded.chars().count().is_multiple_of(8) {
                expanded.push(' ');
            }
        } else {
            expanded.push(character);
        }
    }
    expanded
}

///
/// Normalizes a line for comparison: tabs expanded, trailing whitespace
/// removed, and benchmark timings compared by label only.
///
fn normalize(line: &str) -> String {
    let line = expand_tabs(line);
    let line = line.trim_end();
    if let Some(position) = line.rfind(": ") {
        let value = &line[position + 2..];
        if let Some(digits) = value.strip_suffix("ms")
            && !digits.is_empty()
            && digits.bytes().all(|byte| byte.is_ascii_digit())
        {
            return format!("{}: <N>ms", &line[..position]);
        }
    }
    line.to_owned()
}

///
/// Whether a normalized documentation line matches a normalized output line,
/// treating `...` inside the line as a wildcard.
///
fn line_matches(doc_line: &str, out_line: &str) -> bool {
    if !doc_line.contains("...") {
        return doc_line == out_line;
    }
    // Trim only whitespace adjacent to the `...` itself, so that leading
    // indentation and other layout inside the parts stays significant.
    let raw: Vec<&str> = doc_line.split("...").collect();
    let last = raw.len() - 1;
    let parts: Vec<&str> = raw
        .iter()
        .enumerate()
        .map(|(index, part)| {
            let part = if index > 0 { part.trim_start() } else { part };
            if index < last { part.trim_end() } else { part }
        })
        .collect();
    let mut remainder = out_line;
    for (index, part) in parts.iter().enumerate() {
        if part.is_empty() {
            continue;
        }
        if index == 0 {
            match remainder.strip_prefix(part) {
                Some(rest) => remainder = rest,
                None => return false,
            }
        } else if index == parts.len() - 1 {
            if let Some(position) = remainder.rfind(part) {
                if position + part.len() != remainder.len() {
                    return false;
                }
                remainder = "";
            } else {
                return false;
            }
        } else {
            match remainder.find(part) {
                Some(position) => remainder = &remainder[position + part.len()..],
                None => return false,
            }
        }
    }
    true
}

///
/// Compares a documentation block against the case's output. Blank lines are
/// presentation and ignored on both sides. A line of `...` matches any run
/// of output lines. A block without any `...` must match the tail of the
/// output exactly — a case may open with output the block does not show
/// (e.g. the compiler's own output before a documented `ls`), but must not
/// end with unshown output, so listings stay complete.
///
fn match_block(doc_lines: &[&str], out_lines: &[&str]) -> Result<(), String> {
    let doc: Vec<String> = doc_lines
        .iter()
        .filter(|line| !line.trim().is_empty())
        .map(|line| normalize(line))
        .collect();
    let out: Vec<String> = out_lines
        .iter()
        .filter(|line| !line.trim().is_empty())
        .map(|line| normalize(line))
        .collect();

    if !doc.iter().any(|line| line.contains("...")) {
        if doc.len() > out.len() {
            return Err(format!(
                "block has {} line(s), output only {}",
                doc.len(),
                out.len()
            ));
        }
        let tail = &out[out.len() - doc.len()..];
        return match doc.iter().zip(tail.iter()).find(|(a, b)| a != b) {
            None => Ok(()),
            Some((doc_line, out_line)) => {
                Err(format!("expected {doc_line:?}, actual {out_line:?}"))
            }
        };
    }

    let mut position = 0;
    let mut floating = true;
    for doc_line in doc.iter() {
        if doc_line.trim() == "..." {
            floating = true;
            continue;
        }
        if floating {
            while position < out.len() && !line_matches(doc_line, &out[position]) {
                position += 1;
            }
            if position == out.len() {
                return Err(format!("line not found in output: {doc_line:?}"));
            }
            floating = false;
        } else if position == out.len() || !line_matches(doc_line, &out[position]) {
            let actual = out
                .get(position)
                .map(String::as_str)
                .unwrap_or("<end of output>");
            return Err(format!("expected {doc_line:?}, actual {actual:?}"));
        }
        position += 1;
    }
    Ok(())
}

///
/// Rewrites a stale documentation block from the case's output, preserving
/// the block's `...` elisions. Returns an error when a block cannot be
/// regenerated mechanically and needs a manual update.
///
fn bless_block(doc_lines: &[&str], out_lines: &[&str]) -> Result<Vec<String>, String> {
    let cleaned: Vec<String> = {
        let lines: Vec<String> = out_lines
            .iter()
            .map(|line| expand_tabs(line).trim_end().to_owned())
            .collect();
        let start = lines.iter().position(|line| !line.is_empty()).unwrap_or(0);
        let end = lines
            .iter()
            .rposition(|line| !line.is_empty())
            .map_or(0, |index| index + 1);
        lines[start..end].to_vec()
    };

    if !doc_lines.iter().any(|line| line.contains("...")) {
        // The block shows the output's tail. Try anchoring on the block's
        // first line, then a tail of the block's own length; a candidate
        // counts only if the rewritten block would verify.
        let out: Vec<&str> = cleaned.iter().map(String::as_str).collect();
        let first = doc_lines
            .iter()
            .find(|line| !line.trim().is_empty())
            .map(|line| normalize(line));
        let anchor = first.and_then(|first| out.iter().position(|line| normalize(line) == first));
        let doc_length = doc_lines
            .iter()
            .filter(|line| !line.trim().is_empty())
            .count();
        let length_tail = out.len().checked_sub(doc_length);
        for start in [anchor, length_tail].into_iter().flatten() {
            let candidate = &out[start..];
            if match_block(candidate, out.as_slice()).is_ok() {
                return Ok(candidate.iter().map(|line| (*line).to_owned()).collect());
            }
        }
        return Err(
            "the block's structure changed beyond mechanical regeneration; \
             update manually"
                .to_owned(),
        );
    }

    let out: Vec<String> = cleaned
        .iter()
        .filter(|line| !line.is_empty())
        .cloned()
        .collect();
    let mut blessed = Vec::with_capacity(doc_lines.len());
    let mut position = 0;
    let mut floating = true;
    for doc_line in doc_lines.iter() {
        if doc_line.trim().is_empty() {
            blessed.push((*doc_line).to_owned());
            continue;
        }
        if doc_line.trim() == "..." {
            blessed.push((*doc_line).to_owned());
            floating = true;
            continue;
        }
        let normalized = normalize(doc_line);
        if floating {
            let mut probe = position;
            while probe < out.len() && !line_matches(&normalized, &normalize(&out[probe])) {
                probe += 1;
            }
            if probe == out.len() {
                return Err(format!(
                    "cannot regenerate mechanically; update manually around {doc_line:?}"
                ));
            }
            position = probe;
            blessed.push((*doc_line).to_owned());
            floating = false;
        } else if position < out.len() && line_matches(&normalized, &normalize(&out[position])) {
            blessed.push((*doc_line).to_owned());
        } else if position < out.len() {
            let actual = &out[position];
            if !doc_line.contains("...") {
                blessed.push(actual.clone());
            } else if let Some(prefix) = doc_line.trim_end().strip_suffix("...") {
                // A pure truncation: keep the documented prefix length.
                if actual.len() < prefix.len() {
                    return Err(format!(
                        "output shorter than the documented truncation of {doc_line:?}"
                    ));
                }
                blessed.push(format!("{}...", &actual[..prefix.len()]));
            } else {
                return Err(format!(
                    "elided line changed; update manually: {doc_line:?}"
                ));
            }
        } else {
            return Err(format!(
                "output ended before the documented line {doc_line:?}"
            ));
        }
        position += 1;
    }
    Ok(blessed)
}

#[test]
fn docs_examples() -> anyhow::Result<()> {
    crate::common::setup()?;

    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let doc_path = manifest.join(DOC_PATH);
    let text = std::fs::read_to_string(doc_path.as_path())?;
    let lines: Vec<&str> = text.lines().collect();
    let blocks = parse_doc(text.as_str());

    let workspace = tempfile::tempdir()?;
    for entry in std::fs::read_dir(manifest.join(FIXTURES_PATH))? {
        let entry = entry?;
        if entry
            .path()
            .extension()
            .is_some_and(|extension| extension != "md")
        {
            std::fs::copy(entry.path(), workspace.path().join(entry.file_name()))?;
        }
    }

    let bless = std::env::var_os("SOLX_DOCS_BLESS").is_some();
    let mut failures: Vec<String> = Vec::new();
    let mut edits: Vec<(std::ops::Range<usize>, Vec<String>)> = Vec::new();
    let mut paired = vec![false; blocks.len()];

    for (index, usage) in blocks.iter().enumerate() {
        if usage.info != "bash" && usage.info != "shell" {
            continue;
        }
        let Some(expected) = blocks.get(index + 1) else {
            continue;
        };
        if expected.info != "text" || expected.section != usage.section {
            continue;
        }
        paired[index + 1] = true;
        let heading = usage.heading.as_str();

        let output = match run_case(usage.content.as_str(), workspace.path()) {
            Ok(output) => output,
            Err(error) => {
                failures.push(format!("`{heading}`: {error}"));
                continue;
            }
        };
        let doc_lines = &lines[expected.lines.clone()];
        let out_lines: Vec<&str> = output.lines().collect();
        if let Err(error) = match_block(doc_lines, out_lines.as_slice()) {
            if bless {
                // A blessed block must also verify against a second run, so
                // nondeterministic output (e.g. temp paths inside DWARF)
                // cannot be materialized into the documentation.
                let rerun = run_case(usage.content.as_str(), workspace.path())
                    .map_err(|error| format!("`{heading}`: {error}"));
                match rerun {
                    Ok(rerun) => {
                        let rerun_lines: Vec<&str> = rerun.lines().collect();
                        // The working directory can leak into output (e.g.
                        // hex-encoded inside DWARF) and differs on every run,
                        // so lines embedding it must stay behind `...`.
                        let directory = workspace.path().to_string_lossy();
                        let directory_hex: String = directory
                            .bytes()
                            .map(|byte| format!("{byte:02x}"))
                            .collect();
                        match bless_block(doc_lines, out_lines.as_slice()).and_then(|blessed| {
                            let blessed_refs: Vec<&str> =
                                blessed.iter().map(String::as_str).collect();
                            if blessed.iter().any(|line| {
                                line.contains(directory.as_ref())
                                    || line.contains(directory_hex.as_str())
                            }) {
                                return Err("output embeds the working directory; update \
                                     manually with appropriate `...` elisions"
                                    .to_owned());
                            }
                            match match_block(blessed_refs.as_slice(), rerun_lines.as_slice()) {
                                Ok(()) => Ok(blessed),
                                Err(_) => Err("output is not deterministic; update manually \
                                     with appropriate `...` elisions"
                                    .to_owned()),
                            }
                        }) {
                            Ok(blessed) => edits.push((expected.lines.clone(), blessed)),
                            Err(error) => failures.push(format!("`{heading}`: {error}")),
                        }
                    }
                    Err(error) => failures.push(error),
                }
            } else {
                failures.push(format!("`{heading}`: {error}"));
            }
        }
    }

    for (index, block) in blocks.iter().enumerate() {
        if block.info == "text" && !paired[index] {
            failures.push(format!(
                "`{}`: output block without a preceding usage block to reproduce it",
                block.heading
            ));
        }
    }

    if !edits.is_empty() {
        let mut rewritten: Vec<String> = lines.iter().map(|line| (*line).to_owned()).collect();
        edits.sort_by_key(|(range, _)| std::cmp::Reverse(range.start));
        let count = edits.len();
        for (range, blessed) in edits.into_iter() {
            rewritten.splice(range, blessed);
        }
        let mut updated = rewritten.join("\n");
        if text.ends_with('\n') {
            updated.push('\n');
        }
        std::fs::write(doc_path.as_path(), updated)?;
        println!("blessed {count} documentation block(s); review the diff");
    }

    assert!(
        failures.is_empty(),
        "documentation examples do not match the compiler output:\n  {}\n\
         rerun with `SOLX_DOCS_BLESS=1 cargo test -p solx --test mod docs_examples` \
         to regenerate the stale blocks in place",
        failures.join("\n  ")
    );
    Ok(())
}
