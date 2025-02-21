# Google Scholar rss feed generator

Generates an RSS feed from a user's scientific publications, parses data from Google Scholar.

## How to user

1. Launch the binary
2. Access `http://localhost:3005/?username=Julien%20Caposiena`
3. Change the author name to the desired person

You may want to specify the serving address or port, simply put it as the first argument like so:
```sh
cargo run "<ADDRESS>:<PORT>"
```

Default port is 3005.
