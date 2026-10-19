# 3D floor plan for the HA dashboard: plan

*AI subteam (HA side: interface & dashboard) · drafted 2026-10-18 · status: proposal*

**Goal:** a 3D floor plan on the Home Assistant dashboard that shows **everything that happens in
the house, live**, whether a person, an automation or the AI agent caused it. That includes:
- **Lights** on/off, colour and brightness, and the **light falling on the room**.
- **Room temperature** as numbers and as a room colour.
- **Doors, windows, garage door, curtains and blinds** opening and closing.
- **Fans, AC, TV, purifier and other appliances** running.
- **Robot vacuum** running or docked; **car** in or out of the garage.
- **Who is in which room.**
- **Sun position** (day/night and the light direction).

**Due:** Project Guideline §9 schedules "Dashboard incl. 3D/floor-plan view" for the **Nov 1**
session and polish for **Nov 8**.

---

## 1. The key idea: build the HA side once, then pick a card

Every 3D floor-plan option for HA is a **small community project**, so any of them could break
or be abandoned. We protect ourselves by doing most of the work in plain Home Assistant config,
which every card can read:

1. **One HA area per room:** Kitchen, Bedroom, Living Room, Garage, Entry (plus whatever the
   layout needs).
2. **One entity per thing shown in 3D.** Most are **virtual devices**, which the Guideline
   already plans for the things we don't own (§2 below).
3. **The 3D card only *displays* those entities.** Switching cards later means re-binding them,
   not rebuilding the house.

Bonus: the **same virtual devices are what the AI agent controls through MCP**. When the agent
turns on the living-room fan, the 3D view shows it. That's the demo story.

---

## 2. Entities to create (tool-independent)

| Shown in 3D | Data source | How to create it in HA (no hardware needed) |
|---|---|---|
| Lights per room | Virtual | **Template light** backed by an `input_boolean` (+ `input_number` for brightness) |
| Curtains / blinds | Virtual | **Template cover** backed by an `input_number` (position 0–100) |
| Front door, garage door | Virtual | `input_boolean` → **template binary_sensor** (`device_class: door`). The garage door can be a template cover (`device_class: garage`) |
| Fan, AC/thermostat, purifier, humidifier, TV, smart plug | Virtual | Template fan / template switch / `climate` via a virtual thermostat, each backed by helpers |
| Robot vacuum | Virtual | **Template vacuum** backed by an `input_select` (docked / cleaning / returning), plus a script that "cleans" for N minutes, then returns to the dock |
| Car in garage | Virtual | `input_boolean` → template binary_sensor (`device_class: presence`) |
| Sun position | Built in | `sun.sun`: elevation and azimuth, already in HA |
| Presence per room | **Real data** | The nodes' MQTT `presence` (emulator `emu-*` nodes today) |
| Room temperature (number + colour) | **Real data** | The nodes' MQTT `temperature_c` |

- **Entity names belong in the contract.** Add the new names to the draft HA entities
  contract, `docs/interfaces/ha-entities.md`, first, then build. Don't invent names
  separately in the dashboard.
- **Expose the virtual devices to Assist** (and therefore MCP), so the agent can act on them.
- Keep all YAML in `ai/deployment/home_assistant/` (no secrets).

---

## 3. Candidate tools: what each one shows

> **⚠ Install constraint:** the project runs Home Assistant as **HA Container** (Docker),
> not HAOS (moving to HAOS stays an option if the 3D floor plan needs it). **HA add-ons don't exist on HA Container**, and
> HA-Floorplanner is an add-on; its dashboard card displays the add-on's view. So it **can't be
> installed as documented** on our Pi, unless its server also runs as a plain Docker container
> (unverified) or the team chooses HAOS. **floor3d-card** and **floorplan3d-card** install
> through HACS, which works on HA Container, so they become the main candidates. Check this
> first in the spike.

From each project's README (not tested yet):
- ✅ documented.
- ⚠️ partial, or only with a workaround.
- ❌ not mentioned.

