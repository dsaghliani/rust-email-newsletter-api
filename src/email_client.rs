use crate::domain::SubscriberEmail;
use secrecy::{ExposeSecret, Secret};
use serde_json::json;

pub struct EmailClient {
    sender: SubscriberEmail,
    base_url: String,
    http_client: reqwest::Client,
    authorization_token: Secret<String>,
}

impl EmailClient {
    #[must_use]
    pub fn new(
        sender: SubscriberEmail,
        base_url: String,
        authorization_token: Secret<String>,
    ) -> Self {
        Self {
            sender,
            base_url,
            authorization_token,
            http_client: reqwest::Client::new(),
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
        recipient: SubscriberEmail,
        subject: &str,
        html_content: &str,
        text_content: &str,
    ) -> Result<(), reqwest::Error> {
        let url = format!("{}/v3/mail/send", self.base_url);
        let request_body = SendEmailRequest {
            from: self.sender.as_ref(),
            to: recipient.as_ref(),
            subject,
            html_content,
            text_content,
        }
        .json();
        let auth_token =
            format!("Bearer {}", self.authorization_token.expose_secret());

        self.http_client
            .post(url)
            .header("Authorization", auth_token)
            .json(&request_body)
            .send()
            .await?;

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
    #![allow(clippy::unwrap_used)]

    use crate::{domain::SubscriberEmail, EmailClient};
    use fake::{
        faker::{
            internet::en::SafeEmail,
            lorem::en::{Paragraph, Sentence},
        },
        Fake, Faker,
    };
    use secrecy::Secret;
    use wiremock::{matchers::any, Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn send_email_fires_request_to_base_url() {
        // Arrange.
        let mock_server = MockServer::start().await;
        let email_client = {
            let sender = SubscriberEmail::parse(SafeEmail().fake()).unwrap();
            EmailClient::new(sender, mock_server.uri(), Secret::new(Faker.fake()))
        };

        Mock::given(any())
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;

        let subscriber_email = SubscriberEmail::parse(SafeEmail().fake()).unwrap();
        let subject: String = Sentence(1..2).fake();
        let content: String = Paragraph(1..10).fake();

        // Act.
        let _ = email_client
            .send_email(subscriber_email, &subject, &content, &content)
            .await;

        // Assert.
        // Assertion is done automatically by the `MockServer`: if it doesn't
        // receive the request(s) as specified by `Mock::given(...)...`, it'll
        // panic in its `drop()` method.
    }
}
