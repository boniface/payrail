use crate::{MobileMoneyOperator, PaymentEventType, PaymentStatus};

pub(crate) fn map_status(status: &str) -> PaymentStatus {
    match status {
        "Pending" | "pending" => PaymentStatus::Pending,
        "Successful" | "successful" | "Success" | "success" => PaymentStatus::Succeeded,
        "Failed" | "failed" => PaymentStatus::Failed,
        "Unknown" | "unknown" => PaymentStatus::Processing,
        _ => PaymentStatus::Processing,
    }
}

pub(crate) fn map_event_type(status: PaymentStatus) -> PaymentEventType {
    match status {
        PaymentStatus::Succeeded => PaymentEventType::PaymentSucceeded,
        PaymentStatus::Failed => PaymentEventType::PaymentFailed,
        PaymentStatus::Cancelled => PaymentEventType::PaymentCancelled,
        PaymentStatus::Pending => PaymentEventType::PaymentPending,
        PaymentStatus::Created
        | PaymentStatus::RequiresAction
        | PaymentStatus::Processing
        | PaymentStatus::Authorized
        | PaymentStatus::Expired
        | PaymentStatus::Refunded
        | PaymentStatus::PartiallyRefunded => PaymentEventType::PaymentPending,
    }
}

pub(crate) fn map_payment_type(payment_type: &str) -> MobileMoneyOperator {
    match payment_type {
        "MtnMoney" | "MTNMoney" => MobileMoneyOperator::Mtn,
        "AirtelMoney" => MobileMoneyOperator::Airtel,
        "ZamtelKwacha" => MobileMoneyOperator::Zamtel,
        other => MobileMoneyOperator::Other(other.to_owned()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_successful_status() {
        assert_eq!(map_status("Successful"), PaymentStatus::Succeeded);
    }

    #[test]
    fn maps_mtn_variants() {
        assert_eq!(map_payment_type("MTNMoney"), MobileMoneyOperator::Mtn);
    }

    #[test]
    fn maps_status_and_event_variants() {
        assert_eq!(map_status("Pending"), PaymentStatus::Pending);
        assert_eq!(map_status("Failed"), PaymentStatus::Failed);
        assert_eq!(map_status("Unknown"), PaymentStatus::Processing);
        assert_eq!(map_status("Other"), PaymentStatus::Processing);
        assert_eq!(
            map_event_type(PaymentStatus::Failed),
            PaymentEventType::PaymentFailed
        );
        assert_eq!(
            map_event_type(PaymentStatus::Cancelled),
            PaymentEventType::PaymentCancelled
        );
        assert_eq!(
            map_event_type(PaymentStatus::Processing),
            PaymentEventType::PaymentPending
        );
    }

    #[test]
    fn maps_payment_type_variants() {
        assert_eq!(map_payment_type("MtnMoney"), MobileMoneyOperator::Mtn);
        assert_eq!(map_payment_type("AirtelMoney"), MobileMoneyOperator::Airtel);
        assert_eq!(
            map_payment_type("ZamtelKwacha"),
            MobileMoneyOperator::Zamtel
        );
        assert_eq!(
            map_payment_type("Bank"),
            MobileMoneyOperator::Other("Bank".to_owned())
        );
    }
}
