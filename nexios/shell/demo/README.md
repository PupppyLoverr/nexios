# Demo fixtures

Files used by the live shell demo documented in `nexios/README.md`
("Run it live"):

- `weather.txt` — copy to `~/.config/nexios/weather.txt`; the weather
  widget card reads it (temp, city, condition, `H:x L:y`).
- Put any PNG in `~/Pictures/` and the Photos widget card shows it.
- The example niri config is `../niri.kdl`; the lock PAM service is
  `../nexios-lock.pam` -> `/etc/pam.d/nexios-lock`.
- `nexios-agent-demo` (in `../bin/`) plays a fake runtime through the
  island approval loop; see `docs/agent-protocol.md`.
- `thunar.xml` — copy to `~/.config/xfce4/xfconf/xfce-perchannel-xml/thunar.xml`
  for the Finder look: no menubar/toolbar/statusbar inside the window
  (menus live in the panel's menu bar), icon view, path-buttons location
  bar. Takes effect on new windows only.
