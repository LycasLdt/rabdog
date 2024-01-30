use aes::cipher::{block_padding::Pkcs7, BlockDecryptMut};
use anyhow::{anyhow, Result};
use base64::{
    alphabet,
    engine::{general_purpose::STANDARD, DecodePaddingMode, GeneralPurpose, GeneralPurposeConfig},
    Engine,
};

const ENABLE_TRAILING: GeneralPurposeConfig = GeneralPurposeConfig::new()
    .with_decode_padding_mode(DecodePaddingMode::RequireNone)
    .with_decode_allow_trailing_bits(true);
const STANDARD_NO_PAD_TRAILING: GeneralPurpose =
    GeneralPurpose::new(&alphabet::STANDARD, ENABLE_TRAILING);
pub enum Base64Purpose {
    Standard,
    StandardNoPad,
}

#[inline]
pub fn decode_base64<I: AsRef<[u8]>>(input: I, purpose: Base64Purpose) -> Result<Vec<u8>> {
    let purpose = match purpose {
        Base64Purpose::Standard => STANDARD,
        Base64Purpose::StandardNoPad => STANDARD_NO_PAD_TRAILING,
    };

    Ok(purpose.decode(input.as_ref())?)
}

macro_rules! detect_aes_size {
    ($size: ty, [$key: expr, $iv: expr]) => {{
        use aes::cipher::KeyIvInit;
        cbc::Decryptor::<$size>::new_from_slices($key, $iv)
    }};
}
#[inline]
pub fn decode_cbc_aes<K: AsRef<[u8]>, I: AsRef<[u8]>>(
    input: &[u8],
    key: K,
    iv: I,
) -> Result<Vec<u8>> {
    let (key, iv) = (key.as_ref(), iv.as_ref());

    match key.len() {
        16 => Ok(detect_aes_size!(aes::Aes128, [key, iv])?.decrypt_padded_vec_mut::<Pkcs7>(input)?),
        32 => Ok(detect_aes_size!(aes::Aes256, [key, iv])?.decrypt_padded_vec_mut::<Pkcs7>(input)?),
        _ => Err(anyhow!("incorrect length of the aes key")),
    }
}

#[inline]
pub fn decode_hex(input: &str) -> Result<Vec<u8>> {
    (0..input.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&input[i..i + 2], 16).map_err(|e| e.into()))
        .collect()
}
