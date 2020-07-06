# mulm, a tiny mailing list manager

Free for noncommercial use. For commercial use [buy a
license](https://licensezero.com/offers/556b44a4-315f-434b-972f-0dfb485f32ec)

## Quick Start

Install this package with `cargo install mulm --root /usr/local`. Then

`mkdir /path/to/list/store.list`

Create `/path/to/list/store.list/config.toml` with the content

```
[config]
name = "A List"
slug = "listname"
```

In your MTA's aliases file, add `listname: |/usr/local/bin/mulm /path/to/list/store.list`

Send mail to `listname@yourdomain` to post; use the subject line "subscribe" to
subscribe, and "unsubscribe" to remove yourself from the list.

## Features

- in-band commands: we honor unsubscribe and subscribe sent to list address.
- resumable sending if interrupted
- archives all messages to a group of mboxes

## Current Status

No privacy, it works but it's super simple. It's best used for groups that
trust each other.

## Configuration

- `config.name` — a user-oriented title for the list. A string, required.
- `config.slug` — an identifier for the list, ideally the local part of the
  address. A string, required.
- `config.open_posting` — whether or not posters must be a member of the list.
  Boolean, optional.

## Internal Design

The design of this software is meant for small system use. Each list is stored
in a Maildir, making message archives pretty reliable in the face of failure.

Added to the maildir are several control files and the queue directory.

`config.toml` is the list configuration file.

The queue of messages being relayed to the MTA is resumable (though this does
use file locking, unlike plain maildir, so old school NFS is a hazard there —
duplicate delivery is possible if file locking does not work).

The queue is integrated into the Maildir in the `queue` directory. There are
three files for each queue entry: a position file (`{id}.pos`) tracking the
position in the destination list sent so far and where to stop for this
message, a destination list (`{id}.dest`) with a recipient per line, and the
message (`{id}.msg`). All three are removed when an entry is sent, and the
message is locked while sending. The destination list can be a hard link to the
current subscription list, and the message is a hard link to the message file
as delivered to the maildir.

The recipient lists are appended to for subscriptions, and rewritten as a new
file for unsubscribes.

## Licensing

This code is licensed under [License Zero
Prosperity](https://prosperitylicense.com/). It is not free software! However,
it is available for noncommercial use without payment, and contributions are
welcome. See below.

## Future work

- More complete unit tests
- Perhaps rewrite to use mailparse's types internally rather than a `&[u8]`
  for the message.
- Adding the list slug to the subject line
- Add list management headers
- Parsing HTML parts and reducing them to simple markup only
- Censoring originating headers for privacy
- Masking email addresses entirely
- Welcome messages
- VERP for bounce detection
  - It still has to work with a single line in `/etc/mail/aliases` though so
    perhaps it's time to move to `+` addresses rather than anything more complex.
- A moderation queue
  - Joins
  - Posting
- A web interface
  - Should be a separate project really.
- A command line for subscribing and unsubscribing people

If you'd like to work on any of these, feel free. Talk to me and we can discuss
payment. You don't need to be super experienced — this is my first Rust
project, and a learning one. Some of those things above are pretty simple if
you want to take a stab at it.
