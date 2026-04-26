use payrail_core::{PaymentEventType, PaymentStatus};

pub(crate) fn map_order_status(status: &str) -> PaymentStatus {
    match status {
        "CREATED" | "SAVED" => PaymentStatus::Created,
        "APPROVED" => PaymentStatus::Authorized,
        "VOIDED" => PaymentStatus::Cancelled,
        "COMPLETED" => PaymentStatus::Succeeded,
        "PAYER_ACTION_REQUIRED" => PaymentStatus::RequiresAction,
        _ => PaymentStatus::Processing,
    }
}

pub(crate) fn map_event(event_type: &str) -> (PaymentEventType, PaymentStatus) {
    match event_type {
        "CHECKOUT.ORDER.APPROVED" => (
            PaymentEventType::PaymentRequiresAction,
            PaymentStatus::Authorized,
        ),
        "CHECKOUT.ORDER.COMPLETED" => {
            (PaymentEventType::PaymentSucceeded, PaymentStatus::Succeeded)
        }
        "CHECKOUT.ORDER.VOIDED" => (PaymentEventType::PaymentCancelled, PaymentStatus::Cancelled),
        _ => (PaymentEventType::PaymentPending, PaymentStatus::Processing),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_completed_order() {
        assert_eq!(map_order_status("COMPLETED"), PaymentStatus::Succeeded);
    }

    #[test]
    fn maps_order_status_variants() {
        assert_eq!(map_order_status("CREATED"), PaymentStatus::Created);
        assert_eq!(map_order_status("SAVED"), PaymentStatus::Created);
        assert_eq!(map_order_status("APPROVED"), PaymentStatus::Authorized);
        assert_eq!(map_order_status("VOIDED"), PaymentStatus::Cancelled);
        assert_eq!(
            map_order_status("PAYER_ACTION_REQUIRED"),
            PaymentStatus::RequiresAction
        );
        assert_eq!(map_order_status("OTHER"), PaymentStatus::Processing);
    }

    #[test]
    fn maps_webhook_event_variants() {
        assert_eq!(
            map_event("CHECKOUT.ORDER.APPROVED"),
            (
                PaymentEventType::PaymentRequiresAction,
                PaymentStatus::Authorized
            )
        );
        assert_eq!(
            map_event("CHECKOUT.ORDER.VOIDED"),
            (PaymentEventType::PaymentCancelled, PaymentStatus::Cancelled)
        );
        assert_eq!(
            map_event("UNKNOWN"),
            (PaymentEventType::PaymentPending, PaymentStatus::Processing)
        );
    }
}
