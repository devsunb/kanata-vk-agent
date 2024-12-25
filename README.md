# kanata-appvk

Watch macOS frontmost app and press/release kanata virtual keys

## Background

I'm a macOS user, and until I started using [kanata](https://github.com/jtroo/kanata), I was using [Karabiner-Elements](https://github.com/pqrs-org/Karabiner-Elements) for application-aware key mapping.

kanata uses [Karabiner-DriverKit-VirtualHIDDevice](https://github.com/pqrs-org/Karabiner-DriverKit-VirtualHIDDevice) internally on macOS, so it cannot be used with Karabiner-Elements. [kanata#1211](https://github.com/jtroo/kanata/issues/1211)

The reason I decided to use kanata instead of Karabiner-Elements is that kanata has many features such as layers and macros that are difficult to implement in Karabiner-Elements.

However, one thing I missed when switching from Karabiner-Elements to kanata was application-aware key mapping.

kanata made it clear that application-aware layer switching should be done through an external tool, not within kanata. [kanata#770](https://github.com/jtroo/kanata/discussions/770)

It seemed that many people had already developed tools for Linux and Windows, but I couldn't find one for macOS. So I decided to create an application-aware kanata helper tool for macOS.

Initially, I tried to switch layers, but realized that for simple configuration and to avoid tricky situations like switching apps during layer-toggle(layer-while-held), it was better to use virtual keys rather than layers. So I created a tool that allows application-aware key mappings based on virtual keys.

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
kanata-appvk -p 5829 -d com.apple.Safari,org.mozilla.firefox,com.github.wez.wezterm
```

If you're curious about what your app's bundle identifier is, use the -f option.

kanata-appvk will not connect to kanata, and will just print the bundle identifier of the frontmost app when the frontmost app changes.

```sh
kanata-appvk -f
```

## How it works

- on start
  - release all bundle identifier virtual keys
  - press the virtual key named with the frontmost app's bundle identifier
- when the frontmost app changes (and that app's bundle identifier is included in the bundle identifiers option)
  - release the pressed virtual key
  - press the new frontmost app's bundle identifier virtual key

## Note

- As explained in Background, this tool currently only support macOS
- kanata [live reload](https://jtroo.github.io/config.html#live-reload) does not run when the virtual key is pressed, so it will run on the next app change
- kanata allows [up to 767 virtual keys](https://jtroo.github.io/config.html#virtual-keys)
