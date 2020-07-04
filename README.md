# mulm, a tiny mailing list manager

## Features

- in-band commands: we honor unsubscribe and subscribe sent to list address.
- resumable sending if interrupted
- archives all messages to a group of mboxes

## Internal Design

### Receive process

- if the subject is a command, execute it
- otherwise:
  - write message `{sequenceNo]` to mbox `{mailinglist}.{mailingListArchiveEpoch}.mbox`
  - start send process of `{sequenceNo}` from "{mailingList}.{mailingListArchiveEpoch}.mbox`

### Send process

- open `{mailingList}.{mailingListArchiveEpoch}.{sequenceNo}.record` as the record file
- Lock the record file
- open `{mailingList}.subscribers` as the subscribers file
- Read the subscribers file and record file one line at a time, in tandem FIXME
- Read the record file FIXME
- For each subscriber; queue the outgoing message, and append `{subscriberEmail}\t{status}\n` to the record file
