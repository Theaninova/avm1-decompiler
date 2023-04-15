# AVM1 Decompiler

An avm1 bytecode decompiler using [ruffle](https://github.com/ruffle-rs/ruffle) (bytecode parsing)
and [dprint/typescript](https://dprint.dev/plugins/typescript/) (output formatting).

It can do most things as of right now, with the big notable exception
being any sort of control flow recovery.

The intention is to provide readable results.

## Usage

```shell
avm1-decompiler decompile [--strict] [--out <PATH>] [--pool <PATH>] <PATH>
```

## Features

- [x] Full Push/Pop elimination
- [x] Expressionization
- [ ] Control flow recovery

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

## Branch recovery

We can look at the stack in terms of superpositions.

When a branch occurs, the stack items will be in a superposition.

For example:

```
0  push 4
1  push x
2  gte       // x > 4
3  jumpif 7  // if (x > 4) jump 9        # Open stack @3 end 7
5  push "a"  // push("a"@3)
6  jump 8    //                          # Pause stack @3 until 8
7  push "b"  // push("b")   -> stack: ["a"]@3, ["b"]
8  trace     // trace(x > 4 ? "a" : "b") # Open stack @3
```

There are four cases to consider

1. Simple if/else
```
                                 ┌──────────────────────┐
           ┌─────────────────────┼────┐                 │
           │                     │    ▼                 ▼
push 1  jumpif push "test" call jump push "test2" call push "test3" call
 ┌──────────┐  ┌─────────────┐         ┌────────────┐        ┌────────┐
 └──────────┘  └─────────────┘         └────────────┘        └────────┘

```
2. Ternary
3. Loops
4. Short-circuit logical and/or (don't think amv1 has that?)

It's probably a good time to mention now that there are
things that are _theoretically_ possible to do using push/pop/jump,
but can't happen if the code was compiled with a relatively
simplistic compiler.

One thing that is especially interesting here is that blocks
formed by jumps _always_ have an isolated stack, with the
Exception being remaining stack values in case of a ternary.

