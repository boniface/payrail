use payrail_core::{PaymentEventType, PaymentStatus};

pub(crate) fn map_payment_status(
    status: Option<&str>,
    payment_status: Option<&str>,
) -> PaymentStatus {
    match payment_status.or(status) {
        Some("paid" | "succeeded" | "complete") => PaymentStatus::Succeeded,
        Some("unpaid" | "open" | "requires_payment_method") => PaymentStatus::RequiresAction,
        Some("processing") => PaymentStatus::Processing,
        Some("requires_capture") => PaymentStatus::Authorized,
        Some("canceled" | "cancelled") => PaymentStatus::Cancelled,
        Some("expired") => PaymentStatus::Expired,
        Some("refunded") => PaymentStatus::Refunded,
        Some("partially_refunded") => PaymentStatus::PartiallyRefunded,
        Some(_) | None => PaymentStatus::Processing,
    }
}

pub(crate) fn map_refund_status(status: Option<&str>) -> PaymentStatus {
    match status {
        Some("succeeded") => PaymentStatus::Refunded,
        Some("failed" | "canceled") => PaymentStatus::Failed,
        Some("pending") | Some("requires_action") | Some(_) | None => PaymentStatus::Processing,
    }
}

pub(crate) fn map_event_type(event_type: &str) -> (PaymentEventType, PaymentStatus) {
    match event_type {
        "checkout.session.completed" | "payment_intent.succeeded" => {
            (PaymentEventType::PaymentSucceeded, PaymentStatus::Succeeded)
        }
        "payment_intent.payment_failed" => (PaymentEventType::PaymentFailed, PaymentStatus::Failed),
        "payment_intent.canceled" => (PaymentEventType::PaymentCancelled, PaymentStatus::Cancelled),
        "charge.refunded" => (PaymentEventType::PaymentRefunded, PaymentStatus::Refunded),
        "refund.created" | "refund.updated" => {
            (PaymentEventType::RefundCreated, PaymentStatus::Processing)
        }
        _ => (PaymentEventType::PaymentPending, PaymentStatus::Processing),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_succeeded_status() {
        assert_eq!(
            map_payment_status(None, Some("paid")),
            PaymentStatus::Succeeded
        );
    }

    #[test]
    fn maps_failed_event() {
        assert_eq!(
            map_event_type("payment_intent.payment_failed"),
            (PaymentEventType::PaymentFailed, PaymentStatus::Failed)
        );
    }

    #[test]
    fn maps_payment_status_variants() {
        assert_eq!(
            map_payment_status(Some("processing"), None),
            PaymentStatus::Processing
        );
        assert_eq!(
            map_payment_status(Some("requires_capture"), None),
            PaymentStatus::Authorized
        );
        assert_eq!(
            map_payment_status(Some("canceled"), None),
            PaymentStatus::Cancelled
        );
        assert_eq!(
            map_payment_status(Some("expired"), None),
            PaymentStatus::Expired
        );
        assert_eq!(
            map_payment_status(Some("refunded"), None),
            PaymentStatus::Refunded
        );
        assert_eq!(
            map_payment_status(Some("partially_refunded"), None),
            PaymentStatus::PartiallyRefunded
        );
        assert_eq!(
            map_payment_status(Some("unknown"), None),
            PaymentStatus::Processing
        );
    }

    #[test]
    fn maps_refund_and_event_variants() {
        assert_eq!(
            map_refund_status(Some("succeeded")),
            PaymentStatus::Refunded
        );
        assert_eq!(map_refund_status(Some("failed")), PaymentStatus::Failed);
        assert_eq!(
            map_event_type("payment_intent.canceled"),
            (PaymentEventType::PaymentCancelled, PaymentStatus::Cancelled)
        );
        assert_eq!(
            map_event_type("charge.refunded"),
            (PaymentEventType::PaymentRefunded, PaymentStatus::Refunded)
        );
        assert_eq!(
            map_event_type("refund.created"),
            (PaymentEventType::RefundCreated, PaymentStatus::Processing)
        );
    }
}
