# Versioning

While we're in early alpha, we won't be releasing proper versions. You can require a specific version via git commit hash. For example:

### `Cargo.toml`

```toml
[dependencies]
zaplib = { git = "https://github.com/Zaplib/zaplib", rev="c015a1e" }
```

### `package.json`

```js
"dependencies": {
    "zaplib": "0.0.0-c015a1e"
}
```

## Warnings

1. It's very important that `Cargo.toml` and `package.json` point to the same commit hash.
2. While we're in alpha, please follow along in [Slack](/slack.html) to learn about API changes.
