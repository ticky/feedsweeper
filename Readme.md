# FeedSweeper

🧹 Automatically tidy up your RSS feed backlog.

## About

FeedSweeper is a tool for [Feedbin](https://feedbin.com) intended to help keep your unread list clean. In specific, I wanted a more social media-like vibe for some of the higher frequency news feeds I follow, giving them a timeout before they're marked as read automatically. This is my solution.

## Installation

### Easy mode (macOS or Linux)

```sh
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/ticky/feedsweeper/releases/latest/download/feedsweeper-installer.sh | sh
```

### With cargo

```sh
cargo install feedsweeper
```

### From source

Clone this repository and run `cargo install --path .` inside it.

## Usage

To run FeedSweeper, you should specify the `FEEDBIN_USERNAME` and `FEEDBIN_PASSWORD` environment variables. Feedbin sadly doesn't have a supported method to give applications API keys or individual passwords, so this is your real password.

### On Linux with a `systemd` timer

For example, if your system uses `systemd` you may wish to create a systemd unit file, such as at `~/.config/systemd/user/feedsweeper.service`:

```ini
[Unit]
Description=Tidy up your RSS feed backlog

[Service]
ExecStart=%h/.cargo/bin/feedsweeper --tagged "High Frequency" --max-age 1w
```

Where "High Frequency" is the name of a Feedbin tag you've prepared for feeds you wish for this to apply to.

And a matching timer file at `~/.config/systemd/user/feedsweeper.timer`:

```ini
[Unit]
Description=Automatically tidy up your RSS feed backlog

[Timer]
OnCalendar=hourly
Persistent=true

[Install]
WantedBy=timers.target
```

Then you can set the necessary environment variables by running `systemctl edit --user feedsweeper`, which will open an editor. You should enter the variables like this:

```ini
[Service]
Environment=FEEDBIN_USERNAME="<your feedbin username>"
Environment=FEEDBIN_PASSWORD="<your feedbin password>"
```

Finally, run `systemctl start --user feedsweeper.timer` to enable the timer.
