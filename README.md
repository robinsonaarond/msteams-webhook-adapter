# msteams-webhook-adapter
Small utility to adapt several cloud service notifications to the MS Teams Webhook standard.

MS Teams has been more difficult to integrate with as a notification platform than other options (like Slack).  As we switched over to Teams we found that some of our external services didn't have an integration, or didn't have a working integration, with Teams.  This utility was built to accomodate our specific needs, and so it covers:
 - Dead Man's Snitch [site](https://deadmanssnitch.com/)
 - Buildkite [site](https://buildkite.com)

With the Buildkite integration I took advantage of the somewhat complex data object available for Teams Adaptive Cards.  The DMS integration is a simple text response with an embedded link to the snitch.

This project was originally written in Python, but as a learning opportunity I rewrote it in Rust.  In adaptive_card.rs you can see it took quite a few structs to properly map the needed object, an example of how much more boilerplate you end up with in Rust.  On the other hand, it literally worked the first time I got it to compile.
