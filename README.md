# AVM1 Decompiler

An avm1 bytecode decompiler using [ruffle](https://github.com/ruffle-rs/ruffle) (bytecode parsing)
and [dprint/typescript](https://dprint.dev/plugins/typescript/) (output formatting).

It uses a relatively unusual single-sweep approach, where the whole
result is built during reading.

As of now it can do most things, with the big notable exception
being if/else recovery.

## Usage

```shell
avm1-decompiler decompile [--strict] [--out <PATH>] [--pool <PATH>] <PATH>
```

## Current status

Example of a good result (thank you dprint for the nice formatting!)

```ecma script level 4
function SetupConfData() {
  mcSettingTarget = {
    "country_version00": [
      [$1.scn00.mc_item01.item_userinfo_pack00.text_shogo_set00, 1],
      [$1.scn00.mc_item02.item_userinfo_pack00.text_shogo_set00, 1],
      [$1.scn00.mc_item03.item_userinfo_pack00.text_shogo_set00, 1],
      [$1.scn00.mc_item04.item_userinfo_pack00.text_shogo_set00, 1],
      [$1.scn00.mc_item05.item_userinfo_pack00.text_shogo_set00, 1],
    ],
    "mode_select00": [[$1.scn00.mc_item00.mode_select00, 1], [$1.scn00.mc_item00.usecard_select00, 1]],
  };
  WinInOutConf = [[$1.scn00, [2, 3], [3, 2]], [$1.bg, [0, 0], [0, 0]]];
  mcVarSetTarget = { "main": $1, "main2": $1.scn00 };
}
```
