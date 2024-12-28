# bo

CLI bookmark manager.

## Usage

```console
$ bo --help
Usage: bo [OPTIONS] [ARGS]...

Arguments:
  [ARGS]...  The name of the bookmark to open. If not provided, a list of available bookmarks will be shown. If multiple strings are
             given, the first one is used as the bookmark name, and the remaining strings are used as arguments for the bookmark,
             which will be replaced in the URL `{query}`

Options:
  -c, --config <CONFIG>  Path to the configuration file. Defaults to $XDG_CONFIG_HOME/bo/config.toml
  -h, --help             Print help
  -V, --version          Print version
```

i.e.

```console
$ bo sc
```

or,

```console
$ bo drive meeting minutes
```

or, run it without an argument to select a channel from a list interactively:

```console
$ so
```

## Configuration

Place your configuration file at `$XDG_CONFIG_HOME/so/config.toml` or provide the path using the `--config` option.

```toml
# You need to explicitly set the name of the default browser,
# such as "Google Chrome" or "Firefox", instead of relying on
# the system default.
default_browser = "firefox"

# Optional aliases for bookmarks
[aliases]
gh = "github"
mf = "moneyforward"
sc = "shortcut"
d = "drive"

[bookmarks]
github = { url = "https://github.com/0x6b" }
moneyforward = { url = "https://attendance.moneyforward.com/admin/workflow_requests/waiting" }

# Alternate browser can be specified with the `browser` key
shortcut = { url = "https://app.shortcut.com/dashboard", browser = "Google Chrome" }

# The `{query}` placeholder will be replaced with the rest of the command line
# arguments after the bookmark name
drive = { url = "https://drive.google.com/drive/search?q={query}", browser = "Google Chrome" }
```

## License

MIT. See [LICENSE](LICENSE) for details.
