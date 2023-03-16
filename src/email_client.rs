use crate::domain::SubscriberEmail;
use secrecy::{ExposeSecret, Secret};
use serde_json::json;
use std::time::Duration;

pub struct EmailClient {
    sender: SubscriberEmail,
    base_url: String,
    http_client: reqwest::Client,
    authorization_token: Secret<String>,
}

impl EmailClient {
    /// Create a new `EmailClient`.
    ///
    /// # Panics
    ///
    /// Panics if the HTTP client cannot be built.
    #[must_use]
    #[allow(clippy::unwrap_used)]
    pub fn new(
        sender: SubscriberEmail,
        base_url: String,
        authorization_token: Secret<String>,
        timeout: Duration,
    ) -> Self {
        Self {
            sender,
            base_url,
            authorization_token,
            http_client: reqwest::Client::builder()
                .timeout(timeout)
                .build()
                .unwrap(),
        }
    }

    /// Send an email to the SendGrid API.
    ///
    /// # Errors
    ///
    /// Returns an error if something goes wrong sending the request.
    #[allow(clippy::doc_markdown)]
    pub async fn send_email(
        &self,
        recipient: &SubscriberEmail,
        subject: &str,
        html_content: &str,
        text_content: &str,
    ) -> Result<(), reqwest::Error> {
        let url = format!("{}/v3/mail/send", self.base_url);
        let auth_token =
            format!("Bearer {}", self.authorization_token.expose_secret());
        let request_body = SendEmailRequest {
            from: self.sender.as_ref(),
            to: recipient.as_ref(),
            subject,
            html_content,
            text_content,
        }
        .json();

        self.http_client
            .post(url)
            .header("Authorization", auth_token)
            .json(&request_body)
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }
}

struct SendEmailRequest<'a> {
    from: &'a str,
    to: &'a str,
    subject: &'a str,
    html_content: &'a str,
    text_content: &'a str,
}

impl<'a> SendEmailRequest<'a> {
    pub fn json(self) -> serde_json::Value {
        json!({
            "personalizations": [{ "to": [{ "email": self.to }] }],
            "from": self.from,
            "subject": self.subject,
            "content": [
                {
                    "type": "text/html",
                    "value": self.html_content
                },
                {
                    "type": "text/plain",
                    "value": self.text_content
                }
            ]
        })
    }
}

#[cfg(test)]
mod tests {
    use helpers::{content, email, email_client, subject};
    use k9::{assert_err, assert_ok};
    use matchers::email_body_matches;
    use std::time::Duration;
    use wiremock::{
        matchers::{any, header, header_exists, method, path},
        Mock, MockServer, ResponseTemplate,
    };

    #[tokio::test]
    async fn send_email_fires_request_to_base_url() {
        // Arrange.
        let mock_server = MockServer::start().await;
        let email_client = email_client(mock_server.uri());

        Mock::given(header_exists("Authorization"))
            .and(header("Content-Type", "application/json"))
            .and(path("/v3/mail/send"))
            .and(method("POST"))
            .and(email_body_matches())
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;

        // Act.
        let _ = email_client
            .send_email(&email(), &subject(), &content(), &content())
            .await;

        // Assert.
        // Assertion is done automatically by the `MockServer`: if it doesn't
        // receive the request(s) as specified by `Mock::given(...)...`, it'll
        // panic in its `drop()` method.
    }

    #[tokio::test]
    async fn send_email_succeeds_if_server_returns_200() {
        // Arrange.
        let mock_server = MockServer::start().await;
        let email_client = email_client(mock_server.uri());

        Mock::given(any())
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;

        // Act.
        let outcome = email_client
            .send_email(&email(), &subject(), &content(), &content())
            .await;

        // Assert.
        assert_ok!(outcome);
    }

    #[tokio::test]
    async fn send_email_fails_if_server_returns_500() {
        // Arrange.
        let mock_server = MockServer::start().await;
        let email_client = email_client(mock_server.uri());

        Mock::given(any())
            .respond_with(ResponseTemplate::new(500))
            .expect(1)
            .mount(&mock_server)
            .await;

        // Act.
        let outcome = email_client
            .send_email(&email(), &subject(), &content(), &content())
            .await;

        // Assert.
        assert_err!(outcome);
    }

    #[tokio::test]
    async fn send_email_times_out_if_server_takes_too_long() {
        // Arrange.
        let mock_server = MockServer::start().await;
        let email_client = email_client(mock_server.uri());

        Mock::given(any())
            .respond_with(
                ResponseTemplate::new(500).set_delay(Duration::from_secs(180)),
            )
            .expect(1)
            .mount(&mock_server)
            .await;

        // Act.
        let outcome = email_client
            .send_email(&email(), &subject(), &content(), &content())
            .await;

        // Assert.
        assert_err!(outcome);
    }

    mod helpers {
        use std::time::Duration;

        use fake::{
            faker::{
                internet::en::SafeEmail,
                lorem::en::{Paragraph, Sentence},
            },
            Fake, Faker,
        };
        use secrecy::Secret;

        use crate::{domain::SubscriberEmail, EmailClient};

        pub fn subject() -> String {
            Sentence(1..2).fake()
        }

        pub fn content() -> String {
            Paragraph(1..10).fake()
        }

        pub fn email() -> SubscriberEmail {
            #[allow(clippy::unwrap_used)]
            SubscriberEmail::parse(SafeEmail().fake()).unwrap()
        }

        pub fn email_client(base_url: String) -> EmailClient {
            EmailClient::new(
                email(),
                base_url,
                Secret::new(Faker.fake()),
                Duration::from_millis(200),
            )
        }
    }

    mod matchers {
        pub const fn email_body_matches() -> SendEmailBodyMatcher {
            SendEmailBodyMatcher
        }

        pub struct SendEmailBodyMatcher;

        impl wiremock::Match for SendEmailBodyMatcher {
            fn matches(&self, request: &wiremock::Request) -> bool {
                serde_json::from_slice(&request.body).map_or(
                    false,
                    |body: serde_json::Value| {
                        body.get("personalizations").is_some()
                            && body.get("from").is_some()
                            && body.get("subject").is_some()
                            && body.get("content").is_some()
                    },
                )
            }
        }
    }
}
