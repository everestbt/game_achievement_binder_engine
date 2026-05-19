# Game Achievement Binder Engine (G.A.B.E)

GABE is a tool for tracking your games for completion and your progress through them.

To use you will need to get a Steam API key, [instructions can be found here](https://steamcommunity.com/dev).
You will also need your steam id which can be found following the [instructions here](https://help.steampowered.com/en/faqs/view/2816-BE67-5B69-0FEC).

The program is written in rust so you will need [Rust](https://rust-lang.org/tools/install/) installed to use it. There is no executable shared anywhere. 

You will need cargo to install it, my suggestion is to clone the repo and then install the parts you want:

CLI:
```bash
cargo install --path ./bin/cli
STEAM_API_KEY=<KEY> steam-rand --help
```

UI:
```bash
cargo install --path ./bin/iced-ui
STEAM_API_KEY=<KEY> gabe-ui
```

Replacing `<KEY> <STEAM-ID>` with your own credentials. 
It will save your steam id after first use, you can replace it by re-running the CLI with a new steam id. Note this may cause errors in the goals as it is only designed for one user, so safer to wipe the database and start again.

The Steam Web API can be flaky, there is no automatic retry, just run it again.

Some calls use a cache, so the first time may be slow as it populates with your data.