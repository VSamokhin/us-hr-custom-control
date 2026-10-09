# US-HR settings-panel and USB protocol analysis

This document records the interoperability analysis used by the implementation.
It is deliberately limited to ordinary settings and status operations; firmware
update commands are excluded.

## What the vendor settings panel does

The panel is a controller for state implemented in the interface firmware. It is
not the audio driver and it does not carry the audio stream. For the US-1x2HR it
exposes:

- direct-monitor mono/stereo mode;
- enable/mute state for input channels 1 and 2;
- input-versus-computer monitor balance;
- loopback enable, input mono/stereo, and computer-output mono/stereo;
- broadcast volume;
- automatic power saving;
- product, firmware/build, and current sample-rate information.

The Windows panel additionally exposes the driver's audio buffer size. That is
an ASIO-driver setting rather than a portable device command and is not part of
this USB implementation.

## Device identity and transport

| Model | USB VID:PID | Validation status |
|---|---|---|
| US-1x2HR | `0644:806f` | Control protocol tested with real hardware |
| US-2x2HR | `0644:8070` | USB identity detection only; control protocol untested |
| US-4x4HR | `0644:8071` | USB identity detection only; control protocol untested |

Only the US-1x2HR has been tested with real hardware. Recognizing the product
IDs of the US-2x2HR and US-4x4HR during USB discovery does not establish that
their status or setting commands are compatible with the US-1x2HR mappings
documented below.

Messages use vendor, device-recipient control transfers with value and index set
to zero:

| Direction | `bmRequestType` | `bRequest` | Maximum payload |
|---|---:|---:|---:|
| Host to device | `0x40` | `0x1d` | 64 bytes |
| Device to host | `0xc0` | `0x1e` | 64 bytes |

## Message format

A transfer contains typed fields and ends with `00 00`. Multi-byte integers are
little-endian.

| Type | Encoding after field ID and type |
|---:|---|
| `0` | terminator; valid only as field ID `0` |
| `1` | query marker, no payload |
| `2` | one-byte unsigned value |
| `3` | two-byte unsigned value |
| `4` | four-byte unsigned value |
| `5` | one-byte length followed by that many bytes |

Before sending, the client receives until readiness field `0xa1` reports a
non-zero value. After sending a query, acknowledgement-only replies may arrive
before the response, so the client polls until the requested field and channel
context are present.

## Global queries

| Request | Response | Meaning |
|---:|---:|---|
| `0x31` | `0x32` | firmware version (`u16`, 100 means 1.00) |
| `0x31` | `0x33` | firmware build (`u16`) |
| `0x31` | `0x34` | product name (bytes) |
| `0x31` | `0x35` | firmware build date (bytes) |
| `0x31` | `0x36` | firmware build time (bytes) |
| `0x31` | `0x37` | product signature (bytes) |
| `0x18` | `0x18` | current sample rate (`u32`) |
| `0x14` | `0x15` | automatic power save (`u8`) |

Automatic power save is written with field `0x15`, type `2`, value zero or one.

## Channel commands

Channel operations begin with group field `0x61` and index field `0x62`, both
type `2`. A query then carries a type-1 get command; a write carries the matching
type-2 set command and value.

| Property | Get | Set |
|---|---:|---:|
| fader/value | `0xc1` | `0x81` |
| mute | `0xc3` | `0x83` |
| mono/stereo | `0xc6` | `0x86` |

Observed mono/stereo values are `1` for mono and `4` for stereo. The US-1x2HR
panel maps its controls as follows:

| Control | Group | Index | Property and conversion |
|---|---:|---:|---|
| Direct monitor | 5 | 1 | mono/stereo |
| Audio input 1 | 4 | 1 | inverse of mute |
| Audio input 2 | 4 | 2 | inverse of mute |
| Monitor balance | 5 | 1 | fader; UI value is `127 - wire value` |
| Loopback enabled | 6 | 0 | fader interpreted as Boolean |
| Loopback input | 4 | 1 | mono/stereo |
| Loopback computer output | 3 | 1 | mono/stereo |
| Broadcast volume | 6 | 1 | fader, range 0 through 127 |

## Platform assessment

- macOS arm64: validated against a connected US-1x2HR through the standard USB
  stack. The vendor panel itself is an older Intel Qt application; this project
  is native Rust/arm64 when built on Apple Silicon.
- Linux x86-64: compilation is covered by CI, but the build has not been launched
  or tested with real hardware on Linux. The same libusb control transfers are
  expected to work alongside USB Audio Class support, and a narrow `uaccess`
  udev rule is included, but this remains unvalidated.
- Windows x86-64: descriptor discovery was validated with a connected US-1x2HR
  using the built-in Windows USB Audio 2 driver. The device enumerated as
  `0644:806f`, but libusb could not open it: the composite parent used
  `usbccgp`, its only function used `usbaudio2`, and neither exposed a
  libusb-compatible control path. No status query or write was performed.
  Rebinding the only function or composite parent to WinUSB would disable audio,
  so functional Windows control requires a dedicated backend that coexists with
  the TASCAM audio driver. Windows CI builds and release packages are disabled
  until such a backend exists.

The only hardware validation performed to date used a US-1x2HR reporting
firmware 1.00 build 14. Readback of every modeled setting succeeded, and the
write path was verified by reapplying broadcast volume 77 and reading 77 back.
No status query or setting change has been tested on a US-2x2HR or US-4x4HR.
