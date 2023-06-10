use spayd_rs::{NotifyType, PaymentType, Spayd};

fn main() {
    let spayd = Spayd::builder()
        .account("CZ7907000000001234567890".to_string())
        .amount("239.50".to_string())
        .currency("CZK".to_string())
        .reference("123121".to_string())
        .recipient("ABCDEF1/+. PO:".to_string())
        .date("20230810".to_string())
        .payment_type(PaymentType::Instant)
        .message("PAYMENT".to_string())
        .notify(NotifyType::Email)
        .notify_address("email@example.com".to_string())
        .build();

    let result = spayd.spayd_string_unchecked();

    println!("{}", result);
}
