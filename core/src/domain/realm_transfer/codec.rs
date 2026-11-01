use std::collections::BTreeMap;
use std::fmt::Display;

use ferriskey_security::crypto::bundle_envelope::{
    BundleEnvelopeError, BundleKey, BundleOpener, BundleSealer, EnvelopeHeader, HEADER_LEN,
};
use futures::{Stream, StreamExt};
use tokio::sync::mpsc;

use crate::domain::common::entities::app_errors::CoreError;
use crate::domain::realm_transfer::bundle::{
    BundleCheck, BundleHeader, BundleLine, BundlePassphrase, BundleRow, BundleTrailer,
};
use crate::domain::realm_transfer::ports::{RowSink, RowSource};
use crate::domain::realm_transfer::value_objects::BundleChunk;

const LONGEST_LINE: usize = 16 * 1024 * 1024;
const COMPACT_AFTER: usize = 64 * 1024;

fn invalid(message: impl Into<String>) -> CoreError {
    CoreError::InvalidRealmBundle(message.into())
}

fn envelope(error: BundleEnvelopeError) -> CoreError {
    invalid(error.to_string())
}

pub async fn derive_key(
    passphrase: BundlePassphrase,
    header: Option<EnvelopeHeader>,
) -> Result<BundleKey, CoreError> {
    tokio::task::spawn_blocking(move || match header {
        Some(header) => BundleKey::open(passphrase.expose(), header),
        None => BundleKey::create(
            passphrase.expose(),
            ferriskey_security::crypto::bundle_envelope::BundleKdf::EXPORT,
        ),
    })
    .await
    .map_err(|_| CoreError::InternalServerError)?
    .map_err(envelope)
}

pub struct BundleWriter {
    sealer: BundleSealer,
    out: mpsc::Sender<BundleChunk>,
    counts: BTreeMap<String, u64>,
    line: Vec<u8>,
}

impl BundleWriter {
    pub async fn start(
        key: BundleKey,
        header: BundleHeader,
        out: mpsc::Sender<BundleChunk>,
    ) -> Result<Self, CoreError> {
        let mut writer = Self {
            sealer: BundleSealer::new(key),
            out,
            counts: BTreeMap::new(),
            line: Vec::with_capacity(4096),
        };
        writer.write(&BundleLine::Header(header)).await?;
        Ok(writer)
    }

    pub async fn finish(mut self) -> Result<(), CoreError> {
        let trailer = BundleLine::Trailer(BundleTrailer {
            rows: std::mem::take(&mut self.counts),
        });
        self.write(&trailer).await?;

        let Self { sealer, out, .. } = self;
        let last = sealer.finish().map_err(envelope)?;
        send(&out, last).await
    }

    pub async fn fail(self, error: CoreError) {
        let _ = self.out.send(Err(error)).await;
    }

    async fn write(&mut self, line: &BundleLine) -> Result<(), CoreError> {
        self.line.clear();
        serde_json::to_writer(&mut self.line, line)
            .map_err(|error| CoreError::Database(error.to_string()))?;
        self.line.push(b'\n');

        let sealed = self.sealer.push(&self.line).map_err(envelope)?;
        send(&self.out, sealed).await
    }
}

async fn send(out: &mpsc::Sender<BundleChunk>, bytes: Vec<u8>) -> Result<(), CoreError> {
    if bytes.is_empty() {
        return Ok(());
    }
    out.send(Ok(bytes))
        .await
        .map_err(|_| invalid("the export was abandoned by the client"))
}

impl RowSink for BundleWriter {
    async fn accept(&mut self, row: BundleRow) -> Result<(), CoreError> {
        match self.counts.get_mut(&row.table) {
            Some(count) => *count += 1,
            None => {
                self.counts.insert(row.table.clone(), 1);
            }
        }
        self.write(&BundleLine::Row(row)).await
    }
}

struct LineReader<S> {
    body: S,
    opener: BundleOpener,
    buffer: Vec<u8>,
    cursor: usize,
    body_done: bool,
}

