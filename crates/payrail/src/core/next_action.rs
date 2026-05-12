use url::Url;

/// Action the application or payer must take next.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum NextAction {
    /// Redirect the customer to a hosted provider page.
    RedirectToUrl { url: Url },
    /// Initialize an embedded checkout component with a provider client secret.
    EmbeddedCheckout { client_secret: String },
    /// Ask the customer to complete a Mobile Money prompt.
    MobileMoneyPrompt { message: String },
    /// No action is required.
    None,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_checkout_action_carries_client_secret() {
        let action = NextAction::EmbeddedCheckout {
            client_secret: "cs_test_secret_payrail".to_owned(),
        };

        assert_eq!(
            action,
            NextAction::EmbeddedCheckout {
                client_secret: "cs_test_secret_payrail".to_owned()
            }
        );
    }
}
