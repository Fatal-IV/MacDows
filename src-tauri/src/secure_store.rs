//! Encryption of files MacDows keeps on disk, with the Windows Data Protection API (DPAPI): the data can only be
//! read again by the same Windows user on the same PC, and MacDows has no key of its own to keep or leak.

use windows::Win32::Foundation::{LocalFree, HLOCAL};
use windows::Win32::Security::Cryptography::{CryptProtectData, CryptUnprotectData, CRYPT_INTEGER_BLOB};

/// Marks a file as encrypted by this module (a plain JSON file starts with `[` or `{`).
const MAGIC: &[u8; 4] = b"MDP1";

fn blob(data: &[u8]) -> CRYPT_INTEGER_BLOB {
    CRYPT_INTEGER_BLOB { cbData: data.len() as u32, pbData: data.as_ptr() as *mut u8 }
}

fn take(out: CRYPT_INTEGER_BLOB) -> Vec<u8> {
    unsafe {
        let bytes = std::slice::from_raw_parts(out.pbData, out.cbData as usize).to_vec();
        let _ = LocalFree(Some(HLOCAL(out.pbData as *mut _)));
        bytes
    }
}

/// `MAGIC` followed by the encrypted bytes. `None` if Windows could not encrypt.
pub fn seal(plain: &[u8]) -> Option<Vec<u8>> {
    let input = blob(plain);
    let mut out = CRYPT_INTEGER_BLOB::default();
    unsafe { CryptProtectData(&input, None, None, None, None, 0, &mut out).ok()? };
    let mut sealed = MAGIC.to_vec();
    sealed.extend(take(out));
    Some(sealed)
}

/// What `seal` made, decrypted. `None` for anything else, including a file that was never encrypted.
pub fn open(sealed: &[u8]) -> Option<Vec<u8>> {
    let body = sealed.strip_prefix(MAGIC.as_slice())?;
    let input = blob(body);
    let mut out = CRYPT_INTEGER_BLOB::default();
    unsafe { CryptUnprotectData(&input, None, None, None, None, 0, &mut out).ok()? };
    Some(take(out))
}

pub fn is_sealed(bytes: &[u8]) -> bool {
    bytes.starts_with(MAGIC)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn what_is_sealed_can_be_opened_again() {
        let sealed = seal(b"title: secret message").expect("seal");
        assert!(is_sealed(&sealed));
        assert_eq!(open(&sealed).as_deref(), Some(&b"title: secret message"[..]));
    }

    #[test]
    fn the_sealed_bytes_do_not_contain_the_text() {
        let sealed = seal(b"title: secret message").unwrap();
        assert!(!sealed.windows(6).any(|w| w == b"secret"));
    }

    #[test]
    fn plain_files_and_damaged_data_are_not_opened() {
        assert_eq!(open(b"[{\"title\":\"x\"}]"), None);
        let mut sealed = seal(b"abc").unwrap();
        let last = sealed.len() - 1;
        sealed[last] ^= 0xFF;
        assert_eq!(open(&sealed), None);
    }
}
