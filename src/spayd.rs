use iso_4217::*;
use regex::Regex;
use typed_builder::TypedBuilder;

#[cfg(feature = "qrcode")]
use qrcode::QrResult;

/// Error enum
#[derive(Debug, PartialEq)]
pub enum SpaydError {
    /// Invalid account
    InvalidAccountNumber(&'static str),

    /// Invalid amount
    InvalidAmount(&'static str),

    /// Invalid currency
    InvalidCurrency(&'static str),

    /// Invalid reference
    InvalidReference(&'static str),

    /// Invalid recipient
    InvalidRecipient(&'static str),

    /// Invalid date
    InvalidDate(&'static str),

    /// Invalid payment type
    InvalidPaymentType(&'static str),

    /// Invalid message
    InvalidMessage(&'static str),

    /// Invalid notify address
    InvalidNotifyAddress(&'static str),
}

/// Payment type
#[derive(Debug)]
pub enum PaymentType {
    /// Instant payment (if the bank supports it)
    Instant,

    /// Other payment type (max. 3 characters)
    Other(String),
}

/// Notify type
#[derive(Debug)]
pub enum NotifyType {
    /// Phone notification
    Phone,

    /// Email notification
    Email,
}

/// SPAYD data structure
#[derive(Debug, TypedBuilder)]
pub struct Spayd {
    account: String,
    amount: String,

    #[builder(default, setter(strip_option))]
    currency: Option<String>,

    #[builder(default, setter(strip_option))]
    reference: Option<String>,

    #[builder(default, setter(strip_option))]
    recipient: Option<String>,

    #[builder(default, setter(strip_option))]
    date: Option<String>,

    #[builder(default, setter(strip_option))]
    payment_type: Option<PaymentType>,

    #[builder(default, setter(strip_option))]
    message: Option<String>,

    #[builder(default, setter(strip_option))]
    notify: Option<NotifyType>,

    #[builder(default, setter(strip_option))]
    notify_address: Option<String>,
}

impl Spayd {
    /// Generate SPAYD string
    pub fn spayd_string(&self) -> Result<String, SpaydError> {
        self.validate()?;

        Ok(self.build_string())
    }

    /// Generate SPAYD string without input data validation
    pub fn spayd_string_unchecked(&self) -> String {
        self.build_string()
    }

    /// Generate payment QR code
    #[cfg(feature = "qrcode")]
    pub fn qrcode(&self) -> QrResult<qrcode::QrCode> {
        qrcode::QrCode::new(self.spayd_string().unwrap())
    }

    fn build_string(&self) -> String {
        let mut v: Vec<String> = Vec::with_capacity(11);

        v.push("SPD".to_string()); // header
        v.push("1.0".to_string()); // version
        v.push(format!("ACC:{}", self.account));
        v.push(format!("AM:{}", self.amount));

        if let Some(ref currency) = self.currency {
            v.push(format!("CC:{}", currency));
        }

        if let Some(ref reference) = self.reference {
            v.push(format!("RF:{}", reference));
        }

        if let Some(ref recipient) = self.recipient {
            v.push(format!("RN:{}", recipient));
        }

        if let Some(ref date) = self.date {
            v.push(format!("DT:{}", date));
        }

        if let Some(ref payment_type) = self.payment_type {
            let pt = match payment_type {
                PaymentType::Instant => "IP",
                PaymentType::Other(s) => s,
            };

            v.push(format!("PT:{}", pt));
        }

        if let Some(ref message) = self.message {
            v.push(format!("MSG:{}", message));
        }

        if let Some(ref notify) = self.notify {
            let val = match notify {
                NotifyType::Phone => "P",
                NotifyType::Email => "E",
            };
            v.push(format!("NT:{}", val));
        }

        if let Some(ref notify_address) = self.notify_address {
            v.push(format!("NTA:{}", notify_address));
        }

        v.join("*")
    }

    fn validate(&self) -> Result<(), SpaydError> {
        let re_iban = Regex::new(r"^[A-Z]{2}\d{2}[0-9A-Z]{1,30}$").expect("IBAN regex is valid");
        let re_amount = Regex::new(r"^\d+(\.\d{1,2})?$").expect("Amount regex is valid");
        let re_digits = Regex::new(r"^[0-9]+$").expect("Digits-only regex is valid");
        let re_all_allowed =
            Regex::new(r"^[0-9A-Z $%+\-./:]+$").expect("Allowed characters regex is valid");
        let re_date = Regex::new(r"^([12]\d{3}(0[1-9]|1[0-2])(0[1-9]|[12]\d|3[01]))$")
            .expect("Date regex is valid");
        let re_phone = Regex::new(r"^\+?\d+$").expect("Phone regex is valid");
        let re_email = Regex::new(
            r"^([a-z0-9_+]([a-z0-9_+.]*[a-z0-9_+])?)@([a-z0-9]+([\-\.]{1}[a-z0-9]+)*\.[a-z]{2,6})",
        )
        .expect("Email regex is valid");

        // account number
        if !re_iban.is_match(&self.account) {
            return Err(SpaydError::InvalidAccountNumber(
                "Value is not a valid IBAN",
            ));
        }

        // amount
        if self.amount.len() > 10 {
            return Err(SpaydError::InvalidAmount(
                "Exceeded maximum length of 10 characters",
            ));
        } else if !re_amount.is_match(&self.amount) {
            return Err(SpaydError::InvalidAmount(
                "Value is not in a decimal format. Maximum number of decimal places is 2.",
            ));
        }

        // currency
        if let Some(ref currency) = self.currency {
            (TryFrom::try_from(currency.as_str()) as Result<CurrencyCode, ParseCodeError>)
                .map_err(|_| SpaydError::InvalidCurrency("Invalid currency code"))?;
        }

        // reference
        if let Some(ref reference) = self.reference {
            if reference.len() > 16 {
                return Err(SpaydError::InvalidReference(
                    "Exceeded maximum length of 16 characters",
                ));
            } else if !re_digits.is_match(reference) {
                return Err(SpaydError::InvalidReference(
                    "Value contains non-digit characters",
                ));
            }
        }

        // recipient
        if let Some(ref recipient) = self.recipient {
            if recipient.len() > 35 {
                return Err(SpaydError::InvalidRecipient(
                    "Exceeded maximum length of 35 characters",
                ));
            } else if !re_all_allowed.is_match(recipient) {
                return Err(SpaydError::InvalidRecipient(
                    "Value contains forbidden character(s)",
                ));
            }
        }

        // date
        if let Some(ref date) = self.date {
            if !re_date.is_match(date) {
                return Err(SpaydError::InvalidDate("Date is not in YYYYMMDD format"));
            }
        }

        // payment_type
        if let Some(ref payment_type) = self.payment_type {
            if let PaymentType::Other(s) = payment_type {
                if s.len() > 3 {
                    return Err(SpaydError::InvalidPaymentType(
                        "Exceeded maximum length of 3 characters",
                    ));
                } else if !re_all_allowed.is_match(s) {
                    return Err(SpaydError::InvalidPaymentType(
                        "Value contains forbidden character(s)",
                    ));
                }
            }
        }

        // message
        if let Some(ref message) = self.message {
            if message.len() > 60 {
                return Err(SpaydError::InvalidMessage(
                    "Exceeded maximum length of 60 characters",
                ));
            } else if !re_all_allowed.is_match(message) {
                return Err(SpaydError::InvalidRecipient(
                    "Value contains forbidden character(s)",
                ));
            }
        }

        // notify (no need to validate)

        // notify_address
        if let Some(ref notify_address) = self.notify_address {
            if notify_address.len() > 320 {
                return Err(SpaydError::InvalidNotifyAddress(
                    "Exceeded maximum length of 320 characters",
                ));
            }

            if let Some(ref notify) = self.notify {
                match notify {
                    NotifyType::Phone if !re_phone.is_match(notify_address) => {
                        return Err(SpaydError::InvalidNotifyAddress("Invalid phone number"));
                    }
                    NotifyType::Email if !re_email.is_match(notify_address) => {
                        return Err(SpaydError::InvalidNotifyAddress("Invalid email address"));
                    }
                    _ => {}
                }
            } else {
                return Err(SpaydError::InvalidNotifyAddress(
                    "Notify type was not provided",
                ));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::spayd::*;

    #[test]
    fn basic_works() {
        let spayd = Spayd::builder()
            .account("CZ5508000000001234567899".to_string())
            .amount("239.50".to_string())
            .build();

        let result = spayd.spayd_string();

        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            "SPD*1.0*ACC:CZ5508000000001234567899*AM:239.50".to_string()
        );
    }

    #[test]
    fn invalid_account_fails() {
        let spayd = Spayd::builder()
            .account("C1Z7955000000001027699338".to_string())
            .amount("239.50".to_string())
            .build();

        let result = spayd.spayd_string();

        assert!(result.is_err());
        assert_eq!(
            result,
            Err(SpaydError::InvalidAccountNumber(
                "Value is not a valid IBAN"
            ))
        );
    }

    #[test]
    fn invalid_amount_fails() {
        let spayd = Spayd::builder()
            .account("CZ5508000000001234567899".to_string())
            .amount("239.500".to_string())
            .build();

        let result = spayd.spayd_string();

        assert!(result.is_err());
        assert_eq!(
            result,
            Err(SpaydError::InvalidAmount(
                "Value is not in a decimal format. Maximum number of decimal places is 2."
            ))
        );
    }

    #[test]
    fn reference_works() {
        let spayd = Spayd::builder()
            .account("CZ5508000000001234567899".to_string())
            .amount("239.50".to_string())
            .reference("123121".to_string())
            .build();

        let result = spayd.spayd_string();

        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            "SPD*1.0*ACC:CZ5508000000001234567899*AM:239.50*RF:123121".to_string()
        );
    }

    #[test]
    fn invalid_reference_fails() {
        let spayd = Spayd::builder()
            .account("CZ5508000000001234567899".to_string())
            .amount("239.50".to_string())
            .reference("123121123A".to_string())
            .build();

        let result = spayd.spayd_string();

        assert!(result.is_err());
        assert_eq!(
            result,
            Err(SpaydError::InvalidReference(
                "Value contains non-digit characters"
            ))
        );
    }

    #[test]
    fn recipient_works() {
        let spayd = Spayd::builder()
            .account("CZ5508000000001234567899".to_string())
            .amount("239.50".to_string())
            .recipient("MISTR1/+.% PO:".to_string())
            .build();

        let result = spayd.spayd_string();

        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            "SPD*1.0*ACC:CZ5508000000001234567899*AM:239.50*RN:MISTR1/+.% PO:".to_string()
        );
    }

    #[test]
    fn invalid_recipient_fails() {
        let spayd = Spayd::builder()
            .account("CZ5508000000001234567899".to_string())
            .amount("239.50".to_string())
            .recipient("MISTR1/+*.% PO:".to_string())
            .build();

        let result = spayd.spayd_string();

        assert!(result.is_err());
        assert_eq!(
            result,
            Err(SpaydError::InvalidRecipient(
                "Value contains forbidden character(s)"
            ))
        );
    }

    #[test]
    fn full_works() {
        let spayd = Spayd::builder()
            .account("CZ5508000000001234567899".to_string())
            .amount("239.50".to_string())
            .currency("CZK".to_string())
            .reference("123121".to_string())
            .recipient("MISTR1/+.% PO:".to_string())
            .date("20230810".to_string())
            .payment_type(PaymentType::Instant)
            .message("PAYMENT".to_string())
            .notify(NotifyType::Email)
            .notify_address("email@example.com".to_string())
            .build();

        let result = spayd.spayd_string();

        dbg!(&result);
        assert!(result.is_ok());
        // assert_eq!(
        //     result.unwrap(),
        //     "SPD*1.0*ACC:CZ5508000000001234567899*AM:239.50*RN:MISTR1/+.% PO:".to_string()
        // );
    }
}
