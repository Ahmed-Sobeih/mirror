//! Finite CLI-only lowercase-hex framing around unchanged signed V2 bytes.
//! This captures a checked local file before any authority owner is opened.

use std::{
    fs::{self, Metadata, OpenOptions},
    io::{self, Read},
    ops::Range,
    path::{Component, Path, PathBuf},
};

#[cfg(target_os = "linux")]
use std::os::unix::fs::{MetadataExt, OpenOptionsExt};

use mirror_node::v2::pool::MAX_POOL_OPERATION_BYTES_V2;

use super::{errors::Failure, signals::Signals};

pub(super) const MAX_INPUT_BYTES: usize = 149_248;
pub(super) const MAX_INPUT_COUNT: usize = 256;
const MAX_RECORD_HEX: usize = MAX_POOL_OPERATION_BYTES_V2 * 2;

pub(super) struct Snapshot {
    bytes: Vec<u8>,
    records: Vec<Range<usize>>,
}

impl Snapshot {
    pub(super) fn byte_len(&self) -> usize {
        self.bytes.len()
    }

    pub(super) fn len(&self) -> usize {
        self.records.len()
    }

    /// Framing was checked in full before A1 opening. Reuse one fixed stack
    /// buffer rather than retaining another collection of decoded payloads.
    pub(super) fn decode(
        &self,
        index: usize,
        decoded: &mut [u8; MAX_POOL_OPERATION_BYTES_V2],
    ) -> Result<usize, Failure> {
        let range = self
            .records
            .get(index)
            .ok_or_else(|| Failure::new("pool_invariant"))?;
        let record = self
            .bytes
            .get(range.clone())
            .ok_or_else(|| Failure::new("pool_invariant"))?;
        if record.is_empty() || record.len() > MAX_RECORD_HEX || record.len() % 2 != 0 {
            return Err(Failure::new("pool_invariant"));
        }
        for (output, pair) in decoded.iter_mut().zip(record.chunks_exact(2)) {
            let high = hex_nibble(pair[0]).ok_or_else(|| Failure::new("pool_invariant"))?;
            let low = hex_nibble(pair[1]).ok_or_else(|| Failure::new("pool_invariant"))?;
            *output = high * 16 + low;
        }
        Ok(record.len() / 2)
    }
}

pub(super) fn capture(
    path: &Path,
    max_bytes: usize,
    max_count: usize,
    signals: &Signals,
) -> Result<Snapshot, Failure> {
    if max_bytes == 0
        || max_bytes > MAX_INPUT_BYTES
        || max_count == 0
        || max_count > MAX_INPUT_COUNT
    {
        return Err(Failure::new("pool_invariant"));
    }
    check_stop(signals.requested())?;
    let path = checked_path(path)?;
    let bytes = capture_bytes(&path, max_bytes, signals)?;
    // capture_bytes owns and drops its descriptor, including on every failure.
    let records = frame(&bytes, max_count)?;
    check_stop(signals.requested())?;
    Ok(Snapshot { bytes, records })
}

fn checked_path(path: &Path) -> Result<PathBuf, Failure> {
    if path.as_os_str().is_empty() || path == Path::new("-") {
        return Err(Failure::new("pool_input_path"));
    }
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .map_err(|_| Failure::new("pool_input_io"))?
            .join(path)
    };
    let mut normalized = PathBuf::new();
    for component in absolute.components() {
        match component {
            Component::RootDir | Component::Normal(_) => normalized.push(component.as_os_str()),
            Component::CurDir => {}
            Component::ParentDir | Component::Prefix(_) => {
                return Err(Failure::new("pool_input_path"));
            }
        }
    }
    let parent = normalized
        .parent()
        .ok_or_else(|| Failure::new("pool_input_path"))?;
    let mut prefix = PathBuf::new();
    for component in parent.components() {
        prefix.push(component.as_os_str());
        let metadata = fs::symlink_metadata(&prefix).map_err(initial_io)?;
        if metadata.file_type().is_symlink() || !metadata.is_dir() {
            return Err(Failure::new("pool_input_unsafe"));
        }
    }
    Ok(normalized)
}

#[cfg(target_os = "linux")]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct FileIdentity {
    device: u64,
    inode: u64,
    length: u64,
    modified: i64,
    modified_ns: i64,
    changed: i64,
    changed_ns: i64,
}

