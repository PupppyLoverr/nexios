# Mode map: macOS 27 "Golden Gate" → Solyra's modes

A surface-by-surface extraction for the modes Solyra already ships, so the
macOS reference lives in design decisions, not in asset copying. Principles
only (nexios rule 4). Verdicts reference the Perihelion II design
(`devin/perihelion-refresh`).

## Dock modes

| Solyra mode | macOS analogue | What macOS does better / take | Verdict |
|---|---|---|---|
| Floating (default) | The Dock | Divider before folder/trash zone; running-dot under apps; glossy colourful icons; magnification option | Already matches: we have all four (separator+trash, dots, 2.5D icons, magnify variant). At parity. |
| Minimal | Dock, autohide minimal | macOS has no equivalent — ours is a deliberate reduction | Keep; it's our density answer macOS lacks. |
| Taskbar | Windows 11 taskbar (not macOS) | Centred icons + start-left is the Windows pattern macOS doesn't do | Keep; it's the Windows half of the "best of both" brief. |

## Bar

macOS 27 menu bar ≈24 px translucent strip: logo + bold app name + menus |
status cluster + full date+time. Ours: 28 px glass bar, `nL` + bold app name +
menus | workspace pills | status + clock.

Take: **full "weekday day-month" date on wide bars** (macOS shows
`Sun 22 Nov 6:22 PM`; ours shows `Mon Nov 23 23:32` — ours is denser and fine;
the *idea* is the bar carries a real clock+calendar, already true).
At parity; ours also carries workspace pills macOS lacks.

## Materials

| Solyra material | macOS 27 analogue | Take |
|---|---|---|
| Glass | Vibrancy/NSVisualEffect (menu bar, dock) | Live blur + tint = legibility without opaque bands. Already at parity: our Glass slab uses a neutral white veil over dark bases (code cites the macOS dock) + dual-kawase backdrop blur. |
| Frost | Sidebar/material translucency | Already the middle material. |
| Solid | `Reduce transparency` accessibility mode | Correct to keep. |
| Clear | — | Ours alone; macOS has no fully-clear surface mode. |

New since 27 ("Liquid Glass" language): heavy translucency + refraction-looking
rims, brighter glass on dark. We implement rims (`round_rect_glass_rim`) and
elevation shadows; **do not chase animated refraction** — it's GPU cost for
marketing, and CPU-rendered fallback (which hosted systems use) can't afford it.

## Island / notifications

macOS uses the notch for Live Activities on devices that have one; on the
desktop it's just dead bar space. **Ours is already better positioned** — the
island is our invention space: now-playing, agent tasks, approvals. macOS 27's
Live-Activity model validates the direction; keep expanding what the island
can host (progress, transfers, agent state).

## Launcher

Spotlight: centred capsule + results card, Top Hit, grouped sections,
keystroke hints. Ours matches structurally (query capsule, Top Hit,
Applications/Open Windows groups, `esc` hint). Take: macOS shows
*inline actions* in results (e.g. "Search web"); our launcher could grow a
right-side action hint column — documented, not built.

## Lock screen

macOS 27: big centred clock, avatar + capsule password, wallpaper-dominant,
Control-Strip-style quick actions at edge. Ours: centred clock variant +
corner sign-in variant + dashboard variant — already a superset.

## Desktop

macOS 27's real new surface: **widget cards living on the desktop**
(calendar, weather — glass cards under the bar). Built: `nexios-widgets`,
a click-through `Layer::Bottom` surface painting a Monday-first calendar
card, a blue weather card (extended `weather.txt`: temp, city, condition,
H:L lines) and a photos card (newest PNG in `~/Pictures`, else a
placeholder), all in the shared Glass material. `show_widgets=0` in
`nexios.conf` turns the layer off. The agent-mode `solyra-desk` card stays
for `desktop_mode=agent`.

## The one macOS can't do

`macos-and-linux.jpg`: two kernels' userspaces on one desktop. That is the
nexios thesis — the Space model turns a hosted personality into a workspace,
not a window. No macOS feature maps to it; it is what we're building *for*.
