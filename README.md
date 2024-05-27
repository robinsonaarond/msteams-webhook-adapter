# msteams-webhook-adapter
Small utility to adapt several cloud service notifications to the MS Teams Webhook standard.

MS Teams has been more difficult to integrate with as a notification platform than other options (like Slack).  As we switched over to Teams we found that some of our external services didn't have an integration, or didn't have a working integration, with Teams.  This utility was built to accomodate our specific needs, and so it covers:
 - Dead Man's Snitch [site](https://deadmanssnitch.com/)
 - Buildkite [site](https://buildkite.com)

The DMS integration was the start of the idea, as they supported sending out a json data object to a generic webhook to an endpoint of your choosing, but because it wasn't formatted the way Teams wanted it, it just wouldn't work.  All I had to do was pull in that object, re-map the data into the `'{"text":"Snitch <blah> is reporting"}'` format that Teams wanted to see.  With the Buildkite integration I took advantage of the somewhat complex data object available for Teams Adaptive Cards.

I also added support for shortened URLs, as the standard webhook for MS teams is huge, and doesn't have a human-readable part to let you know what channel it will even go to.  So you can have a webhooks.json object up in AWS with mappings from the large webhook to a smaller, human readable webhook.  This one requires the apiKey to be set in the query string parameters, however, to prevent it from being easily abused.

This project was originally written in Python, but as a learning opportunity I rewrote it in Rust.  In adaptive_card.rs you can see it took quite a few structs to properly map the needed object, an example of how much more boilerplate you end up with in Rust; in Python the code looks pretty much like JSON with a few template variables, way more readable.  On the other hand, the rust code literally worked the first time, once I got it to compile.