#[cfg(target_os = "linux")]
impl FileIdentity {
    fn of(metadata: &Metadata) -> Option<Self> {
        if !metadata.is_file() || metadata.file_type().is_symlink() || metadata.nlink() != 1 {
            return None;
        }
        Some(Self {
            device: metadata.dev(),
            inode: metadata.ino(),
            length: metadata.len(),
            modified: metadata.mtime(),
            modified_ns: metadata.mtime_nsec(),
            changed: metadata.ctime(),
            changed_ns: metadata.ctime_nsec(),
        })
    }
}

#[cfg(target_os = "linux")]
fn capture_bytes(path: &Path, max_bytes: usize, signals: &Signals) -> Result<Vec<u8>, Failure> {
    let before = fs::symlink_metadata(path).map_err(initial_io)?;
    let expected = FileIdentity::of(&before).ok_or_else(|| Failure::new("pool_input_unsafe"))?;
    if expected.length > max_bytes as u64 {
        return Err(Failure::new("pool_input_bytes"));
    }
    let length =
        usize::try_from(expected.length).map_err(|_| Failure::new("arithmetic_overflow"))?;
    let mut file = OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK)
        .open(path)
        .map_err(changed_io)?;
    let opened = file.metadata().map_err(|_| Failure::new("pool_input_io"))?;
    if FileIdentity::of(&opened) != Some(expected) {
        return Err(Failure::new("pool_input_changed"));
    }
    let bytes = read_bounded(&mut file, length, || signals.requested())?;
    let after = file.metadata().map_err(|_| Failure::new("pool_input_io"))?;
    let current = fs::symlink_metadata(path).map_err(changed_io)?;
    if FileIdentity::of(&after) != Some(expected) || FileIdentity::of(&current) != Some(expected) {
        return Err(Failure::new("pool_input_changed"));
    }
    Ok(bytes)
}

#[cfg(not(target_os = "linux"))]
fn capture_bytes(_: &Path, _: usize, _: &Signals) -> Result<Vec<u8>, Failure> {
    Err(Failure::new("unsupported_platform"))
}

fn initial_io(error: io::Error) -> Failure {
    Failure::new(match error.kind() {
        io::ErrorKind::NotFound => "pool_input_missing",
        io::ErrorKind::NotADirectory => "pool_input_unsafe",
        _ => "pool_input_io",
    })
}

#[cfg(target_os = "linux")]
fn changed_io(error: io::Error) -> Failure {
    Failure::new(
        if matches!(
            error.kind(),
            io::ErrorKind::NotFound | io::ErrorKind::NotADirectory
        ) || error.raw_os_error() == Some(libc::ELOOP)
        {
            "pool_input_changed"
        } else {
            "pool_input_io"
        },
    )
}

/// One initialized, fallibly reserved payload plus an EOF/growth sentinel.
/// Stop is checked before I/O and interrupted retries; real I/O errors retain
/// precedence over a flag set during the same read. There is no I/O deadline.
fn read_bounded<R: Read>(
    reader: &mut R,
    length: usize,
    mut stopped: impl FnMut() -> bool,
) -> Result<Vec<u8>, Failure> {
    let amount = length
        .checked_add(1)
        .ok_or_else(|| Failure::new("arithmetic_overflow"))?;
    let mut bytes = Vec::new();
    bytes
        .try_reserve_exact(amount)
        .map_err(|_| Failure::new("allocation_failed"))?;
    bytes.resize(amount, 0);
    let mut read = 0;
    loop {
        check_stop(stopped())?;
        match reader.read(&mut bytes[read..]) {
            Ok(0) => {
                if read != length {
                    return Err(Failure::new("pool_input_changed"));
                }
                check_stop(stopped())?;
                bytes.truncate(length);
                return Ok(bytes);
            }
            Ok(count) => {
                if count > amount - read {
                    return Err(Failure::new("pool_invariant"));
                }
                read += count;
                if read > length {
                    return Err(Failure::new("pool_input_changed"));
                }
            }
            Err(error) if error.kind() == io::ErrorKind::Interrupted => {}
            Err(_) => return Err(Failure::new("pool_input_io")),
        }
    }
}

fn frame(bytes: &[u8], max_count: usize) -> Result<Vec<Range<usize>>, Failure> {
    let mut records = Vec::new();
    records
        .try_reserve_exact(max_count)
        .map_err(|_| Failure::new("allocation_failed"))?;
    let mut start = 0;
    for (position, &byte) in bytes.iter().enumerate() {
        if byte == b'\n' {
            let length = position - start;
            if length == 0 || length > MAX_RECORD_HEX || length % 2 != 0 {
                return Err(Failure::new("pool_input_framing"));
            }
            if records.len() == max_count {
                return Err(Failure::new("pool_input_count"));
            }
            records.push(start..position);
            start = position + 1;
        } else if position - start >= MAX_RECORD_HEX || hex_nibble(byte).is_none() {
            return Err(Failure::new("pool_input_framing"));
        }
    }
    if start != bytes.len() {
        return Err(Failure::new("pool_input_framing"));
    }
    Ok(records)
}

