# The Space model — architecture sketch

Upstream gives us two userspaces sharing a CPU. Nexios makes that a product
feature: **a desktop where each hosted personality is a Space.**

## Model

```
                        Nexios shell (panel + compositor)
                        ┌──────────────────────────────────────┐
                        │  Space switcher · island · approvals │
                        └───────┬───────────┬──────────┬───────┘
                                │           │          │
                          host session   Darwin Space   agent Spaces
                          (Linux)        (hosted XNU)  (bwrap now,
                                                       micro-VM later)
```

A Space is defined by four things, whatever the personality:

1. **An output surface** — where its pixels go. For the Darwin Space today
   that's the hosted framebuffer (`/dev/xnu/fb` once upstream's
   char-device plan lands, or `fb_dump` polling until then); for agents,
   Wayland surfaces on reserved workspaces.
2. **An input route** — keyboard/pointer into the Space (upstream's
   `XnuInput` kext + `/dev/xnu/input`).
3. **A state channel** — task/approval/decision records the shell renders
   (Solyra's file-IPC contract: newline-JSON in a runtime dir).
4. **A lifecycle** — systemd unit (`xnulinux.service` is already that shape:
   start at boot, restart after a hosted-kernel panic, log per run).

## Why this beats "a VM window"

- The hosted desktop becomes a *workspace*, not a window: it participates in
  the shell's own navigation (overview, space switching, per-space scale).
- Agents and personalities share one audit/approval vocabulary — the same
  island that approves an agent's shell command can gate a personality's
  network or file access.
- The compositor stays the single compositor: one input stack, one clipboard
  story (eventually), one place to look.

## Near-term milestones

1. `nx` driver working end-to-end on an arm64 host (wraps upstream scripts,
   then replaces them).
2. Framebuffer → Wayland: a shell client that presents the hosted fb as a
   fullscreen surface on a dedicated workspace (CPU-rendered first).
3. Input routing into the Space when focused.
4. GPU path: hosted apps on the host GPU via the Venus/DRM bridge — the
   machinery exists upstream; integration is the work.

## Hard boundaries (unchanged from upstream)

- Hosted systems get one contiguous RAM carve-out, fixed at load — no balloon.
- Hosted kernels are trusted kernel code: a hosted panic must not take the
  host (upstream already restarts the service cleanly; keep it that way).
- Disk is a file on the Linux side — snapshot/undo for hosted state is just
  file snapshotting, which Solyra's agent undo already knows how to do.