| Feature | **HA-Floorplanner** (add-on + card) | **floor3d-card** | **floorplan3d-card** (jaxx2104 fork of istals) |
|---|---|---|---|
| **Building the house** | ✅ Draw in the browser (2D editor + live 3D), trace a scan, or **import AI-generated JSON**; 270+ furniture pieces + your own `.glb` | ⚠️ Model in **Sweet Home 3D** (free, 2D → 3D), export OBJ → convert to GLB, bind objects by name | ✅ Click room corners in the card; optional GLB with tagged parts |
| **Room temperature as a number** | ❌ Colour only (see heat map) | ✅ Text on a plane, or a room label showing an attribute | ✅ Value markers on the walls |
| **Room coloured by temperature** | ✅ **Built-in** temperature / humidity / **CO₂** colour maps | ⚠️ Map the value to bands (e.g. "hot"/"cool") with a template, then colour by band | ❌ |
| **Lights on/off, colour, brightness** | ✅ Real colour, dimmable | ✅ Uses colour and brightness | ✅ Colour and brightness |
| **Light cast onto the room** | ✅ Light pools that depend on the fixture type (a strip doesn't light a room like a ceiling lamp) | ✅ **Real 3D lights** (brightness, falloff, range, optional shadows), the most realistic | ✅ Glow on the floor; lights the room and facade around it |
| **Doors open/close** | ✅ Open on the contact sensor; real openings in walls | ✅ **Swing or slide animation** with hinge, direction and angle | ❌ Icon only |
| **Windows** | ✅ Tilt open, a sensor per pane | ✅ Swing or slide | ❌ Icon only |
| **Garage door** | ❌ | ⚠️ Workaround: a `cover` panel that rolls up, or show/hide the panel | ⚠️ Not mentioned; a tagged `cover` panel might work |
| **Roller shutters / blinds** | ✅ A roller shutter per window with its own cover entity | ⚠️ `cover` rolls up/down only ("experimental") | ⚠️ Not described |
| **Curtains opening sideways** | ❌ | ⚠️ Workaround: treat the panels as sliding doors (open/closed only) | ✅ **Gathers to one side, follows the position percentage** |
| **Fans spinning** | ⚠️ Fan controls (speed, swing); spinning not described | ✅ Spins while on (speed isn't tied to the fan percentage) | ❌ Toggle only |
| **AC / heating** | ⚠️ Controls (target, modes); visual not described | ⚠️ Generic colour / show / text | ✅ **Glows warm or cool while heating or cooling**, shows its temperature |
| **TV / media** | ⚠️ Controls (play, volume, source), TV backlight; screen not described | ✅ Text or colour on the screen plane | ⚠️ Shows the media picture |
| **Plugs, purifier, other appliances** | ✅ Power cables with **animated flow** between power devices | ✅ Generic: colour, show/hide or spin any named object | ⚠️ Marker + status dot |
| **Presence / motion** | ✅ **Glowing person figure** while present; dot on the room label | ⚠️ Show/hide a figure, or recolour the room | ⚠️ Person pictures; motion as an icon |
| **Cameras** | ✅ **Field of view drawn on the floor, red when there's movement** | ✅ Double-click shows the camera image | ❌ |
| **Alarms (smoke, water, …)** | ⚠️ Panel controls in the card | ⚠️ Generic colour | ✅ Coloured diamonds while active; problems in yellow |
| **Robot vacuum** | ⚠️ Controls only (clean, pause, dock) | ⚠️ Spin or show/hide; can't move it | ⚠️ Live position documented for **mowers** ("any entity that reports a position"); a fake vacuum with x/y might move |
| **Car in/out** | ⚠️ Car lights follow an entity; show/hide not stated | ✅ Show/hide any object | ⚠️ Objects bind to entities; show/hide not stated |
| **Sun position** | ❌ Only "dim at night" | ✅ Sky and sun from `sun.sun` + **shadows** | ✅ Sun and shadows from `sun.sun`, plus a time-of-day slider |
| **Tap to control** | ✅ Tap a device; **tap a room for the whole room** | ⚠️ Double-click runs a service; **touch on iOS/Android "not fully supported"** | ✅ Tap to toggle, hold for a popup |
| **History** | ✅ **Replay past changes in 3D** ("which light went on, which door opened") | ❌ | ⚠️ Links to HA's history/logbook |
| **Bind any custom object to any entity** | ❌ Not documented | ✅ **Most flexible:** light, colour, show/hide, rotate, text, door, cover on any named object | ✅ Tag parts of a GLB |
| **Weak tablets** | ✅ Low-power mode | ⚠️ Turn shadows off | ✅ `lights: off` (glow only) |
| **Maintenance** | **Active** (v3.75, Oct 2026), one developer, ~9 stars, MIT | **Last commit April 2023**; HACS default | New (2026), ~0 stars, MIT |
| **Needs** | HAOS or Supervised (it's an add-on); **check aarch64 support** | HACS | HACS integration |

**What the matrix says:**
- **HA-Floorplanner** is the best "live house": easiest to build, temperature/CO₂ colouring,
  person figures, camera cones, energy flow, 3D history replay. **Gaps:** temperature numbers,
  sun position, curtains, garage door.
- **floor3d-card** gives the most realistic 3D (real lights, door animation, sun and shadows) and
  can bind anything. **Risks:** unmaintained since 2023, and **weak touch support**, which matters
  for a tablet demo.
- **floorplan3d-card (fork)** has the only documented position-following curtains, AC glow and
  temperature numbers, and is the most promising for a moving vacuum. **Risk:** the newest, with
  ~0 users.

**Proposal:**
- On HA Container, start the spike (§5) with the HACS options: **floor3d-card** (proven; doors,
  sun, shadows) and the **floorplan3d fork** (position-following curtains, temperature numbers).
- Keep **HA-Floorplanner** as the option if the team decides to try HAOS (or its server turns
  out to run as a plain Docker container). Its missing displays (temperature numbers, sun) would
  come from §3b's side cards.
- Test the curtain and garage-door workarounds early.

**Already considered:**
- **Pre-rendered images:** Sweet Home 3D renders shown in a `picture-elements` card. Looks great,
  but each render takes ~20–25 min and there's no real 3D or animation.
- **`ha-floorplan`:** a 2D SVG floor plan.

---

## 3b. Showing *who* made each change (person, automation or AI)

None of the 3D cards shows who caused a change. Home Assistant records it, though: every state
change carries a **context** with **one** of these:
- A **user** (someone used the app or dashboard).
- A **parent automation or script.**
- **Neither** (the device itself changed, e.g. a wall switch).

The **Logbook** shows this as "turned on by *user*" or "triggered by *automation*".

1. **Give the AI agent its own HA user** (e.g. "IntelliThings AI") and connect the MCP client as
   that user. Every agent action then appears in the Logbook **attributed to the AI**, separate
   from people and automations.
2. **Put two built-in cards next to the 3D view:**
   - A **Logbook card** limited to the rooms' entities: a live feed of "what happened and who did
     it".
   - A **Markdown card** with the agent's latest decision and explanation, the same text sent to
     the node displays.
3. **Optional highlight in 3D:** a helper holding "the entity the AI changed last" can flash or
   colour that object for a few seconds (in floor3d-card through colour conditions).

---

## 4. Where AI helps (and where it doesn't)

**Good uses:**
- **House layout from a description.** Write the rooms and approximate sizes, then have an AI
  produce HA-Floorplanner's import JSON (check its format in the README first), import it, and
  fix details in the editor.
- **Repetitive YAML:** the template entities from §2; the card's entity bindings (floor3d-card
  bindings are long and repetitive, which AI does well from a list of object names); temperature
  colour bands.
- **Debugging card config** from error messages.

**Poor uses:**
- **Generating a whole 3D house as a mesh.** Results are unreliable. Use a 2D-drawing tool
  instead (the HA-Floorplanner editor or Sweet Home 3D); both turn a 2D plan into 3D
  automatically.
- **Props** (car, vacuum, sofa): download **free low-poly GLB models** (e.g. CC0 libraries)
  instead of generating them. Check each licence and note attribution.

---

## 5. Timeline

| When | What | Who |
|---|---|---|
| **By Oct 21** | **Team decision: the demo home layout.** Which rooms, and where the 3 nodes go (an open decision, Guideline §10). Proposal: a compact demo home with **kitchen, living room, bedroom** (the 3 node rooms, matching the emulator) **+ garage + entry**, with one window with blinds per room | Whole team (PMs coordinate) |
| **By Oct 21** | Entity names agreed in the `docs/interfaces/` contract; areas + virtual entities created in HA; a dedicated HA user for the AI agent | AI team (HA side) |
| **Oct 21–25** | **Spike, 2 people, 2–3 days.** Install floor3d-card and/or the floorplan3d fork via HACS on the real HA Pi; build the 3 rooms; run the test cases below | AI team (HA side) |
| **Oct 25 session** | Pick the tool | AI subteam |
| **Oct 25 – Nov 1** | Full model + all bindings from §2 + the §3b side cards; show it at the **Nov 1** session with the emulator driving presence and temperature | AI team (HA side) |
| **Nov 1–8** | Polish: sun/shadows, colours, speed on the demo device, agent actions visible in 3D; swap in real node data as the rigs come online | AI team (HA side) |

**Spike test cases** (each should show up within ~1–2 s of the entity changing):
- [ ] A door swings open; a window tilts.
- [ ] The **garage door** rolls up (or the workaround works).
- [ ] A **blind at 50%**; a **sideways curtain** opening.
- [ ] A light turns on and **lights its area**; colour and brightness follow.
- [ ] **Room temperature** shown as a number *and* as a room colour.
- [ ] A fan running, the AC heating, the TV on.
- [ ] A person shown in the room the emulator says is occupied.
- [ ] The vacuum running vs. docked (moving between two points is a bonus).
- [ ] The car shown/hidden.
- [ ] Sun position changes the light (or a side card shows it).
- [ ] An AI action appears in the Logbook card **attributed to the AI user**.

**Also check:**
- [ ] Installs on our HA (HA Container on the Pi 5, aarch64).
- [ ] Runs smoothly on the **demo screen** (tablet or laptop), with touch working.
- [ ] Usable by a teammate with no 3D experience within an afternoon.
- [ ] The config can be exported and kept in Git.

---

## 6. Practical notes

- **Rendering happens in the viewer's browser** (WebGL / three.js), not on the Pi; the Pi only
  serves the files. Test on the actual demo device. Keep the 3D model small and low-poly, and
  turn shadows off if a tablet struggles.
- **"Real time" comes for free:** HA pushes state changes to the dashboard over its WebSocket
  connection.
- **Vacuum honesty:** a virtual vacuum doesn't really move. For the demo, show running vs.
  docked. An animated cleaning path is a stretch goal; it needs scripted x/y positions and a card
  that can use them.
- **Repo:** commit the *source* (the HA-Floorplanner JSON export or the Sweet Home 3D `.sh3d`)
  and the dashboard YAML under `ai/deployment/home_assistant/`. Exported models are re-generated output; ask in Discord
  before committing anything over ~10 MB (CONTRIBUTING.md).
- **Branch:** `ai/ha-dashboard` (CONTRIBUTING.md example), PR into `dev`.

---

## Sources

- HA-Floorplanner: https://github.com/BuRnEd4AiM/HA-Floorplanner
- floor3d-card: https://github.com/adizanni/floor3d-card (commit history: https://github.com/adizanni/floor3d-card/commits/master)
- floorplan3d-card: https://github.com/istals/floorplan3d-card · fork: https://github.com/jaxx2104/floorplan3d-card
- Sweet Home 3D render-time discussion: https://community.home-assistant.io/t/3d-floor-plan-alternatives-to-sweet-home-3d-tool-or-service-recommendations/915581
- Sweet Home 3D → picture-elements plugin (HACF): https://forum.hacf.fr/t/un-plugin-de-sweet-home-3d-pour-generer-des-floor-plan-sous-home-assistant/52261
