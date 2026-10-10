# Private Turnip driver

`build.sh` pins Mesa `eda9aceb39d5ff169b096444abe366bdf2269e24` and applies
`q10c.patch`: P010 sampling, TP10 compressed sampling, and 48×4/24×4 UBWC blocks.
P010 format/blit handling follows the [upstream proposal](https://gitlab.freedesktop.org/Valentine/mesa/-/commits/tu-p010-v4).
`build.patch` fixes Zig linking and Meson configuration warnings.
The compiler wrapper uses regular archives because Zig 0.15 rejects thin archives.

Only Turnip/MSM is built, with static libdrm and zlib. Mesa's pinned wraps verify
their downloads. `dist/mesa/` contains the driver, relative ICD manifest and
modified source archive with original notices. Launchers select it per process.
