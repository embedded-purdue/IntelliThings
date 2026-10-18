# Scheduled pipeline

## Background pipeline

`ai/main.py` collects fresh context, requests recommendations from OpenRouter,
optionally executes one configured MCP turn-on action, and updates the dashboard
summary, room-specific MQTT displays, and shared thinking status. The systemd user timer runs the full pipeline after user-manager startup
and every five minutes. Each run opens fresh MCP connections and loads the root
`.env`, so credential changes take effect on the next run.

Runtime files default to `~/.local/state/intellithings/mcp/` (or
`$XDG_STATE_HOME/intellithings/mcp/`). See the
[output file table](../../docs/inference.md#outputs-and-failures) for snapshots, model
responses, discovery, actions, and stage statuses. Check statuses and timestamps
before treating a saved result as current.

Install on a Pi where the repository is at `~/IntelliThings` and the virtualenv
and `.env` have been set up as in [local pipeline setup](../../README.md#current-local-python-pipeline):

```bash
mkdir -p ~/.config/systemd/user
cp ai/deployment/systemd/intellithings-context.* ~/.config/systemd/user/
loginctl enable-linger "$USER"
systemctl --user daemon-reload
systemctl --user enable --now intellithings-context.timer
systemctl --user start intellithings-context.service
```

Lingering starts the user's service manager at boot without an interactive login.
For a different checkout location, edit `WorkingDirectory` and `ExecStart` in the
installed service. Home Assistant may still be starting on the first attempt;
failed requests are retried on the next five-minute tick. Each run has a 60-second
collection budget, a 45-second inference budget (including the start status),
parallel dashboard and MQTT publication with 10-second deadlines, a 2-second
stop-status deadline, and a 120-second systemd limit. The startup status reset
runs concurrently with collection. Timer runs cannot overlap. After updating
an installed unit, copy it again and run `systemctl --user daemon-reload`.

```bash
systemctl --user list-timers intellithings-context.timer
journalctl --user -u intellithings-context.service -n 30 --no-pager
cat ~/.local/state/intellithings/mcp/latest.json
# Run a full cycle now:
systemctl --user start intellithings-context.service
# Stop scheduled pipeline runs:
systemctl --user disable --now intellithings-context.timer
```

