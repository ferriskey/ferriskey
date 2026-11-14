use argon2::{Algorithm, Argon2, Params, Version};
use aws_lc_rs::aead::{AES_256_GCM, Aad, LessSafeKey, NONCE_LEN, Nonce, UnboundKey};
use aws_lc_rs::rand::fill;
use thiserror::Error;

pub const MAGIC: &[u8; 4] = b"FKRB";
pub const HEADER_LEN: usize = 41;
pub const FRAME_PLAINTEXT_LEN: usize = 64 * 1024;

const ENVELOPE_VERSION: u8 = 1;
const KDF_ARGON2ID: u8 = 1;
const SALT_LEN: usize = 16;
const PREFIX_LEN: usize = 7;
const KEY_LEN: usize = 32;
const TAG_LEN: usize = 16;
const LENGTH_LEN: usize = 4;
const MAX_FRAME_LEN: usize = FRAME_PLAINTEXT_LEN + TAG_LEN;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum BundleEnvelopeError {
    #[error("the file is not a realm bundle")]
    NotABundle,

    #[error("the bundle envelope version {0} is not supported")]
    UnsupportedVersion(u8),

    #[error("the bundle key derivation is not supported or too costly")]
    UnsupportedKdf,

    #[error("the bundle does not open: wrong passphrase or tampered file")]
    OpenFailed,

    #[error("the bundle is cut short")]
    Truncated,

    #[error("the bundle has bytes after its last frame")]
    TrailingBytes,

    #[error("the bundle frame is malformed")]
    MalformedFrame,

    #[error("the bundle could not be sealed")]
    SealFailed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BundleKdf {
    pub memory_kib: u32,
    pub passes: u32,
    pub lanes: u32,
}

impl BundleKdf {
    pub const EXPORT: Self = Self {
        memory_kib: 64 * 1024,
        passes: 3,
        lanes: 1,
    };

    pub const IMPORT_CEILING: Self = Self {
        memory_kib: 256 * 1024,
        passes: 10,
        lanes: 8,
    };

    fn within(&self, ceiling: &Self) -> bool {
        self.memory_kib <= ceiling.memory_kib
            && self.passes <= ceiling.passes
            && self.lanes <= ceiling.lanes
    }

    fn derive(&self, passphrase: &str, salt: &[u8]) -> Result<LessSafeKey, BundleEnvelopeError> {
        let params = Params::new(self.memory_kib, self.passes, self.lanes, Some(KEY_LEN))
            .map_err(|_| BundleEnvelopeError::UnsupportedKdf)?;
        let mut key = [0u8; KEY_LEN];
        Argon2::new(Algorithm::Argon2id, Version::V0x13, params)
            .hash_password_into(passphrase.as_bytes(), salt, &mut key)
            .map_err(|_| BundleEnvelopeError::UnsupportedKdf)?;

        let unbound =
            UnboundKey::new(&AES_256_GCM, &key).map_err(|_| BundleEnvelopeError::SealFailed)?;
        key.fill(0);

        Ok(LessSafeKey::new(unbound))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnvelopeHeader {
    bytes: [u8; HEADER_LEN],
}

impl EnvelopeHeader {
    fn new(kdf: BundleKdf) -> Result<Self, BundleEnvelopeError> {
        let mut bytes = [0u8; HEADER_LEN];
        bytes[0..4].copy_from_slice(MAGIC);
        bytes[4] = ENVELOPE_VERSION;
        bytes[5] = KDF_ARGON2ID;
        bytes[6..10].copy_from_slice(&kdf.memory_kib.to_be_bytes());
        bytes[10..14].copy_from_slice(&kdf.passes.to_be_bytes());
        bytes[14..18].copy_from_slice(&kdf.lanes.to_be_bytes());
        fill(&mut bytes[18..HEADER_LEN]).map_err(|_| BundleEnvelopeError::SealFailed)?;

        Ok(Self { bytes })
    }

    pub fn parse(bytes: &[u8]) -> Result<Option<Self>, BundleEnvelopeError> {
        let available = bytes.len().min(MAGIC.len());
        if bytes[..available] != MAGIC[..available] {
            return Err(BundleEnvelopeError::NotABundle);
        }
        let Some(header) = bytes.get(..HEADER_LEN) else {
            return Ok(None);
        };
        if header[4] != ENVELOPE_VERSION {
            return Err(BundleEnvelopeError::UnsupportedVersion(header[4]));
        }
        if header[5] != KDF_ARGON2ID {
            return Err(BundleEnvelopeError::UnsupportedKdf);
        }

        let mut fixed = [0u8; HEADER_LEN];
        fixed.copy_from_slice(header);
        Ok(Some(Self { bytes: fixed }))
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    pub fn kdf(&self) -> BundleKdf {
        let word = |at: usize| {
            u32::from_be_bytes([
                self.bytes[at],
                self.bytes[at + 1],
                self.bytes[at + 2],
                self.bytes[at + 3],
            ])
        };
        BundleKdf {
            memory_kib: word(6),
            passes: word(10),
            lanes: word(14),
        }
    }

    fn salt(&self) -> &[u8] {
        &self.bytes[18..18 + SALT_LEN]
    }

    fn prefix(&self) -> &[u8] {
        &self.bytes[18 + SALT_LEN..HEADER_LEN]
    }

    fn nonce(&self, counter: u32, last: bool) -> Nonce {
        let mut nonce = [0u8; NONCE_LEN];
        nonce[..PREFIX_LEN].copy_from_slice(self.prefix());
        nonce[PREFIX_LEN..PREFIX_LEN + 4].copy_from_slice(&counter.to_be_bytes());
        nonce[NONCE_LEN - 1] = u8::from(last);
        Nonce::assume_unique_for_key(nonce)
    }
}

pub struct BundleKey {
    header: EnvelopeHeader,
    key: LessSafeKey,
}

impl BundleKey {
    pub fn create(passphrase: &str, kdf: BundleKdf) -> Result<Self, BundleEnvelopeError> {
        let header = EnvelopeHeader::new(kdf)?;
        let key = kdf.derive(passphrase, header.salt())?;
        Ok(Self { header, key })
    }

    pub fn open(passphrase: &str, header: EnvelopeHeader) -> Result<Self, BundleEnvelopeError> {
        let kdf = header.kdf();
        if !kdf.within(&BundleKdf::IMPORT_CEILING) {
            return Err(BundleEnvelopeError::UnsupportedKdf);
        }
        let key = kdf.derive(passphrase, header.salt())?;
        Ok(Self { header, key })
    }
}

pub struct BundleSealer {
    key: BundleKey,
    counter: u32,
    pending: Vec<u8>,
    header_sent: bool,
}

impl BundleSealer {
    pub fn new(key: BundleKey) -> Self {
        Self {
            key,
            counter: 0,
            pending: Vec::with_capacity(FRAME_PLAINTEXT_LEN),
            header_sent: false,
        }
    }

    pub fn push(&mut self, mut plaintext: &[u8]) -> Result<Vec<u8>, BundleEnvelopeError> {
        let mut out = self.take_header();

        while !plaintext.is_empty() {
            let room = FRAME_PLAINTEXT_LEN - self.pending.len();
            let (now, later) = plaintext.split_at(room.min(plaintext.len()));
            self.pending.extend_from_slice(now);
            plaintext = later;

            if self.pending.len() == FRAME_PLAINTEXT_LEN && !plaintext.is_empty() {
                self.seal_pending(false, &mut out)?;
            }
        }

        Ok(out)
    }

    pub fn finish(mut self) -> Result<Vec<u8>, BundleEnvelopeError> {
        let mut out = self.take_header();
        self.seal_pending(true, &mut out)?;
        Ok(out)
    }

    fn take_header(&mut self) -> Vec<u8> {
        if self.header_sent {
            return Vec::new();
        }
        self.header_sent = true;
        self.key.header.as_bytes().to_vec()
    }

    fn seal_pending(&mut self, last: bool, out: &mut Vec<u8>) -> Result<(), BundleEnvelopeError> {
        let nonce = self.key.header.nonce(self.counter, last);
        self.counter = self
            .counter
            .checked_add(1)
            .ok_or(BundleEnvelopeError::SealFailed)?;

        let mut frame = std::mem::take(&mut self.pending);
        self.key
            .key
            .seal_in_place_append_tag(nonce, Aad::from(self.key.header.as_bytes()), &mut frame)
            .map_err(|_| BundleEnvelopeError::SealFailed)?;

        let length = u32::try_from(frame.len()).map_err(|_| BundleEnvelopeError::SealFailed)?;
        out.reserve(LENGTH_LEN + frame.len());
        out.extend_from_slice(&length.to_be_bytes());
        out.extend_from_slice(&frame);
        self.pending = Vec::with_capacity(FRAME_PLAINTEXT_LEN);
        Ok(())
    }
}

pub struct BundleOpener {
    key: BundleKey,
    counter: u32,
    buffer: Vec<u8>,
    finished: bool,
}

impl BundleOpener {
    pub fn new(key: BundleKey) -> Self {
        Self {
            key,
            counter: 0,
            buffer: Vec::new(),
            finished: false,
        }
    }

    pub fn push(&mut self, bytes: &[u8]) -> Result<Vec<u8>, BundleEnvelopeError> {
        if self.finished {
            return if bytes.is_empty() {
                Ok(Vec::new())
            } else {
                Err(BundleEnvelopeError::TrailingBytes)
            };
        }
        self.buffer.extend_from_slice(bytes);

        let mut plaintext = Vec::new();
        let mut consumed = 0;
        while let Some(frame) = self.next_frame(consumed)? {
            consumed += LENGTH_LEN + frame;
            let start = consumed - frame;
            let last = self.open_frame(start, frame, &mut plaintext)?;
            if last {
                self.finished = true;
                if consumed != self.buffer.len() {
                    return Err(BundleEnvelopeError::TrailingBytes);
                }
                break;
            }
        }
        self.buffer.drain(..consumed);

        Ok(plaintext)
    }

    pub fn finish(&self) -> Result<(), BundleEnvelopeError> {
        if self.finished {
            Ok(())
        } else {
            Err(BundleEnvelopeError::Truncated)
        }
    }

    fn next_frame(&self, from: usize) -> Result<Option<usize>, BundleEnvelopeError> {
        let Some(length) = self.buffer.get(from..from + LENGTH_LEN) else {
            return Ok(None);
        };
        let length = u32::from_be_bytes([length[0], length[1], length[2], length[3]]) as usize;
        if !(TAG_LEN..=MAX_FRAME_LEN).contains(&length) {
            return Err(BundleEnvelopeError::MalformedFrame);
        }
        if self.buffer.len() < from + LENGTH_LEN + length {
            return Ok(None);
        }
        Ok(Some(length))
    }

    fn open_frame(
        &mut self,
        start: usize,
        length: usize,
        plaintext: &mut Vec<u8>,
    ) -> Result<bool, BundleEnvelopeError> {
        let sealed = self.buffer[start..start + length].to_vec();

        for last in [false, true] {
            let mut frame = sealed.clone();
            let nonce = self.key.header.nonce(self.counter, last);
            if let Ok(opened) =
                self.key
                    .key
                    .open_in_place(nonce, Aad::from(self.key.header.as_bytes()), &mut frame)
            {
                plaintext.extend_from_slice(opened);
                self.counter = self
                    .counter
                    .checked_add(1)
                    .ok_or(BundleEnvelopeError::MalformedFrame)?;
                return Ok(last);
            }
        }

        Err(BundleEnvelopeError::OpenFailed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PASSPHRASE: &str = "correct horse battery staple";
    const CHEAP: BundleKdf = BundleKdf {
        memory_kib: 64,
        passes: 1,
        lanes: 1,
    };

    fn seal(plaintext: &[u8], chunk: usize) -> Vec<u8> {
        let mut sealer = BundleSealer::new(BundleKey::create(PASSPHRASE, CHEAP).unwrap());
        let mut out = Vec::new();
        for piece in plaintext.chunks(chunk.max(1)) {
            out.extend(sealer.push(piece).unwrap());
        }
        out.extend(sealer.finish().unwrap());
        out
    }

    fn open(sealed: &[u8], passphrase: &str, chunk: usize) -> Result<Vec<u8>, BundleEnvelopeError> {
        let header = EnvelopeHeader::parse(sealed)?.ok_or(BundleEnvelopeError::Truncated)?;
        let mut opener = BundleOpener::new(BundleKey::open(passphrase, header)?);
        let mut out = Vec::new();
        for piece in sealed[HEADER_LEN..].chunks(chunk.max(1)) {
            out.extend(opener.push(piece)?);
        }
        opener.finish()?;
        Ok(out)
    }

    fn sample(len: usize) -> Vec<u8> {
        (0..len).map(|i| (i % 251) as u8).collect()
    }

    #[test]
    fn what_is_sealed_opens_whatever_the_chunking() {
        for len in [
            0,
            1,
            FRAME_PLAINTEXT_LEN - 1,
            FRAME_PLAINTEXT_LEN,
            FRAME_PLAINTEXT_LEN + 1,
            3 * FRAME_PLAINTEXT_LEN + 17,
        ] {
            let plaintext = sample(len);
            for (seal_chunk, open_chunk) in [(1 << 20, 1 << 20), (7, 13), (4096, 1)] {
                if len > 100_000 && (seal_chunk < 64 || open_chunk < 64) {
                    continue;
                }
                let sealed = seal(&plaintext, seal_chunk);
                assert_eq!(
                    open(&sealed, PASSPHRASE, open_chunk).unwrap(),
                    plaintext,
                    "len {len}, seal by {seal_chunk}, open by {open_chunk}"
                );
            }
        }
    }

    #[test]
    fn the_sealed_file_does_not_show_its_content() {
        let secret = b"-----BEGIN PRIVATE KEY----- $argon2id$v=19$m=19456";
        let sealed = seal(secret, 1 << 20);

        assert!(sealed.starts_with(MAGIC));
        assert!(!sealed.windows(11).any(|w| w == b"PRIVATE KEY"));
        assert!(!sealed.windows(7).any(|w| w == b"$argon2"));
    }

    #[test]
    fn a_wrong_passphrase_is_refused() {
        let sealed = seal(b"payload", 64);

        assert_eq!(
            open(&sealed, "another passphrase entirely", 64),
            Err(BundleEnvelopeError::OpenFailed)
        );
    }

    #[test]
    fn a_tampered_byte_anywhere_is_refused() {
        let sealed = seal(&sample(FRAME_PLAINTEXT_LEN + 10), 1 << 20);

        for at in [6, 20, 40, HEADER_LEN + 2, HEADER_LEN + 9, sealed.len() - 1] {
            let mut tampered = sealed.clone();
            tampered[at] ^= 0x01;
            assert!(open(&tampered, PASSPHRASE, 1 << 20).is_err(), "byte {at}");
        }
    }

    #[test]
    fn a_bundle_cut_at_a_frame_boundary_is_refused() {
        let sealed = seal(&sample(2 * FRAME_PLAINTEXT_LEN + 10), 1 << 20);
        let first_frame_end = HEADER_LEN + LENGTH_LEN + FRAME_PLAINTEXT_LEN + TAG_LEN;

        assert_eq!(
            open(&sealed[..first_frame_end], PASSPHRASE, 1 << 20),
            Err(BundleEnvelopeError::Truncated)
        );
        assert_eq!(
            open(&sealed[..sealed.len() - 3], PASSPHRASE, 1 << 20),
            Err(BundleEnvelopeError::Truncated)
        );
    }

    #[test]
    fn frames_cannot_be_reordered_or_dropped() {
        let sealed = seal(&sample(3 * FRAME_PLAINTEXT_LEN), 1 << 20);
        let frame = LENGTH_LEN + FRAME_PLAINTEXT_LEN + TAG_LEN;
        let body = &sealed[HEADER_LEN..];

        let mut swapped = sealed[..HEADER_LEN].to_vec();
        swapped.extend_from_slice(&body[frame..2 * frame]);
        swapped.extend_from_slice(&body[..frame]);
        swapped.extend_from_slice(&body[2 * frame..]);
        assert_eq!(
            open(&swapped, PASSPHRASE, 1 << 20),
            Err(BundleEnvelopeError::OpenFailed)
        );

        let mut dropped = sealed[..HEADER_LEN].to_vec();
        dropped.extend_from_slice(&body[frame..]);
        assert_eq!(
            open(&dropped, PASSPHRASE, 1 << 20),
            Err(BundleEnvelopeError::OpenFailed)
        );
    }

    #[test]
    fn bytes_after_the_last_frame_are_refused() {
        let mut sealed = seal(b"payload", 64);
        sealed.extend_from_slice(b"extra");

        assert_eq!(
            open(&sealed, PASSPHRASE, 1 << 20),
            Err(BundleEnvelopeError::TrailingBytes)
        );
    }

    #[test]
    fn another_file_is_not_taken_for_a_bundle() {
        assert_eq!(
            EnvelopeHeader::parse(b"{\"kind\":\"header\"}"),
            Err(BundleEnvelopeError::NotABundle)
        );
        assert_eq!(EnvelopeHeader::parse(b"FK"), Ok(None));
    }

    #[test]
    fn a_header_asking_for_a_costly_derivation_is_refused() {
        let mut sealed = seal(b"payload", 64);
        sealed[6..10].copy_from_slice(&(1024 * 1024u32).to_be_bytes());
        let header = EnvelopeHeader::parse(&sealed).unwrap().unwrap();

        assert!(matches!(
            BundleKey::open(PASSPHRASE, header),
            Err(BundleEnvelopeError::UnsupportedKdf)
        ));
    }

    #[test]
    fn the_export_cost_is_accepted_at_import() {
        assert!(BundleKdf::EXPORT.within(&BundleKdf::IMPORT_CEILING));
    }
}
