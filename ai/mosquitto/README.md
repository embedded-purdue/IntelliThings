# Mosquitto MQTT broker

Mosquitto runs in its own Docker container alongside Home Assistant Container on
the Raspberry Pi. Docker Engine and the Docker Compose plugin are required.
The broker requires a username and password, stores persistent state in a named
Docker volume, and restarts automatically unless explicitly stopped.

## First-time setup

From the repository root:

```bash
cd ai/mosquitto
docker compose pull
docker run --rm -it --user 0 --entrypoint sh \
  -v "$PWD/config:/mosquitto/config" eclipse-mosquitto:2 \
  -c 'test ! -e /mosquitto/config/passwords && mosquitto_passwd -c /mosquitto/config/passwords homeassistant && chown mosquitto:mosquitto /mosquitto/config/passwords && chmod 600 /mosquitto/config/passwords'
docker compose up -d
docker compose ps
docker compose logs --tail 50 mosquitto
```

The password command prompts interactively, keeping the password out of shell
history. Save it in your password manager. The command refuses to overwrite an
existing password file. Password hashes and local credentials are ignored by Git;
do not commit them. The root `.env.example` documents MQTT connection variables
for clients; this broker does not read those variables.

## Connect clients

In Home Assistant, open **Settings → Devices & services → Add integration → MQTT**.
Use `127.0.0.1` as the broker when Home Assistant uses host networking on the same
Pi, port `1883`, username `homeassistant`, and the password created above.

Sensor nodes and other LAN clients use the Pi's LAN address instead of
`127.0.0.1`. For Home Assistant on another machine or a bridge network, use the
Pi's reachable LAN address as well.

Port 1883 is published on the host interfaces and uses unencrypted MQTT. This
configuration is intended for a trusted LAN; remote access needs a VPN or TLS.
Anonymous connections are disabled. Authenticated users can access all topics.

## Emulator readings in Home Assistant

The emulator publishes JSON to `intellithings/emulator/state`. Register its
Kitchen, Bedroom, and Living Room nodes using MQTT discovery (enabled by default
in HA). First connect HA's MQTT integration to this broker, then from the repository
root, using the virtualenv and `.env` described in [the MCP setup](../probes/README.md):

```bash
# Preview configuration without publishing:
.venv/bin/python ai/mosquitto/register_emulator.py
# Create or update the discovery configurations:
.venv/bin/python ai/mosquitto/register_emulator.py --publish
```

This uses HA's `mqtt.publish` service with the HA access token, so it needs no
additional broker credentials. It creates three devices with ten entities each:
temperature, humidity, VOC index, CO2, illuminance, distance, PM1, PM2.5, PM10, and
presence. Templates select nodes by ID, regardless of their order in the JSON.
Discovery configurations are retained; repeating registration updates the same
entities. No additional background process is needed: HA subscribes directly to
the state topic. Entities expire after 60 seconds without updates and also check
each node's connection and publication age. Retained state messages can temporarily
replay old readings on HA restart; publish state without retention when relying
on this expiry behavior.

Find the devices under **Settings → Devices & services → MQTT**. To include their
readings in MCP live context, expose the desired entities to Assist under
**Settings → Voice assistants → Expose**.

## Broker operations

Run these commands from `ai/mosquitto/`:

```bash
docker compose logs -f mosquitto
docker compose restart mosquitto
docker compose down
docker compose up -d
```

`down` preserves the data volume. `down -v` deletes it and its persisted messages.
Back up the volume and the local password file when moving the broker.

To change a password or add another user, replace `homeassistant` below as needed:

```bash
docker run --rm -it --user 0 --entrypoint sh \
  -v "$PWD/config:/mosquitto/config" eclipse-mosquitto:2 \
  -c 'mosquitto_passwd /mosquitto/config/passwords homeassistant && chown mosquitto:mosquitto /mosquitto/config/passwords && chmod 600 /mosquitto/config/passwords'
docker compose restart mosquitto
```

## Existing project Pi

The initial deployment lives at `/home/intellithings/mosquitto/`, with its own
`compose.yaml`, `config/`, and private `credentials.txt`. Its data volume is
`mosquitto_mosquitto_data`. The repository contains the matching configuration;
adding these files does not relocate the running container.

Use the existing deployment directory to manage that instance until deliberately
migrating it. Do not start a second copy from this directory while it is running:
both use container name `mosquitto` and port `1883`.

Reference: [Mosquitto authentication documentation](https://mosquitto.org/documentation/authentication-methods/).
