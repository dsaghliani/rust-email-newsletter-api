pub use new_subscriber::NewSubscriber;
pub use subscriber_email::SubscriberEmail;
pub use subscriber_name::SubscriberName;

mod subscriber_email {
    use serde::Deserialize;
    use validator::Validate;

    #[derive(Deserialize, Validate)]
    pub struct SubscriberEmail {
        #[validate(email)]
        email: String,
    }

    impl AsRef<str> for SubscriberEmail {
        fn as_ref(&self) -> &str {
            &self.email
        }
    }

    #[cfg(test)]
    mod tests {}
}
mod subscriber_name {
    use serde::Deserialize;
    use validator::{Validate, ValidationError, ValidationErrors};

    #[derive(Deserialize, Debug, Validate)]
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

    impl SubscriberName {
        pub fn parse(name: String) -> Result<Self, ValidationErrors> {
            let subscriber_name = Self { name };
            subscriber_name.validate()?;
            Ok(subscriber_name)
        }
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

        #[test]
        fn a_256_grapheme_long_name_is_valid() {
            let name = "a".repeat(256);
            assert_ok!(SubscriberName::parse(name));
        }

        #[test]
        fn name_longer_than_256_graphemes_is_rejected() {
            let name = "a".repeat(257);
            assert_err!(SubscriberName::parse(name));
        }

        #[test]
        fn whitespace_only_names_are_rejected() {
            let name = " ".to_string();
            assert_err!(SubscriberName::parse(name));
        }

        #[test]
        fn empty_string_is_rejected() {
            let name = String::new();
            assert_err!(SubscriberName::parse(name));
        }

        #[test]
        fn names_containing_an_invalid_character_are_rejected() {
            for name in &['/', '(', ')', '"', '<', '>', '\\', '{', '}'] {
                let name = name.to_string();
                assert_err!(SubscriberName::parse(name));
            }
        }

        #[test]
        fn valid_name_is_parsed_successfully() {
            let name = "Ursula Le Guin".to_string();
            assert_ok!(SubscriberName::parse(name));
        }
    }
}

mod new_subscriber {
    use super::{SubscriberEmail, SubscriberName};
    use serde::Deserialize;
    use validator::Validate;

    #[derive(Deserialize, Validate)]
    pub struct NewSubscriber {
        #[serde(flatten)]
        #[validate]
        pub name: SubscriberName,
        #[serde(flatten)]
        #[validate]
        pub email: SubscriberEmail,
    }
}
