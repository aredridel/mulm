use mailparse::{parse_mail, MailHeaderMap, MailParseError};
use std::io::{self, Read};
use std::env::args;

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

    if let Some(list) = args().nth(1) {
        handle(buffer.as_slice(), list).unwrap();
    } else {
        panic!("No list given");
    }
}

fn handle(mail: &[u8], list: String) -> Result<(), MailParseError> {
    let action = action_for_mail(mail)?;

    println!("{:?} {:?}", action, list);

    match action {
        MailingListAction::Subscribe(_email) => (),
        MailingListAction::Unsubscribe(_email) => (),
        MailingListAction::Message(_message) => (),
        MailingListAction::Reject => (),
    }

    return Ok(());
}

fn action_for_mail(mail: &[u8]) -> Result<MailingListAction, MailParseError> {
    let mail_rec = parse_mail(mail)?;
    let from = mail_rec.headers.get_first_value("from");

    if let None = from {
        return Ok(MailingListAction::Reject);
    }

    let subject = mail_rec.headers.get_first_value("subject");

    if let Some(subject) = subject {
        if subject.to_lowercase().trim() == "subscribe" {
            return Ok(MailingListAction::Subscribe(from.unwrap()));
        } else if subject.to_lowercase().trim() == "unsubscribe" {
            return Ok(MailingListAction::Unsubscribe(from.unwrap()));
        }
    }

    return Ok(MailingListAction::Message(mail));
}

#[cfg(test)]
mod test {
    use super::{action_for_mail, MailingListAction};
    use mailparse::MailParseError;

    #[test]
    fn a_basic_parse() {
        assert_eq!(
            action_for_mail(b"Subject: test\r\n\r\n").unwrap(),
            MailingListAction::Reject
        );
    }

    #[test]
    fn a_subscribe_command() {
        assert_eq!(
            action_for_mail(b"From: test@example.org\r\nSubject: subsCRIbe     \r\n\r\n").unwrap(),
            MailingListAction::Subscribe("test@example.org".to_string())
        );
    }

    #[test]
    fn an_unsubscribe_command() {
        assert_eq!(
            action_for_mail(b"From: test@example.org\r\nSubject: UNSUbscribe\r\n\r\n").unwrap(),
            MailingListAction::Unsubscribe("test@example.org".to_string())
        );
    }

    #[test]
    fn a_message_to_pass_through() {
        let message = b"From: test@example.org\r\nSubject: A message to the list\r\n\r\n";

        assert_eq!(
            action_for_mail(message).unwrap(),
            MailingListAction::Message(message)
        );
    }

    #[test]
    fn a_bad_email() {
        if let MailParseError::Generic(msg) = action_for_mail(b"bad input\r\n").unwrap_err() {
            assert_eq!(msg, "Unexpected newline in header key")
        } else {
            panic!()
        }
    }
}
