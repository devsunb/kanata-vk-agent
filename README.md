# kanata-appvk

Watch macOS frontmost app and press/release kanata virtual keys to enable application-aware key mappings

## What does this do?

- on start
  - release all bundle id virtual keys
  - press the virtual key named with the frontmost app's bundle id (if it is included in the `-b` option)
- when the frontmost app changes (and that app's bundle id is included in the `-b` option)
  - release the pressed bundle id virtual key (if exists)
  - press the new frontmost app's bundle id virtual key

## Install

For now, you can clone this repository, build it yourself, copy it to your PATH, and run it.

```sh
git clone https://github.com/devsunb/kanata-appvk.git
cd kanata-appvk
cargo build --release
cp target/release/kanata-appvk "$HOME/.local/bin/kanata-appvk"
```

If you're interested in using launchd to automatically run in the background, see the following script.

```sh
LAUNCH_AGENTS_PATH="$HOME/Library/LaunchAgents"
KANATA_APPVK_ID="local.kanata-appvk"
KANATA_APPVK="$HOME/.local/bin/kanata-appvk"
KANATA_APPVK_PLIST="$LAUNCH_AGENTS_PATH/$KANATA_APPVK_ID.plist"

cat <<EOF | tee "$KANATA_APPVK_PLIST" >/dev/null
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
  <dict>
    <key>Label</key>
    <string>$KANATA_APPVK_ID</string>

    <key>ProgramArguments</key>
    <array>
      <string>$KANATA_APPVK</string>
      <string>-p</string>
      <string>5829</string>
      <string>-b</string>
      <string>com.apple.Safari,org.mozilla.firefox</string>
    </array>

    <key>RunAtLoad</key>
    <true />

    <key>KeepAlive</key>
    <dict>
      <key>Crashed</key>
      <true />
      <key>SuccessfulExit</key>
      <false />
    </dict>

    <key>StandardOutPath</key>
    <string>/tmp/kanata_appvk_stdout.log</string>
    <key>StandardErrorPath</key>
    <string>/tmp/kanata_appvk_stderr.log</string>
  </dict>
</plist>
EOF

launchctl bootout gui/501 "$KANATA_APPVK_PLIST" 2>/dev/null || true
launchctl bootstrap gui/501 "$KANATA_APPVK_PLIST"
launchctl enable "gui/501/$KANATA_APPVK_ID"
```

You may need to modify the executable(`$HOME/.local/bin/`) and log(`/tmp/`) path, uid(`501`), port(`5829`), bundle identifiers(`com.apple.Safari,org.mozilla.firefox`), etc. to suit your system.

## Usage

```sh
$ kanata-appvk --help

Watch macOS frontmost app and press/release kanata virtual keys

Example: kanata-appvk -p 5829 -b com.apple.Safari,org.mozilla.firefox

Usage: kanata-appvk [OPTIONS]

Options:
  -l, --log-level <LOG_LEVEL>
          Log level

          [default: info]
          [possible values: off, error, warn, info, debug, trace]

  -p, --port <PORT>
          TCP port number of kanata

          [default: 5829]

  -b, --bundle-ids <BUNDLE_IDS>
          Bundle Identifiers, each of which is the name of a virtual key

  -f, --find-id-mode
          Just print frontmost app's Bundle Identifier when it changes without connecting to kanata

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version
```

## Prerequisites

- kanata must be running on localhost with the TCP port set
- Make sure all bundle identifiers are defined in defvirtualkeys in your kanata configuration.
  - The action for each virtual key can be any value (when in doubt, use nop0)
  - Use [switch and input syntax](https://jtroo.github.io/config.html#switch) for application-aware key behavior

### kanata configuration example

```kbd
(defsrc
  1
)

(deflayer test
  @test
)

(defvirtualkeys
  com.apple.Safari nop0
  org.mozilla.firefox nop0
  com.github.wez.wezterm nop0
)

(defalias
  test (switch
    ((input virtual com.apple.Safari)) 1 break
    ((input virtual org.mozilla.firefox)) 2 break
    () 3 break
  )
)
```

The above kanata configuration would make the key `1` act as `1` in Safari, `2` in Firefox, and `3` in other apps.

kanata-appvk should be running with a command like:

```sh
kanata-appvk -p 5829 -b com.apple.Safari,org.mozilla.firefox,com.github.wez.wezterm
```

Unless the current frontmost app is one of the apps passed with the `-b` option, all bundle identifier virtual keys will be in the released state.

This means that in the above configuration, if you switch apps in the order Safari - Finder - Firefox,
the activated virtual key will be `com.apple.Safari` - (all released) - `org.mozilla.firefox`, respectively.

If you're curious about what your app's bundle identifier is, use the `-f` option.

kanata-appvk will not connect to kanata, and will just print the bundle identifier of the frontmost app when the frontmost app changes.

```sh
kanata-appvk -f
```

## Background

I'm a macOS user, and until I started using [kanata](https://github.com/jtroo/kanata), I was using [Karabiner-Elements](https://github.com/pqrs-org/Karabiner-Elements) for application-aware key mapping.

kanata uses [Karabiner-DriverKit-VirtualHIDDevice](https://github.com/pqrs-org/Karabiner-DriverKit-VirtualHIDDevice) internally on macOS, so it cannot be used with Karabiner-Elements. [kanata#1211](https://github.com/jtroo/kanata/issues/1211)

The reason I decided to use kanata instead of Karabiner-Elements is that kanata has many features such as layers and macros that are difficult to implement in Karabiner-Elements.

However, one thing I missed when switching from Karabiner-Elements to kanata was application-aware key mapping.
kanata made it clear that application-aware layer switching should be done through an external tool, not within kanata. [kanata#770](https://github.com/jtroo/kanata/discussions/770)

It seemed that many people had already developed tools for Linux and Windows,
([Community projects related to kanata](https://github.com/jtroo/kanata#community-projects-related-to-kanata))
but I couldn't find one for macOS. So I decided to create an application-aware kanata helper tool for macOS.

Initially, I tried to switch layers, but realized that for simple configuration and to avoid tricky situations like switching apps during layer-toggle(layer-while-held),
it was better to use virtual keys rather than layers. So I created a tool that allows application-aware key mappings based on virtual keys.

## Note

- This tool uses Apple's macOS Objective-C frameworks, so it only supports macOS
- kanata allows [up to 767 virtual keys](https://jtroo.github.io/config.html#virtual-keys)

### live reload

kanata [live reload](https://jtroo.github.io/config.html#live-reload) does not run when the virtual key is pressed, so it will run on the next time all virtual keys are released (when switched to an app not included in `-b`)

```
...
18:06:24.3392 [INFO] Requested live reload of file: /Users/sunb/dev/dotfiles/kanata/kanata.kbd
// Live reload triggered but not executed because the virtual key is pressed
18:06:25.1873 [INFO] tcp server fake-key action: org.mozilla.firefox,Release
18:06:25.1875 [INFO] tcp server fake-key action: com.apple.Safari,Press
// Frontmost app switched Firefox to Safari but live reload still not executed because both app included in bundle ids
18:06:28.0506 [INFO] tcp server fake-key action: com.apple.Safari,Release
18:06:28.0525 [INFO] process unmapped keys: true
// Frontmost app switched to Finder and live reload executed
...
```

## Credit

- [appwatcher](https://github.com/meschbach/appwatcher): Example of using cgo to execute go code when macOS frontmost app changes
- [clavy](https://github.com/rami3l/clavy): I had never used rust before, so I was going to make it in go, but then I saw that this repository uses objc2 libraries and channel, so I rebuilt it in rust.
- [qanata](https://github.com/veyxov/qanata): Referenced how to communicate over a TCP connection with kanata in rust.
