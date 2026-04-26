# payrail-lipila

Lipila connector for PayRail Zambia Mobile Money.

Supports Mobile Money collection creation, status checks, callback parsing, HMAC webhook verification, and normalized event mapping.

## Security

Webhook verification requires the raw request body and rejects stale signed timestamps.