impl<S, B, E> LineReader<S>
where
    S: Stream<Item = Result<B, E>> + Send + Unpin,
    B: AsRef<[u8]> + Send,
    E: Display + Send,
{
    async fn next_line(&mut self) -> Result<Option<Vec<u8>>, CoreError> {
        loop {
            let unread = &self.buffer[self.cursor..];
            if let Some(end) = unread.iter().position(|byte| *byte == b'\n') {
                let line = unread[..end].to_vec();
                self.cursor += end + 1;
                if self.cursor > COMPACT_AFTER {
                    self.buffer.drain(..self.cursor);
                    self.cursor = 0;
                }
                return Ok(Some(line));
            }
            if unread.len() > LONGEST_LINE {
                return Err(invalid("a line of the bundle is too long"));
            }
            if self.body_done {
                return if unread.is_empty() {
                    Ok(None)
                } else {
                    Err(invalid("the bundle ends in the middle of a line"))
                };
            }

            match self.body.next().await {
                Some(chunk) => {
                    let chunk = chunk.map_err(|error| invalid(error.to_string()))?;
                    let opened = self.opener.push(chunk.as_ref()).map_err(envelope)?;
                    self.buffer.extend_from_slice(&opened);
                }
                None => {
                    self.opener.finish().map_err(envelope)?;
                    self.body_done = true;
                }
            }
        }
    }
}

pub struct BundleReader<S> {
    lines: LineReader<S>,
    header: BundleHeader,
    check: BundleCheck,
    finished: bool,
}

impl<S, B, E> BundleReader<S>
where
    S: Stream<Item = Result<B, E>> + Send + Unpin,
    B: AsRef<[u8]> + Send,
    E: Display + Send,
{
    pub async fn open(passphrase: BundlePassphrase, mut body: S) -> Result<Self, CoreError> {
        let mut raw = Vec::new();
        let header = loop {
            if let Some(header) = EnvelopeHeader::parse(&raw).map_err(envelope)? {
                break header;
            }
            match body.next().await {
                Some(chunk) => raw
                    .extend_from_slice(chunk.map_err(|error| invalid(error.to_string()))?.as_ref()),
                None => return Err(envelope(BundleEnvelopeError::Truncated)),
            }
        };

        let mut opener = BundleOpener::new(derive_key(passphrase, Some(header)).await?);
        let buffer = opener.push(&raw[HEADER_LEN..]).map_err(envelope)?;
        let mut lines = LineReader {
            body,
            opener,
            buffer,
            cursor: 0,
            body_done: false,
        };

        let first = lines
            .next_line()
            .await?
            .ok_or_else(|| invalid("the bundle is empty"))?;
        let BundleLine::Header(header) = serde_json::from_slice(&first)
            .map_err(|error| invalid(format!("unreadable header: {error}")))?
        else {
            return Err(invalid("the bundle does not start with its header"));
        };

        Ok(Self {
            check: BundleCheck::new(&header),
            lines,
            header,
            finished: false,
        })
    }

    pub fn header(&self) -> &BundleHeader {
        &self.header
    }
}

