/// Checkout user interface mode.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum CheckoutUiMode {
    /// Redirect the payer to a provider-hosted checkout page.
    #[default]
    Hosted,
    /// Keep the payer on-site with provider-hosted payment elements.
    Elements,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_checkout_ui_mode_is_hosted() {
        assert_eq!(CheckoutUiMode::default(), CheckoutUiMode::Hosted);
    }
}
