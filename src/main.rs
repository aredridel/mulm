use mailparse::{parse_mail, MailParseError};
use std::io::{self, Read};

#[derive(Debug, Eq, PartialEq)]
enum MailingListAction<'a> {
    Subscribe(String),
    Unsubscribe(String),
    Message(&'a [u8]),
    Reject,
}

fn main() {
    let stdin = io::stdin();
    let mut rh = stdin.lock();

    let mut buffer = Vec::new();
    rh.read_to_end(&mut buffer).unwrap();
    handle(buffer.as_slice()).unwrap();
}

fn handle(mail: &[u8]) -> Result<(), MailParseError> {
    let action = action_for_mail(mail)?;
    println!("{:?}", action);
    return Ok(());
}

fn action_for_mail(mail: &[u8]) -> Result<MailingListAction, MailParseError> {
    let mail_rec = parse_mail(mail)?;
    let from = mail_rec
        .headers
        .iter()
        .find(|&h| &h.get_key().to_lowercase() == "from");

    if let None = from {
        return Ok(MailingListAction::Reject);
    }

    let subject = mail_rec
        .headers
        .iter()
        .find(|&h| &h.get_key().to_lowercase() == "subject");
    // If command
    //      Handle command
    // Else if list allows outside posting or sender is member of list
    //      Send to list
    // Else
    //      reject

    if let Some(subject) = subject {
        if subject.get_value().to_lowercase().trim() == "subscribe" {
            return Ok(MailingListAction::Subscribe(from.unwrap().get_value()));
        } else if subject.get_value().to_lowercase().trim() == "unsubscribe" {
            return Ok(MailingListAction::Unsubscribe(from.unwrap().get_value()));
        }
    }

    println!("{:?}", mail_rec);
    return Ok(MailingListAction::Message(mail));
}

#[cfg(test)]
mod test {
    use super::{action_for_mail, MailingListAction};
    use mailparse::MailParseError;

    #[test]
    fn a_basic_parse() {
        assert_eq!(
            action_for_mail("Subject: test\r\n\r\n".as_bytes()).unwrap(),
            MailingListAction::Reject
        );
    }

    #[test]
    fn a_subscribe_command() {
        assert_eq!(
            action_for_mail("From: test@example.org\r\nSubject: subsCRIbe     \r\n\r\n".as_bytes())
                .unwrap(),
            MailingListAction::Subscribe("test@example.org".to_string())
        );
    }

    #[test]
    fn an_unsubscribe_command() {
        assert_eq!(
            action_for_mail("From: test@example.org\r\nSubject: UNSUbscribe\r\n\r\n".as_bytes())
                .unwrap(),
            MailingListAction::Unsubscribe("test@example.org".to_string())
        );
    }

    #[test]
    fn a_message_to_pass_through() {
        let message = "From: test@example.org\r\nSubject: A message to the list\r\n\r\n".as_bytes();

        assert_eq!(
            action_for_mail(message)
                .unwrap(),
            MailingListAction::Message(message)
        );
    }

    #[test]
    fn a_bad_email() {
        if let MailParseError::Generic(msg) =
            action_for_mail("bad input\r\n".as_bytes()).unwrap_err()
        {
            assert_eq!(msg, "Unexpected newline in header key")
        } else {
            panic!()
        }
    }
}
