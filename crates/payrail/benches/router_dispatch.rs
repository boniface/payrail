use criterion::{Criterion, criterion_group, criterion_main};
use payrail::{
    BuiltinProvider, CountryCode, CryptoAsset, CryptoNetwork, MobileMoneyPaymentMethod,
    PaymentMethod, PaymentRouter, PhoneNumber,
};

fn routed_router() -> PaymentRouter {
    let mut router = PaymentRouter::new();
    router.route_mobile_money(
        CountryCode::new("KE").expect("country should be valid"),
        BuiltinProvider::Lipila,
    );
    router.route_crypto(BuiltinProvider::Coinbase);
    router.route_crypto_asset(CryptoAsset::Usdt, BuiltinProvider::Binance);
    router.route_crypto_asset_network(
        CryptoAsset::Usdc,
        CryptoNetwork::Base,
        BuiltinProvider::Bridge,
    );
    router
}

fn router_dispatch(criterion: &mut Criterion) {
    let default_router = PaymentRouter::new();
    let routed_router = routed_router();
    let card = PaymentMethod::card();
    let zambia_mobile_money =
        PaymentMethod::mobile_money_zambia("260971234567").expect("phone should be valid");
    let kenya_mobile_money = PaymentMethod::MobileMoney(MobileMoneyPaymentMethod {
        country: CountryCode::new("KE").expect("country should be valid"),
        phone_number: PhoneNumber::new("254712345678").expect("phone should be valid"),
        operator: None,
    });
    let base_usdc = PaymentMethod::usdc_on(CryptoNetwork::Base);
    let usdt = PaymentMethod::stablecoin_usdt();
    let btc = PaymentMethod::crypto(CryptoAsset::Btc);

    criterion.bench_function("resolve_card_default_stripe", |bencher| {
        bencher.iter(|| {
            assert_eq!(
                default_router
                    .resolve_provider(&card)
                    .expect("card should route"),
                BuiltinProvider::Stripe
            );
        });
    });

    criterion.bench_function("resolve_mobile_money_default_lipila", |bencher| {
        bencher.iter(|| {
            assert_eq!(
                default_router
                    .resolve_provider(&zambia_mobile_money)
                    .expect("mobile money should route"),
                BuiltinProvider::Lipila
            );
        });
    });

    criterion.bench_function("resolve_mobile_money_country_route", |bencher| {
        bencher.iter(|| {
            assert_eq!(
                routed_router
                    .resolve_provider(&kenya_mobile_money)
                    .expect("mobile money should route"),
                BuiltinProvider::Lipila
            );
        });
    });

    criterion.bench_function("resolve_crypto_exact_asset_network_route", |bencher| {
        bencher.iter(|| {
            assert_eq!(
                routed_router
                    .resolve_provider(&base_usdc)
                    .expect("crypto should route"),
                BuiltinProvider::Bridge
            );
        });
    });

    criterion.bench_function("resolve_stablecoin_asset_route", |bencher| {
        bencher.iter(|| {
            assert_eq!(
                routed_router
                    .resolve_provider(&usdt)
                    .expect("stablecoin should route"),
                BuiltinProvider::Binance
            );
        });
    });

    criterion.bench_function("resolve_crypto_default_route", |bencher| {
        bencher.iter(|| {
            assert_eq!(
                routed_router
                    .resolve_provider(&btc)
                    .expect("crypto should route"),
                BuiltinProvider::Coinbase
            );
        });
    });
}

criterion_group!(benches, router_dispatch);
criterion_main!(benches);
