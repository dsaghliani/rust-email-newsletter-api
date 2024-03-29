pub use new_subscriber::NewSubscriber;
pub use subscriber_email::SubscriberEmail;
pub use subscriber_name::SubscriberName;

mod subscriber_email {
    use serde::Deserialize;
    use validator::{Validate, ValidationErrors};

    #[derive(Debug, Deserialize, Validate)]
    pub struct SubscriberEmail {
        #[validate(email)]
        email: String,
    }

    impl SubscriberEmail {
        pub fn parse(email: String) -> Result<Self, ValidationErrors> {
            let subscriber_email = Self { email };
            subscriber_email.validate()?;
            Ok(subscriber_email)
        }
    }

    impl AsRef<str> for SubscriberEmail {
        fn as_ref(&self) -> &str {
            &self.email
        }
    }

    #[cfg(test)]
    mod tests {
        use super::SubscriberEmail;
        use fake::{faker::internet::en::SafeEmail, Fake};
        use k9::assert_err;
        use quickcheck_macros::quickcheck;
        use rand::{rngs::StdRng, SeedableRng};
        use validator::Validate;

        #[derive(Clone, Debug)]
        struct ValidEmailFixture(pub String);

        impl quickcheck::Arbitrary for ValidEmailFixture {
            fn arbitrary(g: &mut quickcheck::Gen) -> Self {
                let mut rng = StdRng::seed_from_u64(u64::arbitrary(g));
                let email = SafeEmail().fake_with_rng(&mut rng);
                Self(email)
            }
        }

        #[quickcheck]
        fn valid_emails_are_parsed_successfully(
            ValidEmailFixture(email): ValidEmailFixture,
        ) -> bool {
            SubscriberEmail { email }.validate().is_ok()
        }

        #[test]
        fn empty_string_is_rejected() {
            let email = String::new();
            assert_err!(SubscriberEmail { email }.validate());
        }

        #[test]
        fn email_missing_at_symbol_is_rejected() {
            let email = "ursuladomain.com".to_string();
            assert_err!(SubscriberEmail { email }.validate());
        }

        #[test]
        fn email_missing_subject_is_rejected() {
            let email = "@domain.com".to_string();
            assert_err!(SubscriberEmail { email }.validate());
        }
    }
}
mod subscriber_name {
    use serde::Deserialize;
    use validator::{Validate, ValidationError};

    #[derive(Debug, Deserialize, Validate)]
    pub struct SubscriberName {
        #[validate(
            length(min = 1, max = 256),
            custom = "is_not_all_whitespace_and_does_not_contain_invalid_characters"
        )]
        name: String,
    }

    fn is_not_all_whitespace_and_does_not_contain_invalid_characters(
        value: &str,
    ) -> Result<(), ValidationError> {
        // Validate it's not all whitespace.
        if value.trim().is_empty() {
            return Err(ValidationError::new("must not be all whitespace"));
        }

        // Validate it doesn't contain invalid characters.
        let invalid_characters = ['/', '(', ')', '"', '<', '>', '\\', '{', '}'];

        for character in value.chars() {
            if invalid_characters
                .into_iter()
                .any(|invalid_character| invalid_character == character)
            {
                return Err(ValidationError::new(
                    "may not contain any of the following characters: \
                    /, (, ), \", <, >, \\, {, }",
                ));
            }
        }

        Ok(())
    }

    impl AsRef<str> for SubscriberName {
        fn as_ref(&self) -> &str {
            &self.name
        }
    }

    #[cfg(test)]
    mod tests {
        use super::SubscriberName;
        use k9::{assert_err, assert_ok};
        use validator::Validate;

        #[test]
        fn a_256_grapheme_long_name_is_valid() {
            let name = "a".repeat(256);
            assert_ok!(SubscriberName { name }.validate());
        }

        #[test]
        fn name_longer_than_256_graphemes_is_rejected() {
            let name = "a".repeat(257);
            assert_err!(SubscriberName { name }.validate());
        }

        #[test]
        fn whitespace_only_names_are_rejected() {
            let name = " ".to_string();
            assert_err!(SubscriberName { name }.validate());
        }

        #[test]
        fn empty_string_is_rejected() {
            let name = String::new();
            assert_err!(SubscriberName { name }.validate());
        }

        #[test]
        fn names_containing_an_invalid_character_are_rejected() {
            for name in &['/', '(', ')', '"', '<', '>', '\\', '{', '}'] {
                let name = name.to_string();
                assert_err!(SubscriberName { name }.validate());
            }
        }

        #[test]
        fn valid_name_is_parsed_successfully() {
            let name = "Ursula Le Guin".to_string();
            assert_ok!(SubscriberName { name }.validate());
        }
    }
}

mod new_subscriber {
    use super::{SubscriberEmail, SubscriberName};
    use serde::Deserialize;
    use validator::Validate;

    #[derive(Debug, Deserialize, Validate)]
    pub struct NewSubscriber {
        #[serde(flatten)]
        #[validate]
        pub name: SubscriberName,
        #[serde(flatten)]
        #[validate]
        pub email: SubscriberEmail,
    }
}
