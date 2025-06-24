// Must only be constructed once, which is what we guarantee with the "unsafe impl" below:
#[derive(Debug)]
pub struct OtCryptoLibHMACID;

unsafe impl omniglot::id::OGID for OtCryptoLibHMACID {
    type Imprint = OtCryptoLibHMACIDImprint;

    fn get_imprint(&self) -> Self::Imprint {
        OtCryptoLibHMACIDImprint
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, PartialOrd)]
pub struct OtCryptoLibHMACIDImprint;

unsafe impl omniglot::id::OGIDImprint for OtCryptoLibHMACIDImprint {
    fn numeric_id(&self) -> u64 {
        0
    }
}
