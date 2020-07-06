# Contributing

So if you contribute to this software, I'd love to pay you — probably not a
lot as this is a labor of love and is extremely unlikely to employ someone to
work on it. I don't actually want it to. This is small software. However,
paying for contributions is fair, and opens contribution up to a _different_
set of people than open source softare does. It also clarifies that
contributions are work for hire, and lets the copyright of the software remain
simple.

Philosophically, I want this project to reflect that I value sustainability. If
this software can be installed once and not changed for a decade, and stay
running securely, I consider that a success. It's built for a unix user, to
interface with unix mail systems. I'd support other options if someone wants to
write them, but as I've run a small community server on successive linux
distributions for 25 years, I'd like it to fit into that ecosystem. If you have
to run kubernetes or docker to have a little community, you've maybe entailed
too much complexity. I don't favor the status quo particularly hard, but things
that stay working and don't need much care and feeding make me happy.

This code is reasonably careful, but it's also started as my first Rust code.
Why is it how it is? Either because I have some philosophical biases toward
removing runtime dependencies — why use MySQL if a plain text file will do? —
or because I got it working that way and I didn't change it. I have no
particularly strong opinions about the code itself. Take that to mean that
there will be no snobbery about code someone wishes to propose to contribute.
If you think your code is bad, well, my code is bad too. Maybe if we work
together we can make it all less bad.

I'm open to contributions other than the things in the future work section of
[the README file](README.md), but those are things I can think of now that have
some relatively obvious value.

## Technical approach

I've been testing things on the command line as I work.

You can test the subscribe command with `cargo run test-data/test.list <
test-data/subscribe.mail`

You can test the unsubscribe command with `cargo run test-data/test.list <
test-data/unsubscribe.mail`

You can test sending a message to the list with `cargo run test-data/test.list
< test-data/test.mail` — on a Mac laptop, you can see the bounce message with
the `mail` command in your terminal. Old school unix stuff, but it works.

You can test invalid input with `cargo run test-data/test.list <
test-data/bad.mail`

And as always, `cargo test` runs the unit tests.


