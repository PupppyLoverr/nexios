# Take-map: what nexios (and Solyra) can reuse, component by component

Everything here is license-clean to reuse per the per-file SPDX headers.
"Reuse" = lift into another project as-is or with small changes;
"Recipe" = the design/know-how is the valuable part, not the code.

| Component | License | Reuse value | Notes |
|---|---|---|---|
| `fki-ko/` | GPL-2.0 | **Recipe + code** | The foreign-kernel interface: kthread-per-hosted-CPU, fault forwarding, CMA carve-out, hrtimer/IPI bridging. Generalises to *any* hosted kernel, not just XNU — the abstraction is the IP. |
| `xnu-ko/` (incl. `world.S`) | APSL-2.0 | **Recipe** | The world switch (TPIDR_EL1/PAC-key/SP swap) and the page-table trick (XNU range inside Linux's unused L1 slot L0[511]/L1[384..495], hardware walks XNU's tables). Specific to XNU but the pattern — "host lends its tables to the guest" — is transferable. |
| `xnu-patches/` | APSL-2.0 | **Code (for XNU only)** | The LINUX64 machine layer: versioned `linux64_abi.h`, named host calls, `if (linux64_hook)` escape hatches. Model for how to keep hosted-kernel diffs maintainable across upstream releases. |
| `kexts/` (XnuDisk, XnuDrm, XnuKeystore, CPU) | APSL-2.0 / BSD | **Recipe** | How a hosted OS gets devices with no hardware: disk-as-file, framebuffer, netif, keystore, DRM user-client. The device inventory of a minimal personality. |
| `gpu/xnu-drm/` + `gpu/venus/` + `gpu/xnu-cpu-metal/` | GPL/MIT | **★ Biggest prize — recipe + patches** | The GPU bridge: Darwin Mesa Vulkan driver → IOKit user client → FKI → `xnudrmd` → host render node (virtio-gpu Venus / lavapipe). Includes the hard-won Mesa/virglrenderer patch notes: 16 KiB blob alignment on macOS hosts, `VK_KHR_external_memory_fd` vs in-process sharing, `VK_LUNARG_direct_driver_loading`. **Directly applicable to Solyra's planned per-agent micro-VMs (virtio-gpu + PipeWire)** — this is the same bridge problem from the other side. |
| `scripts/` (build/boot/disk/console/service) | GPL-2.0 | **Recipe** | The operational shape: build modules against running kernel → bring up net + GPU bridge → boot → console/screenshot/input tools → systemd service with log retention. `nx/` turns this into one driver. |
| `docs/production.md` | GPL-2.0 | **Recipe** | The "PoC→production" debt ledger format itself: two columns, "what it costs if left in place". Worth copying as a doc pattern for any bring-up. |
| `linux/` (in-tree FKI driver form) | GPL-2.0 | **Code** | Same FKI as an in-tree driver — for a distro that builds its own kernel (Solyra ships a Fedora kernel today; in-tree FKI is the cleaner long-term route vs DKMS-style builds). |
| Architecture: the page-table slot trick | — | **Recipe** | Sharing the root page table at an unused L1 slot so hardware walks the guest's own tables — eliminates the VM-exit/MMU-shadow problem entirely. Generalisable to hosted personalities on arm64. |

| `tools/hosted/` (fb2png, xnu-input, serial-pty, net-up, console-send-file, gui-user) | GPL-2.0 | **Code — lift as-is** | The host-side plumbing for a hosted desktop: `fb2png.py` (~40 lines: raw BGRA framebuffer → PNG), `xnu-input.py` (writes Linux evdev records into a file → click/type/scroll inside the guest), `gui-user.py` (auto-login setup). Directly reusable for any "draw guest screen, send guest input" feature. |
| `scripts/mk-macvm-disk.sh` + `gui-user.py` (the reach-the-desktop recipe) | GPL-2.0 | **Recipe — the punch-list a hosted macOS needs before Dock/Finder appear** | loginwindow holds the Dock and Finder until Setup Assistant quits: autoLoginUser + `/etc/kcpassword` (XOR `7D 89 52 23 D2 BC DD EA A3 B9 1F`, NUL-padded ×12), `.AppleSetupDone`, the 18-key `DidSee*`/`LastSeen*Build|Version` plist set + `~/.skipbuddy` + `MiniBuddyLaunch=false`. Plus: launchd disable list (watchdogd, audio, PerfPowerServices, Rosetta/oahd), `pmset -a sleep 0`, `screensaver idleTime=0` (ByHost), `com.apple.keyboardtype` (else Keyboard Setup Assistant opens at every login), `kern.memorystatus_highwater_enabled=0` (CPU-Metal blows jetsam limits), cryptex merge into the system volume (unseals root; needs `lp-sip0=0x800`). |
| `docs/images/*.jpg` | GPL-2.0 (photos of a running system) | **Visual reference** | Real macOS 27 desktop/menu bar/dock/widgets/Finder — the exact target proportions for the nexios clone. Reference only, never copy pixels. |

## Solyra-side takeaways (for the Infi-AI repo, recorded here)

- **Dock/Finder themselves**: not takeable — they are Apple's binaries
  (`Dock.app`, `Finder.app`, `loginwindow`, `WindowServer`) on the user's own
  macOS install; the repo deliberately ships none of them. What is takeable is
  the recipe that gets them running (table above) and the host plumbing.


- **Agent micro-VM GPU path**: use their Venus/Mesa patch list + the
  `xnudrmd` service shape (userspace daemon on the render node, FKI-style
  call channel) when level-2 sandboxing lands virtio-gpu.
- **Space model**: a hosted personality wants the same panel surfaces an
  agent gets — task state, approval requests, audit — i.e. the file-IPC
  contract generalises from "agent runtime" to "any hosted thing with a UI".
- **aarch64 image becomes more valuable**: nexios only runs on arm64 Linux;
  an aarch64 Solyra image is the prerequisite for any in-person demo.
