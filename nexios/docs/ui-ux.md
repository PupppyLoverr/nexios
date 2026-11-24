# UI/UX extraction: macOS 27 "Golden Gate" via the hosted desktop

Observed from upstream's screenshots (`../../docs/images/`) and from what the
hosted system is *structurally* — written for the Nexios experiment and for
any shell built alongside it. **We borrow principles, never assets** (see
Nexios rule 4): no Apple icons, fonts, wallpapers, or trademarked names ship
anywhere in the tree.

## What the hosted desktop shows

From `docs/images/desktop.jpg` — real macOS 27.2.1 running under XNU-on-Linux,
rendered on CPU:

- **Menu bar**: ~24 px translucent strip. Left: logo glyph + *bold* focused-app
  name + the app's own menus, flush left, tight tracking. Right: small status
  cluster (screen sharing, Spotlight, control toggle) + full date+time.
  Everything in one line — no second row, ever.
- **Dock**: floating glass capsule, ~28 px corner radius, generous icon spacing
  (~10 px), a hairline divider separating the folder/trash zone at the right.
  Icons are colourful, glossy, squircle-ish — high contrast against any
  wallpaper because the glass behind them is bright and blurred.
- **Widgets**: stacked glass cards top-left under the bar (calendar, clock, weather, photos, music),
  ~16 px radius, semi-transparent, content-first with minimal chrome.
- **Wallpaper**: full-bleed photography, warm gradients — the bar and dock stay
  legible purely through translucency, not through opaque bands.
- **Windows** (`macos-and-linux.jpg`): traffic-light cluster top-left,
  large radius, thick drop shadows, title bar integrated with toolbar.

From `docs/images/dgx-spark.jpg` — macOS Screen Sharing into the hosted
system over VNC, and `Terminal.app` next to the DGX host's own terminal:
two kernels' userspaces, side by side on one screen. *This image is the
whole Nexios thesis.*

## Principles to take (legal: ideas are not copyrightable)

1. **One thin bar owns global state.** App menus move to the focused app; the
   bar never grows. (Solyra panel already does this — validated.)
2. **Translucency is the legibility strategy.** Bar/dock/cards are glass over
   live wallpaper — identity comes from blur + tint, not opaque chrome. Needs
   real backdrop blur (we have dual-kawase in comp).
3. **A floating dock beats a full-width taskbar for focus** — but keep both;
   taskbar density wins for some users. macOS 27 keeps ~62 px card height and
   ~10 px gaps at 1280×800; our Perihelion II dock (62 px card, r22, 9 px
   gaps) is already in that family.
4. **Widgets as quiet glass cards**, not a separate dashboard — ambient info
   lives on the desktop edge, calendar/weather-style.
5. **Colourful, dimensional icons** — macOS 27's icon set is glossy and
   saturated again (post-Liquid-Glass), not flat. Our Perihelion II 2.5D set
   was the right call.
6. **Honest affordances**: every transient surface (Spotlight, Control Centre,
   notifications) is a rounded card *with elevation* — shadow first, rim
   second. Now covered by `material::paint` shadows in the panel.

## What macOS 27 does that we don't (yet)

- **A real widget layer** on the desktop — now built (`nexios-widgets`:
  calendar + clock + weather + photos + music glass cards (music tracks the live MPRIS player)). Remaining: editing/adding
  widgets, more card kinds.
- **Dynamic-island-as-notification-model**: the notch expands for live
  activity. We have the island; theirs proves the pattern scales to a whole
  OS's worth of glanceable state.
- **Two-kernel desktop**: `macos-and-linux.jpg` is the target UX for the
  Space model — hosted personality windows *on the same desktop*, not in a
  VM window. That is a UI problem (compositing the hosted framebuffer as a
  workspace/surface) as much as a kernel problem.

## What to avoid copying

- The Apple logo glyph,  glyph, traffic-light window buttons, SF Symbols,
  Apple wallpapers, "Finder/Dock/Spotlight" naming — trademarked/copyrighted.
- macOS's exact layout *metrics-as-clone*: being "inspired by" is fine; a
  pixel-identical menu bar + dock + icons could read as trade-dress. Solyra's
  iris identity is the differentiator.

## Notification centre

Clicking the bar's clock/date (or `nexios-cmd toggle-notifs`) opens a
right-edge drawer, like macOS: "Notification Centre", today's date, a
Clear All pill, a Now Playing card at the top when media is playing
(transport buttons drive the real player over MPRIS), and notification
history newest-first: each card carries the sending app's icon and name
plus a relative timestamp ("just now" / "1m ago"), like the real centre's
per-app cards. History is capped at 30 and
deduped; Focus mode still hides the island banner but the centre keeps
recording. `nexios-cmd clear-notifs` empties it; `nexios-cmd notifs`
lists the backlog for scripted checks.

## Lock screen

`nexios-cmd lock` (or the Nexios menu) spawns `nexios-panel lock` — a
session lock that verifies the user's password through PAM. The distro
image must ship `shell/nexios-lock.pam` at `/etc/pam.d/nexios-lock`;
without that service every password fails (PAM denies by default).
