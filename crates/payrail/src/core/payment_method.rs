use crate::{CountryCode, PaymentError, PhoneNumber};

/// Provider-neutral payment method.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum PaymentMethod {
    /// Card payment.
    Card(CardPaymentMethod),
    /// Stablecoin payment.
    Stablecoin(StablecoinPaymentMethod),
    /// Crypto payment.
    Crypto(CryptoPaymentMethod),
    /// `PayPal` order.
    PayPal(PayPalPaymentMethod),
    /// Mobile Money collection.
    MobileMoney(MobileMoneyPaymentMethod),
}

impl PaymentMethod {
    /// Creates a card payment method.
    #[inline]
    #[must_use]
    pub const fn card() -> Self {
        Self::Card(CardPaymentMethod)
    }

    /// Creates a `PayPal` payment method.
    #[inline]
    #[must_use]
    pub const fn paypal() -> Self {
        Self::PayPal(PayPalPaymentMethod)
    }

    /// Creates a stablecoin payment method for an asset.
    #[inline]
    #[must_use]
    pub const fn stablecoin(asset: StablecoinAsset) -> Self {
        Self::Stablecoin(StablecoinPaymentMethod {
            preferred_asset: Some(asset),
        })
    }

    /// Creates a USDC stablecoin method.
    #[inline]
    #[must_use]
    pub const fn stablecoin_usdc() -> Self {
        Self::stablecoin(StablecoinAsset::Usdc)
    }

    /// Creates a USDT stablecoin method.
    #[inline]
    #[must_use]
    pub const fn stablecoin_usdt() -> Self {
        Self::stablecoin(StablecoinAsset::Usdt)
    }

    /// Creates a crypto payment method for an asset.
    #[inline]
    #[must_use]
    pub const fn crypto(asset: CryptoAsset) -> Self {
        Self::Crypto(CryptoPaymentMethod {
            asset,
            network: None,
        })
    }

    /// Creates a crypto payment method for an asset on a specific network.
    #[inline]
    #[must_use]
    pub const fn crypto_on(asset: CryptoAsset, network: CryptoNetwork) -> Self {
        Self::Crypto(CryptoPaymentMethod {
            asset,
            network: Some(network),
        })
    }

    /// Creates a USDC crypto payment method on a specific network.
    #[inline]
    #[must_use]
    pub const fn usdc_on(network: CryptoNetwork) -> Self {
        Self::crypto_on(CryptoAsset::Usdc, network)
    }

    /// Creates a USDT crypto payment method on a specific network.
    #[inline]
    #[must_use]
    pub const fn usdt_on(network: CryptoNetwork) -> Self {
        Self::crypto_on(CryptoAsset::Usdt, network)
    }

    /// Creates a Zambia Mobile Money payment method.
    ///
    /// # Errors
    ///
    /// Returns an error when the phone number or country code is invalid.
    pub fn mobile_money_zambia(phone_number: impl AsRef<str>) -> Result<Self, PaymentError> {
        Ok(Self::MobileMoney(MobileMoneyPaymentMethod {
            country: CountryCode::new("ZM")?,
            phone_number: PhoneNumber::new(phone_number)?,
            operator: None,
        }))
    }
}

/// Card payment marker.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CardPaymentMethod;

/// `PayPal` payment marker.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PayPalPaymentMethod;

/// Stablecoin payment configuration.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct StablecoinPaymentMethod {
    /// Preferred stablecoin asset.
    pub preferred_asset: Option<StablecoinAsset>,
}

/// Stablecoin asset.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum StablecoinAsset {
    /// USDC.
    Usdc,
    /// USDT.
    Usdt,
    /// USDP.
    Usdp,
    /// USDG.
    Usdg,
    /// Other stablecoin asset.
    Other(String),
}

/// Crypto payment configuration.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CryptoPaymentMethod {
    /// Requested crypto asset.
    pub asset: CryptoAsset,
    /// Optional requested network.
    pub network: Option<CryptoNetwork>,
}