impl<S, B, E> RowSource for BundleReader<S>
where
    S: Stream<Item = Result<B, E>> + Send + Unpin,
    B: AsRef<[u8]> + Send,
    E: Display + Send,
{
    async fn next_row(&mut self) -> Result<Option<BundleRow>, CoreError> {
        if self.finished {
            return Ok(None);
        }
        let line = self
            .lines
            .next_line()
            .await?
            .ok_or_else(|| invalid("the bundle ends before its trailer"))?;

        match serde_json::from_slice(&line)
            .map_err(|error| invalid(format!("unreadable line: {error}")))?
        {
            BundleLine::Row(row) => {
                self.check.row(&row)?;
                Ok(Some(row))
            }
            BundleLine::Trailer(trailer) => {
                self.check.trailer(&trailer)?;
                if self.lines.next_line().await?.is_some() {
                    return Err(invalid("the bundle goes on after its trailer"));
                }
                self.finished = true;
                Ok(None)
            }
            BundleLine::Header(_) => Err(invalid("the bundle holds a second header")),
        }
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use ferriskey_security::crypto::bundle_envelope::BundleKdf;
    use futures::stream;
    use serde_json::{Value, json};

    use super::*;
    use crate::domain::realm_transfer::bundle::BundleRealm;

    const REALM_ID: &str = "0190c2a4-0000-7000-8000-000000000001";
    const PASSPHRASE: &str = "correct horse battery staple";
    const CHEAP: BundleKdf = BundleKdf {
        memory_kib: 64,
        passes: 1,
        lanes: 1,
    };

    fn passphrase() -> BundlePassphrase {
        BundlePassphrase::try_from(PASSPHRASE.to_string()).unwrap()
    }

    fn header() -> BundleHeader {
        BundleHeader::new(
            BundleRealm {
                id: REALM_ID.parse().unwrap(),
                name: "acme".to_string(),
            },
            1,
            Utc::now(),
        )
    }

    fn row(table: &str, value: Value) -> BundleRow {
        BundleRow {
            table: table.to_string(),
            row: value.as_object().cloned().unwrap(),
        }
    }

    fn realm_rows(users: usize) -> Vec<BundleRow> {
        let mut rows = vec![
            row("realms", json!({ "id": REALM_ID, "name": "acme" })),
            row(
                "jwt_keys",
                json!({ "id": "k", "realm_id": REALM_ID, "private_key": "-----BEGIN PRIVATE KEY-----", "rotated_at": null, "revoked_at": null }),
            ),
        ];
        rows.extend((0..users).map(|n| {
            row(
                "users",
                json!({ "id": format!("user-{n}"), "realm_id": REALM_ID, "username": format!("alice-{n}") }),
            )
        }));
        rows
    }

    async fn write(rows: Vec<BundleRow>) -> Vec<u8> {
        let (out, mut chunks) = mpsc::channel(1024);
        let key = BundleKey::create(PASSPHRASE, CHEAP).unwrap();
        let mut writer = BundleWriter::start(key, header(), out).await.unwrap();
        for row in rows {
            writer.accept(row).await.unwrap();
        }
        writer.finish().await.unwrap();

        let mut bundle = Vec::new();
        while let Some(chunk) = chunks.recv().await {
            bundle.extend(chunk.unwrap());
        }
        bundle
    }

    async fn read(bundle: Vec<u8>, piece: usize) -> Result<Vec<BundleRow>, CoreError> {
        let pieces: Vec<Result<Vec<u8>, String>> =
            bundle.chunks(piece).map(|p| Ok(p.to_vec())).collect();
        let mut reader = BundleReader::open(passphrase(), stream::iter(pieces)).await?;
        assert_eq!(reader.header().realm.name, "acme");

        let mut rows = Vec::new();
        while let Some(row) = reader.next_row().await? {
            rows.push(row);
        }
        Ok(rows)
    }

    #[tokio::test]
    async fn a_bundle_reads_back_the_rows_it_was_written_with() {
        let rows = realm_rows(3000);
        let bundle = write(rows.clone()).await;

        for piece in [1 << 20, 4093] {
            assert_eq!(read(bundle.clone(), piece).await.unwrap(), rows);
        }
    }

    #[tokio::test]
    async fn the_written_bundle_shows_no_private_material() {
        let bundle = write(realm_rows(5)).await;

        assert!(!bundle.windows(11).any(|w| w == b"PRIVATE KEY"));
        assert!(!bundle.windows(5).any(|w| w == b"alice"));
    }

    #[tokio::test]
    async fn a_bundle_cut_short_is_refused() {
        let bundle = write(realm_rows(3000)).await;

        for keep in [bundle.len() / 2, bundle.len() - 1] {
            assert!(
                read(bundle[..keep].to_vec(), 4096).await.is_err(),
                "kept {keep}"
            );
        }
    }

    #[tokio::test]
    async fn a_wrong_passphrase_is_refused_before_any_row() {
        let bundle = write(realm_rows(1)).await;
        let other = BundlePassphrase::try_from("another passphrase entirely".to_string()).unwrap();
        let body = stream::iter(vec![Ok::<_, String>(bundle)]);

        assert!(matches!(
            BundleReader::open(other, body).await,
            Err(CoreError::InvalidRealmBundle(_))
        ));
    }

    #[tokio::test]
    async fn a_bundle_without_keys_is_refused() {
        let mut rows = realm_rows(2);
        rows.remove(1);
        let bundle = write(rows).await;

        assert!(matches!(
            read(bundle, 1 << 20).await,
            Err(CoreError::RealmBundleWithoutKeys)
        ));
    }

    #[tokio::test]
    async fn a_body_error_stops_the_read() {
        let bundle = write(realm_rows(1)).await;
        let body = stream::iter(vec![
            Ok(bundle[..HEADER_LEN + 10].to_vec()),
            Err("connection reset".to_string()),
        ]);

        assert!(BundleReader::open(passphrase(), body).await.is_err());
    }
}
