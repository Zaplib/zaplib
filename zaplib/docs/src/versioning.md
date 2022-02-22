# Versioning

We don't release proper versions yet. Instead, you should pick a git commit and pin to that. In your `Cargo.toml`:

```toml
[dependencies]
zaplib = { git = "https://github.com/Zaplib/zaplib", rev="c015a1e" }
```

And in `package.json` (a version like this is pushed automatically to [npm](https://www.npmjs.com/) on every commit):

```js
"dependencies": {
    "zaplib": "0.0.0-c015a1e"
}
```

When upgrading, be sure to update both `Cargo.toml` and `package.json`, and be sure to follow along in [Slack](/slack.html) to learn about API changes.