fn hex_nibble(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        _ => None,
    }
}

fn check_stop(stopped: bool) -> Result<(), Failure> {
    if stopped {
        Err(Failure::new("pool_shutdown"))
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{cell::Cell, collections::VecDeque};

    fn failure<T>(result: Result<T, Failure>) -> &'static str {
        match result {
            Ok(_) => panic!("expected refusal"),
            Err(error) => error.class,
        }
    }

    #[test]
    fn empty_and_canonical_records_keep_exact_ranges_and_decode_bytes() {
        assert!(
            frame(b"", 1)
                .unwrap_or_else(|_| panic!("empty framing"))
                .is_empty()
        );
        let bytes = b"00\n0123456789abcdef\nff\n".to_vec();
        let records = frame(&bytes, 3).unwrap_or_else(|_| panic!("canonical framing"));
        assert_eq!(records, [0..2, 3..19, 20..22]);
        let snapshot = Snapshot { bytes, records };
        let mut decoded = [0; MAX_POOL_OPERATION_BYTES_V2];
        assert_eq!(
            snapshot
                .decode(0, &mut decoded)
                .unwrap_or_else(|_| panic!("decode")),
            1
        );
        assert_eq!(decoded[0], 0);
        let length = snapshot
            .decode(1, &mut decoded)
            .unwrap_or_else(|_| panic!("decode"));
        assert_eq!(&decoded[..length], &[1, 35, 69, 103, 137, 171, 205, 239]);
        assert_eq!(
            snapshot
                .decode(2, &mut decoded)
                .unwrap_or_else(|_| panic!("decode")),
            1
        );
        assert_eq!(decoded[0], 255);
    }

    #[test]
    fn framing_rejects_noncanonical_bytes_and_missing_or_blank_records() {
        for bytes in [
            &b"\n"[..],
            &b"00\n\n"[..],
            &b"00"[..],
            &b"0\n"[..],
            &b"000\n"[..],
            &b"AB\n"[..],
            &b"aB\n"[..],
            &b"00\r\n"[..],
            &b" 00\n"[..],
            &b"00 \n"[..],
            &b"00\t\n"[..],
            &b"#00\n"[..],
            &b"gg\n"[..],
            &b"\xef\xbb\xbf00\n"[..],
            &b"00\nff"[..],
        ] {
            assert_eq!(failure(frame(bytes, MAX_INPUT_COUNT)), "pool_input_framing");
        }
    }

    #[test]
    fn record_size_and_count_caps_accept_the_exact_boundary() {
        let mut largest = vec![b'f'; MAX_RECORD_HEX];
        largest.push(b'\n');
        assert_eq!(
            frame(&largest, 1)
                .unwrap_or_else(|_| panic!("exact size"))
                .len(),
            1
        );
        let all = largest.repeat(MAX_INPUT_COUNT);
        assert_eq!(all.len(), MAX_INPUT_BYTES);
        assert_eq!(
            frame(&all, MAX_INPUT_COUNT)
                .unwrap_or_else(|_| panic!("exact count"))
                .len(),
            MAX_INPUT_COUNT
        );
        assert_eq!(
            failure(frame(&all, MAX_INPUT_COUNT - 1)),
            "pool_input_count"
        );
        largest.insert(0, b'0');
        largest.insert(0, b'0');
        assert_eq!(failure(frame(&largest, 1)), "pool_input_framing");
        assert_eq!(
            failure(frame(&b"00\n".repeat(MAX_INPUT_COUNT + 1), MAX_INPUT_COUNT)),
            "pool_input_count"
        );
    }

    enum ReadStep {
        Bytes(&'static [u8]),
        Error(io::ErrorKind),
    }

    struct ScriptedReader<'a> {
        steps: VecDeque<ReadStep>,
        calls: usize,
        largest_request: usize,
        stop_on_call: Option<(usize, &'a Cell<bool>)>,
    }

    impl<'a> ScriptedReader<'a> {
        fn new(steps: impl IntoIterator<Item = ReadStep>) -> Self {
            Self {
                steps: steps.into_iter().collect(),
                calls: 0,
                largest_request: 0,
                stop_on_call: None,
            }
        }
    }

    impl Read for ScriptedReader<'_> {
        fn read(&mut self, bytes: &mut [u8]) -> io::Result<usize> {
            self.calls += 1;
            self.largest_request = self.largest_request.max(bytes.len());
            if let Some((call, flag)) = self.stop_on_call {
                if self.calls == call {
                    flag.set(true);
                }
            }
            match self.steps.pop_front().expect("unexpected extra read") {
                ReadStep::Bytes(value) => {
                    assert!(value.len() <= bytes.len());
                    bytes[..value.len()].copy_from_slice(value);
                    Ok(value.len())
                }
                ReadStep::Error(kind) => Err(kind.into()),
            }
        }
    }

    #[test]
    fn bounded_reader_handles_short_reads_and_requires_explicit_eof() {
        let mut reader = ScriptedReader::new([
            ReadStep::Bytes(b"a"),
            ReadStep::Bytes(b"bc"),
            ReadStep::Bytes(b""),
        ]);
        assert_eq!(
            read_bounded(&mut reader, 3, || false).unwrap_or_else(|_| panic!("read")),
            b"abc"
        );
        assert_eq!(reader.calls, 3);
        assert_eq!(reader.largest_request, 4);
        assert!(reader.steps.is_empty());
        let mut empty = ScriptedReader::new([ReadStep::Bytes(b"")]);
        assert!(
            read_bounded(&mut empty, 0, || false)
                .unwrap_or_else(|_| panic!("empty read"))
                .is_empty()
        );
        assert_eq!(empty.largest_request, 1);
    }

    #[test]
    fn bounded_reader_rejects_growth_early_eof_and_length_overflow() {
        let mut growth = ScriptedReader::new([ReadStep::Bytes(b"abcd")]);
        assert_eq!(
            failure(read_bounded(&mut growth, 3, || false)),
            "pool_input_changed"
        );
        assert_eq!(growth.calls, 1);
        let mut short = ScriptedReader::new([ReadStep::Bytes(b"ab"), ReadStep::Bytes(b"")]);
        assert_eq!(
            failure(read_bounded(&mut short, 3, || false)),
            "pool_input_changed"
        );
        let mut no_read = ScriptedReader::new([]);
        assert_eq!(
            failure(read_bounded(&mut no_read, usize::MAX, || false)),
            "arithmetic_overflow"
        );
        assert_eq!(no_read.calls, 0);
    }

    #[test]
    fn interrupted_reads_retry_only_while_shutdown_is_absent() {
        let mut reader = ScriptedReader::new([
            ReadStep::Error(io::ErrorKind::Interrupted),
            ReadStep::Bytes(b"a"),
            ReadStep::Bytes(b""),
        ]);
        assert_eq!(
            read_bounded(&mut reader, 1, || false).unwrap_or_else(|_| panic!("retry")),
            b"a"
        );
        let stopped = Cell::new(false);
        let mut reader = ScriptedReader::new([ReadStep::Error(io::ErrorKind::Interrupted)]);
        reader.stop_on_call = Some((1, &stopped));
        assert_eq!(
            failure(read_bounded(&mut reader, 1, || stopped.get())),
            "pool_shutdown"
        );
        assert_eq!(reader.calls, 1);
        let mut reader = ScriptedReader::new([]);
        assert_eq!(
            failure(read_bounded(&mut reader, 1, || true)),
            "pool_shutdown"
        );
        assert_eq!(reader.calls, 0);
    }

    #[test]
    fn real_read_failure_wins_over_a_stop_set_during_that_read() {
        let stopped = Cell::new(false);
        let mut reader = ScriptedReader::new([ReadStep::Error(io::ErrorKind::PermissionDenied)]);
        reader.stop_on_call = Some((1, &stopped));
        assert_eq!(
            failure(read_bounded(&mut reader, 1, || stopped.get())),
            "pool_input_io"
        );
        let stopped = Cell::new(false);
        let mut growth = ScriptedReader::new([ReadStep::Bytes(b"ab")]);
        growth.stop_on_call = Some((1, &stopped));
        assert_eq!(
            failure(read_bounded(&mut growth, 1, || stopped.get())),
            "pool_input_changed"
        );
        let stopped = Cell::new(false);
        let mut eof = ScriptedReader::new([ReadStep::Bytes(b"")]);
        eof.stop_on_call = Some((1, &stopped));
        assert_eq!(
            failure(read_bounded(&mut eof, 0, || stopped.get())),
            "pool_shutdown"
        );
    }

    #[test]
    fn stdin_and_parent_traversal_are_refused_without_leaf_access() {
        for path in ["", "-", "../input", "/tmp/../input", "missing/../input"] {
            assert_eq!(failure(checked_path(Path::new(path))), "pool_input_path");
        }
    }
}
