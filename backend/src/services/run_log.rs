//! Windowed views over a stored run log.
//!
//! Callers fetch only a byte range from the database, then [`lines_from_chunk`]
//! snaps that range to whole lines so the API never materialises the full blob.

pub const DEFAULT_WINDOW_BYTES: usize = 64 * 1024;
pub const MAX_WINDOW_BYTES: usize = 256 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogQuery {
    FromStart { window_bytes: usize },
    FromEnd { window_bytes: usize },
    After { offset: usize, window_bytes: usize },
    Before { offset: usize, window_bytes: usize },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogLine {
    pub offset: usize,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogWindow {
    pub lines: Vec<LogLine>,
    pub total_bytes: usize,
    pub at_start: bool,
    pub at_end: bool,
    pub prev_offset: Option<usize>,
    pub next_offset: Option<usize>,
}

pub fn clamp_window(window_bytes: usize) -> usize {
    if window_bytes == 0 {
        return DEFAULT_WINDOW_BYTES;
    }
    window_bytes.min(MAX_WINDOW_BYTES)
}

/// 0-based `[start, start+len)` range to read from the stored log.
pub fn fetch_range(query: LogQuery, total: usize) -> (usize, usize) {
    if total == 0 {
        return (0, 0);
    }
    match query {
        LogQuery::FromStart { window_bytes } => (0, window_bytes.min(total)),
        LogQuery::FromEnd { window_bytes } => {
            let len = window_bytes.min(total);
            (total - len, len)
        }
        LogQuery::After {
            offset,
            window_bytes,
        } => {
            if offset >= total {
                return (total, 0);
            }
            (offset, window_bytes.min(total - offset))
        }
        LogQuery::Before {
            offset,
            window_bytes,
        } => {
            let end = offset.min(total);
            let start = end.saturating_sub(window_bytes);
            (start, end - start)
        }
    }
}

/// Snap a fetched chunk to whole lines and attach offsets in the full log.
///
/// `starts_at_line` is true when `chunk_start` is known to be the first byte
/// of a line (FromStart / After). FromEnd / Before may begin mid-line.
pub fn lines_from_chunk(
    chunk: &str,
    chunk_start: usize,
    total_bytes: usize,
    starts_at_line: bool,
) -> LogWindow {
    if total_bytes == 0 {
        return LogWindow {
            lines: Vec::new(),
            total_bytes,
            at_start: true,
            at_end: true,
            prev_offset: None,
            next_offset: None,
        };
    }

    let (snapped_start, snapped) = snap_to_lines(chunk, chunk_start, total_bytes, starts_at_line);
    let lines = split_lines(snapped, snapped_start);
    let at_start = snapped_start == 0;
    let consumed_end = match lines.last() {
        Some(line) => line.offset + line.text.len() + 1,
        None => snapped_start,
    };
    let at_end = consumed_end >= total_bytes || chunk_start + chunk.len() >= total_bytes;
    let prev_offset = if at_start {
        None
    } else {
        lines.first().map(|line| line.offset)
    };
    let next_offset = if at_end {
        None
    } else {
        Some(consumed_end.min(total_bytes))
    };

    LogWindow {
        lines,
        total_bytes,
        at_start,
        at_end,
        prev_offset,
        next_offset,
    }
}

fn snap_to_lines(
    chunk: &str,
    chunk_start: usize,
    total_bytes: usize,
    starts_at_line: bool,
) -> (usize, &str) {
    let at_file_end = chunk_start + chunk.len() >= total_bytes;

    let mut local_start = 0usize;
    if !starts_at_line && chunk_start > 0 {
        match chunk.find('\n') {
            Some(i) => local_start = i + 1,
            None => return (chunk_start + chunk.len(), ""),
        }
    }

    let mut local_end = chunk.len();
    if !at_file_end {
        match chunk[local_start..].rfind('\n') {
            Some(i) => local_end = local_start + i,
            None if local_start == 0 => {}
            None => return (chunk_start + local_start, ""),
        }
    }

    (chunk_start + local_start, &chunk[local_start..local_end])
}

/// Slice the stored UTF-8 the same way Postgres `substr(convert_to(...), start, len)` does:
/// 0-based byte offset, then drop a split codepoint at either end.
pub fn slice_stored_log(log: &str, start: usize, len: usize) -> String {
    let bytes = log.as_bytes();
    if start >= bytes.len() || len == 0 {
        return String::new();
    }
    let end = (start + len).min(bytes.len());
    decode_chunk(&bytes[start..end])
}

pub fn starts_at_line(query: LogQuery) -> bool {
    matches!(query, LogQuery::FromStart { .. } | LogQuery::After { .. })
}

pub fn window_stored_log(log: &str, query: LogQuery) -> LogWindow {
    let (start, len) = fetch_range(query, log.len());
    let chunk = slice_stored_log(log, start, len);
    lines_from_chunk(&chunk, start, log.len(), starts_at_line(query))
}

pub fn decode_chunk(bytes: &[u8]) -> String {
    let mut start = 0;
    while start < bytes.len() && bytes[start] & 0b1100_0000 == 0b1000_0000 {
        start += 1;
    }
    let mut end = bytes.len();
    while end > start && std::str::from_utf8(&bytes[start..end]).is_err() {
        end -= 1;
    }
    String::from_utf8_lossy(&bytes[start..end]).into_owned()
}

fn split_lines(snapped: &str, start_offset: usize) -> Vec<LogLine> {
    if snapped.is_empty() {
        return Vec::new();
    }

    let mut offset = start_offset;
    let mut lines = Vec::new();
    for (i, text) in snapped.split('\n').enumerate() {
        if i > 0 {
            offset += 1;
        }
        if text.is_empty() && offset >= start_offset + snapped.len() {
            break;
        }
        lines.push(LogLine {
            offset,
            text: text.to_string(),
        });
        offset += text.len();
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    fn joined(window: &LogWindow) -> Vec<&str> {
        window.lines.iter().map(|line| line.text.as_str()).collect()
    }

    #[test]
    fn empty_log_is_at_both_ends() {
        let window = lines_from_chunk("", 0, 0, true);
        assert!(window.lines.is_empty());
        assert!(window.at_start);
        assert!(window.at_end);
        assert_eq!(window.prev_offset, None);
        assert_eq!(window.next_offset, None);
    }

    #[test]
    fn small_log_fits_in_one_window() {
        let log = "alpha\nbeta\ngamma";
        let (start, len) = fetch_range(
            LogQuery::FromStart {
                window_bytes: DEFAULT_WINDOW_BYTES,
            },
            log.len(),
        );
        let window = lines_from_chunk(&log[start..start + len], start, log.len(), true);
        assert_eq!(joined(&window), ["alpha", "beta", "gamma"]);
        assert!(window.at_start);
        assert!(window.at_end);
        assert_eq!(window.lines[0].offset, 0);
        assert_eq!(window.lines[1].offset, 6);
        assert_eq!(window.lines[2].offset, 11);
    }

    #[test]
    fn from_end_drops_a_leading_partial_line() {
        let log = "aaaa\nbbbb\ncccc\ndddd";
        let query = LogQuery::FromEnd { window_bytes: 10 };
        let (start, len) = fetch_range(query, log.len());
        let window = lines_from_chunk(&log[start..start + len], start, log.len(), false);
        assert!(window.at_end);
        assert!(!window.at_start);
        assert_eq!(joined(&window), ["cccc", "dddd"]);
        assert_eq!(window.prev_offset, Some(window.lines[0].offset));
    }

    #[test]
    fn after_offset_walks_forward() {
        let log = "one\ntwo\nthree\nfour";
        let query = LogQuery::After {
            offset: 4,
            window_bytes: 20,
        };
        let (start, len) = fetch_range(query, log.len());
        let window = lines_from_chunk(&log[start..start + len], start, log.len(), true);
        assert_eq!(joined(&window), ["two", "three", "four"]);
        assert!(window.at_end);
        assert_eq!(window.lines[0].offset, 4);
    }

    #[test]
    fn before_offset_walks_backward() {
        let log = "one\ntwo\nthree\nfour";
        let four = log.rfind("four").unwrap();
        let query = LogQuery::Before {
            offset: four,
            window_bytes: 10,
        };
        let (start, len) = fetch_range(query, log.len());
        let window = lines_from_chunk(&log[start..start + len], start, log.len(), false);
        assert!(joined(&window).contains(&"two") || joined(&window).contains(&"three"));
        assert!(!window.at_end);
        assert_eq!(window.next_offset, Some(four));
    }

    #[test]
    fn fetch_range_is_clamped_to_the_file() {
        assert_eq!(
            fetch_range(LogQuery::FromStart { window_bytes: 9 }, 4),
            (0, 4)
        );
        assert_eq!(
            fetch_range(LogQuery::After {
                offset: 10,
                window_bytes: 8
            }, 10),
            (10, 0)
        );
    }

    #[test]
    fn clamp_window_rejects_zero_and_caps_the_max() {
        assert_eq!(clamp_window(0), DEFAULT_WINDOW_BYTES);
        assert_eq!(clamp_window(MAX_WINDOW_BYTES + 10), MAX_WINDOW_BYTES);
    }

    #[test]
    fn character_substring_with_byte_offsets_misses_the_end() {
        let log = "café\n".repeat(20_000);
        let (start, len) = fetch_range(
            LogQuery::FromEnd {
                window_bytes: 1024,
            },
            log.len(),
        );
        let by_chars: String = log.chars().skip(start).take(len).collect();
        assert!(start > log.chars().count());
        assert!(by_chars.is_empty());
        assert!(!slice_stored_log(&log, start, len).is_empty());
    }

    #[test]
    fn from_end_of_a_multibyte_log_returns_the_last_lines() {
        let log = "échec opération café\n".repeat(20_000);
        assert!(log.len() > DEFAULT_WINDOW_BYTES);
        assert!(log.len() > log.chars().count());

        let window = window_stored_log(
            &log,
            LogQuery::FromEnd {
                window_bytes: DEFAULT_WINDOW_BYTES,
            },
        );
        assert!(!window.lines.is_empty());
        assert!(window.at_end);
        assert!(window
            .lines
            .last()
            .is_some_and(|line| line.text.contains("café") || line.text.contains("échec")));
    }

    #[test]
    fn decode_chunk_keeps_whole_characters_and_drops_a_split_prefix() {
        assert_eq!(decode_chunk("é".as_bytes()), "é");
        let cafe = "café".as_bytes();
        assert_eq!(&cafe[3..], "é".as_bytes());
        let continuation = &cafe[4..];
        assert_eq!(continuation[0] & 0b1100_0000, 0b1000_0000);
        assert_eq!(decode_chunk(continuation), "");
    }
}