/// Crypto asset.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum CryptoAsset {
    /// Bitcoin.
    Btc,
    /// Ether.
    Eth,
    /// Solana.
    Sol,
    /// USDC.
    Usdc,
    /// USDT.
    Usdt,
    /// USDP.
    Usdp,
    /// USDG.
    Usdg,
    /// Other crypto asset.
    Other(String),
}

/// Crypto settlement network.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum CryptoNetwork {
    /// Bitcoin network.
    Bitcoin,
    /// Ethereum mainnet.
    Ethereum,
    /// Solana.
    Solana,
    /// Polygon.
    Polygon,
    /// Base.
    Base,
    /// BNB Smart Chain.
    Bsc,
    /// Arbitrum.
    Arbitrum,
    /// Optimism.
    Optimism,
    /// Other crypto network.
    Other(String),
}

impl From<&StablecoinAsset> for CryptoAsset {
    fn from(asset: &StablecoinAsset) -> Self {
        match asset {
            StablecoinAsset::Usdc => Self::Usdc,
            StablecoinAsset::Usdt => Self::Usdt,
            StablecoinAsset::Usdp => Self::Usdp,
            StablecoinAsset::Usdg => Self::Usdg,
            StablecoinAsset::Other(asset) => Self::Other(asset.clone()),
        }
    }
}

/// Mobile Money payment configuration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MobileMoneyPaymentMethod {
    /// Country for the Mobile Money account.
    pub country: CountryCode,
    /// Customer phone number.
    pub phone_number: PhoneNumber,
    /// Optional operator hint.
    pub operator: Option<MobileMoneyOperator>,
}

/// Mobile Money operator.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum MobileMoneyOperator {
    /// MTN.
    Mtn,
    /// Airtel.
    Airtel,
    /// Zamtel.
    Zamtel,
    /// M-Pesa.
    Mpesa,
    /// Orange Money.
    Orange,
    /// Other operator.
    Other(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mobile_money_zambia_sets_country() {
        let method =
            PaymentMethod::mobile_money_zambia("260971234567").expect("method should be valid");

        match method {
            PaymentMethod::MobileMoney(method) => {
                assert_eq!(method.country.as_str(), "ZM");
                assert_eq!(method.phone_number.digits(), "260971234567");
            }
            PaymentMethod::Card(_)
            | PaymentMethod::Stablecoin(_)
            | PaymentMethod::Crypto(_)
            | PaymentMethod::PayPal(_) => panic!("expected mobile money"),
        }
    }

    #[test]
    fn helper_constructors_create_expected_variants() {
        assert!(matches!(PaymentMethod::card(), PaymentMethod::Card(_)));
        assert!(matches!(PaymentMethod::paypal(), PaymentMethod::PayPal(_)));
        assert!(matches!(
            PaymentMethod::stablecoin_usdc(),
            PaymentMethod::Stablecoin(StablecoinPaymentMethod {
                preferred_asset: Some(StablecoinAsset::Usdc)
            })
        ));
        assert!(matches!(
            PaymentMethod::stablecoin_usdt(),
            PaymentMethod::Stablecoin(StablecoinPaymentMethod {
                preferred_asset: Some(StablecoinAsset::Usdt)
            })
        ));
        assert!(matches!(
            PaymentMethod::usdc_on(CryptoNetwork::Base),
            PaymentMethod::Crypto(CryptoPaymentMethod {
                asset: CryptoAsset::Usdc,
                network: Some(CryptoNetwork::Base)
            })
        ));
        assert!(matches!(
            PaymentMethod::usdt_on(CryptoNetwork::Solana),
            PaymentMethod::Crypto(CryptoPaymentMethod {
                asset: CryptoAsset::Usdt,
                network: Some(CryptoNetwork::Solana)
            })
        ));
    }

    #[test]
    fn stablecoin_assets_convert_to_crypto_assets() {
        assert_eq!(CryptoAsset::from(&StablecoinAsset::Usdc), CryptoAsset::Usdc);
        assert_eq!(CryptoAsset::from(&StablecoinAsset::Usdt), CryptoAsset::Usdt);
        assert_eq!(
            CryptoAsset::from(&StablecoinAsset::Other("eurc".to_owned())),
            CryptoAsset::Other("eurc".to_owned())
        );
    }
}
