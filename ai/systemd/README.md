# Scheduled pipeline

## Background pipeline

`ai/main.py` fetches a single read-only live-context snapshot, then requests
recommendations from OpenRouter. It can execute one configured MCP turn-on action per cycle. The systemd
user timer runs it at startup and every five minutes. Each run opens a fresh MCP
session and loads the root `.env`, so token changes take effect on the next run.

The default output directory is `~/.local/state/intellithings/mcp/` (or
`$XDG_STATE_HOME/intellithings/mcp/`):

- `latest.json`: last successful snapshot, including UTC `fetched_at`, readable
  `context`, and the full `mcp_result`.
- `status.json`: whether the latest attempt succeeded, with its timestamp and any
  connection error. A failed fetch leaves the last successful snapshot intact.

Files are replaced atomically and readable only by the owner. An LLM consumer can
read `context` from `latest.json`; check `fetched_at` and `status.json` to avoid
treating stale data as current. This collects Assist-exposed context only; it does
not fetch shopping list contents. Inference sends the fresh context to OpenRouter
and saves `inference.json` with the input timestamp and model ID;
`inference_status.json` tracks inference errors separately. Configure the model,
API key, and editable prompt as described in [OpenRouter inference](../llm/README.md).

Install on a Pi where the repository is at `~/IntelliThings` and the virtualenv
and `.env` have been set up as in [the AI README](../README.md):

```bash
mkdir -p ~/.config/systemd/user
cp ai/systemd/intellithings-context.* ~/.config/systemd/user/
loginctl enable-linger "$USER"
systemctl --user daemon-reload
systemctl --user enable --now intellithings-context.timer
systemctl --user start intellithings-context.service
```

Lingering starts the user's service manager at boot without an interactive login.
For a different checkout location, edit `WorkingDirectory` and `ExecStart` in the
installed service. Home Assistant may still be starting on the first attempt;
failed requests are retried on the next five-minute tick. Each run has a 60-second
collection budget, a 45-second inference budget, and a 120-second systemd limit. Timer runs cannot overlap. After updating an existing installation, copy the
service file again and run `systemctl --user daemon-reload`.

```bash
systemctl --user list-timers intellithings-context.timer
journalctl --user -u intellithings-context.service -n 30 --no-pager
cat ~/.local/state/intellithings/mcp/latest.json
# Fetch now:
systemctl --user start intellithings-context.service
# Stop scheduled collection:
systemctl --user disable --now intellithings-context.timer
```

