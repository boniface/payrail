use url::Url;

/// Action the application or payer must take next.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum NextAction {
    /// Redirect the customer to a hosted provider page.
    RedirectToUrl { url: Url },
    /// Ask the customer to complete a Mobile Money prompt.
    MobileMoneyPrompt { message: String },
    /// No action is required.
    None,
}
