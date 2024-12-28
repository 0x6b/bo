# bo

CLI bookmark manager.

## Usage

```console
$ bo --help
Usage: bo [OPTIONS] [NAME]

Arguments:
  [NAME]  The name of the bookmark to open. If not provided, select from a list
          of available bookmarks

Options:
  -c, --config <CONFIG>  Path to the configuration file. Defaults to
                         $XDG_CONFIG_HOME/bo/config.toml
  -h, --help             Print help
  -V, --version          Print version
```

i.e.

```console
$ bo github
```

or, run it without an argument to select a channel from a list interactively:

```console
$ so
```

## Configuration

Place your configuration file at `$XDG_CONFIG_HOME/so/config.toml` or provide the path using the `--config` option.

```toml
# You need to explicitly set the name of the default browser, such as "Google Chrome" or "Firefox",
# instead of relying on the system default.
default_browser = "firefox"

[aliases]
mf = "moneyforward"

[bookmarks]
moneyforward = { url = "https://attendance.moneyforward.com/admin/workflow_requests/waiting" }

# Alternate browser can be specified with the `browser` key
shortcut = { url = "https://app.shortcut.com/dashboard", browser = "Google Chrome" }
```

## License

MIT. See [LICENSE](LICENSE) for details.
