# kanata-vk-agent

Control kanata virtual keys while observing frontmost app and input source on macOS to enable application and input source aware key mapping

## What does this do?

- on start
  - release all virtual keys passed as options
  - press the virtual keys named with the id of the frontmost app and current input source
- when the frontmost app or input source changes
  - release the pressed virtual key
  - press the virtual key named with the id of the new frontmost app or input source

## Install

Building from source is the only currently available installation method.

```sh
git clone https://github.com/devsunb/kanata-vk-agent.git
cd kanata-vk-agent
cargo build --release
cp target/release/kanata-vk-agent "$HOME/.local/bin/kanata-vk-agent"
```

If you're interested in using launchd to automatically run in the background, see the following script.

```sh
LAUNCH_AGENTS_PATH="$HOME/Library/LaunchAgents"
KANATA_VK_AGENT_ID="local.kanata-vk-agent"
KANATA_VK_AGENT="$HOME/.local/bin/kanata-vk-agent"
KANATA_VK_AGENT_PLIST="$LAUNCH_AGENTS_PATH/$KANATA_VK_AGENT_ID.plist"

cat <<EOF | tee "$KANATA_VK_AGENT_PLIST" >/dev/null
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
  <dict>
    <key>Label</key>
    <string>$KANATA_VK_AGENT_ID</string>

    <key>ProgramArguments</key>
    <array>
      <string>$KANATA_VK_AGENT</string>
      <string>-p</string>
      <string>5829</string>
      <string>-b</string>
      <string>com.apple.Safari,org.mozilla.firefox</string>
      <string>-i</string>
      <string>com.apple.keylayout.ABC,com.apple.inputmethod.Korean.2SetKorean</string>
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
    <string>/tmp/kanata_vk_agent_stdout.log</string>
    <key>StandardErrorPath</key>
    <string>/tmp/kanata_vk_agent_stderr.log</string>
  </dict>
</plist>
EOF

launchctl bootout gui/501 "$KANATA_VK_AGENT_PLIST" 2>/dev/null || true
launchctl bootstrap gui/501 "$KANATA_VK_AGENT_PLIST"
launchctl enable "gui/501/$KANATA_VK_AGENT_ID"
```

You may need to modify the executable(`$HOME/.local/bin/`) and log(`/tmp/`) path, uid(`501`), port(`5829`), bundle ids(`com.apple.Safari,org.mozilla.firefox`), input source ids(`com.apple.keylayout.ABC,com.apple.inputmethod.Korean.2SetKorean`), etc. to suit your system.

## Usage

```sh
$ kanata-vk-agent --help

Control kanata virtual keys while observing frontmost app and input source on macOS

Example: kanata-vk-agent -p 5829 -b com.apple.Safari,org.mozilla.firefox -i com.apple.keylayout.ABC,com.apple.inputmethod.Korean.2SetKorean

Usage: kanata-vk-agent [OPTIONS]

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

  -i, --input-source-ids <INPUT_SOURCE_IDS>
          Input Source Identifiers, each of which is the name of a virtual key

  -f, --find-id-mode
          Just print the app's bundle id or input source id when the frontmost app and input source change. In this mode, it will not connect to kanata

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version
```

## Prerequisites

- kanata must be running on localhost with the TCP port set
- Make sure all bundle ids and input source ids are defined in defvirtualkeys in your kanata configuration.
  - The action for each virtual key can be any value (when in doubt, use nop0)
  - Use [switch and input syntax](https://jtroo.github.io/config.html#switch) for application and input source aware key action

## kanata configuration example

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

  com.apple.keylayout.ABC nop0
  com.apple.inputmethod.Korean.2SetKorean nop0
)

(defalias
  test (switch
    ((input virtual com.apple.Safari)) 2 break
    ((input virtual org.mozilla.firefox)) 3 break
    () 1 break
  )
)
```

The above kanata configuration would make the key `1` act as `2` in Safari, `3` in Firefox, and `1` in other apps.

You can use a similar structure to define keys that take different actions based on the input source.
This can be useful for users who want to use the Colemak layout for English and the Qwerty layout for other input sources, such as Korean.

kanata-vk-agent should be running with a command like:

```sh
kanata-vk-agent -p 5829 -b com.apple.Safari,org.mozilla.firefox,com.github.wez.wezterm -i com.apple.keylayout.ABC,com.apple.inputmethod.Korean.2SetKorean
```

Unless the current frontmost app is one of the apps passed with the `-b` option, all bundle id virtual keys will be in the released state.
This means that in the above configuration, if you switch apps in the order Safari - Finder - Firefox,
the activated bundle id virtual key will be `com.apple.Safari` - (all released) - `org.mozilla.firefox`, respectively.

If you're curious about what your app's bundle id or input source's id is, use the `-f` option.
kanata-vk-agent will not connect to kanata, and will just print the id of the frontmost app and current input source when they change.

```sh
kanata-vk-agent -f
```

## Background

I'm a macOS user, and until I started using [kanata](https://github.com/jtroo/kanata), I was using [Karabiner-Elements](https://github.com/pqrs-org/Karabiner-Elements) for conditional (like application and input source) key mapping.

kanata uses [Karabiner-DriverKit-VirtualHIDDevice](https://github.com/pqrs-org/Karabiner-DriverKit-VirtualHIDDevice) internally on macOS, so it cannot be used with Karabiner-Elements. [kanata#1211](https://github.com/jtroo/kanata/issues/1211)

The reason I decided to use kanata instead of Karabiner-Elements is that kanata has many features such as layers and macros that are difficult to implement in Karabiner-Elements.

However, one thing I missed when switching from Karabiner-Elements to kanata was conditional key mapping.
kanata made it clear that application-aware layer switching should be done through an external tool, not within kanata. [kanata#770](https://github.com/jtroo/kanata/discussions/770)

It seemed that many people had already developed tools for Linux and Windows,
([Community projects related to kanata](https://github.com/jtroo/kanata#community-projects-related-to-kanata))
but I couldn't find one for macOS. So I decided to create a kanata helper tool for macOS.

Initially, I tried to switch layers, but realized that for simple configuration and to avoid tricky situations like switching apps during layer-toggle(layer-while-held),
it was better to use virtual keys rather than layers. So I created a tool to enable conditional key mapping based on virtual keys.

## Note

- This tool uses Apple's macOS Objective-C frameworks, so it only supports macOS
- kanata allows [up to 767 virtual keys](https://jtroo.github.io/config.html#virtual-keys)

### live reload

kanata [live reload](https://jtroo.github.io/config.html#live-reload) does not run when the virtual key is pressed,
so it will run on the next time all virtual keys are released.

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

If you pass all the input sources you're using as input-source-ids,
you might never get to a state where all virtual keys are released,
in which case live reload is not available at this time.

## Credit

- [appwatcher](https://github.com/meschbach/appwatcher): Example of using cgo to execute go code when macOS frontmost app changes
- [clavy](https://github.com/rami3l/clavy): I had never used rust before, so I tried to build it in go, but then I saw that this repository uses objc2 libraries, so I was able to rebuild it in rust.
- [qanata](https://github.com/veyxov/qanata): Referenced how to communicate over a TCP connection with kanata in rust.
