# Nexios

Nexios is the **Nexi experiment**: a fork of
[AFK-surf/xnulinux](https://github.com/AFK-surf/xnulinux) that treats "a hosted
foreign kernel personality" as a first-class desktop citizen — not a VM window,
not a container, but a *Space* living next to the host's own.

Upstream's technical achievement is the kernel plumbing: XNU (macOS's kernel,
built from Apple's open source, APSL-2.0) runs as a Linux kernel module via the
FKI ("foreign kernel interface") bridge, so macOS processes execute natively
alongside Linux processes on the same CPUs.

Nexios asks the next question: **what does an OS that *hosts* personalities
look like?** One desktop where a Linux session, a hosted macOS session, and
eventually other personalities each live in their own space — sharing one
screen, one input stack, one audit trail, and nothing else.

## What lives here

| Path | Contents |
|---|---|
| `fki-ko/`, `xnu-ko/`, `xnu-patches/`, `kexts/`, `linux/` | Upstream kernel plumbing — kept close to upstream for rebasing. |
| `gpu/` | The Metal→Vulkan→DRM GPU bridge (Venus path + lavapipe fallback). |
| `scripts/`, `tools/` | Upstream bring-up tooling. |
| `nx/` | The Nexios driver: one CLI (`build`/`disk`/`boot`/`console`/`screenshot`/`test`) — upstream's own stated "replace ~30 scripts" direction, started here. |
| `nexios/docs/` | The experiment's own documents (below). |

## Documents

- [`docs/ui-ux.md`](docs/ui-ux.md) — what macOS 27 "Golden Gate" does well,
  observed from the hosted desktop, and how to borrow the *ideas* legally.
- [`docs/solyra-modes-map.md`](docs/solyra-modes-map.md) — the extraction
  applied to every Solyra mode (dock variants, materials, island, launcher,
  lock, desktop): at-parity vs real gaps.
- [`docs/take-map.md`](docs/take-map.md) — component-by-component map of what
  can be reused elsewhere (the GPU bridge recipe is the biggest prize).
- [`docs/architecture.md`](docs/architecture.md) — the Space model: how hosted
  personalities fit a desktop shell's security and UX story.

## Rules of the house

1. **License discipline is upstream's — keep it.** Every file keeps its SPDX
   header; the APSL/GPL boundary at FKI is a *design constraint*, not red tape.
   Nothing under `nx/` or `nexios/` may call GPL symbols from APSL-land or
   vice versa except through `fki_*`.
2. **No Apple binaries, ever.** macOS system disks, Apple's kexts and trust
   caches come from the user's own installation. A Nexios image must never
   contain a byte of macOS.
3. **Non-Apple arm64 = user's own choice.** macOS's license limits it to
   Apple-branded hardware. Nexios ships tooling only; hosted-macOS paths must
   refuse to encourage or automate the install on non-Apple silicon.
4. **Principles, not assets.** We borrow design *ideas*; no Apple icons,
   wallpapers, fonts or artwork are copied (see `docs/ui-ux.md`).

## Status

Experiment scaffolding: documents and the `nx` driver skeleton. Upstream code
is unmodified; the fork tracks `AFK-surf/xnulinux@main` and should rebase, not
diverge, until a Nexios-specific change earns its place.

Verified bring-up on stock Ubuntu 24.04/7.0 aarch64 (`7.0.0-38-generic`):
`fki.ko` and `xnu.ko` compile out-of-tree against the running headers and
`fki.ko` insmods/rmmods cleanly. A hosted-macOS boot still needs an arm64
host with 16 GB+ RAM (no nested-virt barrier — an Apple VZ guest can't run
VZ guests) and a macOS 27 disk made by Apple's installer on a Mac with Xcode.
