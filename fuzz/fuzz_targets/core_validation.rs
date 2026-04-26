#![no_main]

use libfuzzer_sys::fuzz_target;
use payrail::{
    CountryCode, CurrencyCode, IdempotencyKey, MerchantReference, Money, PaymentMethod, PhoneNumber,
};

fuzz_target!(|data: &[u8]| {
    if let Ok(value) = std::str::from_utf8(data) {
        let _ = CurrencyCode::new(value);
        let _ = CountryCode::new(value);
        let _ = MerchantReference::new(value);
        let _ = IdempotencyKey::new(value);
        let _ = PhoneNumber::new(value);
        let _ = PaymentMethod::mobile_money_zambia(value);
        let _ = Money::new_minor(1, value);
    }
});
