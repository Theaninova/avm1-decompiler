# AVM1 Decompiler

An avm1 bytecode decompiler using [ruffle](https://github.com/ruffle-rs/ruffle)

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

Example result

```ecma script level 4
function SetMcPath() {
  mcMeterHari = [[$1.mc_meter_set.mc_meter_set_fps.mc_speed_hari, $1.mc_meter_set.mc_meter_set_fps.mc_tacho_hari, $1.mc_meter_set.mc_meter_set_fps.mc_boost_hari], [$1.mc_meter_set.mc_meter_set_tps.mc_speed_hari, $1.mc_meter_set.mc_meter_set_tps.mc_tacho_hari, $1.mc_meter_set.mc_meter_set_tps.mc_boost_hari]]
}
function Init() {
  SetMcPath()
  CheckOfVarsDef()
  return undefined
}
function MeterMain() {
  $1 = 0
  $2 = 0
  $3 = [CarSPD, CarRPM, CarBOOST]
  $4 = 0
  $5 = [false, false, false]
  $1 = 0
  ??? If(If { offset: 1159 })
  ??? If(If { offset: 5 })
  ??? Jump(Jump { offset: 1102 })
  $2 = 0
  ??? If(If { offset: 878 })
  ??? If(If { offset: 29 })
  ??? If(If { offset: 12 })
  ??? If(If { offset: 207 })
  $4 = ((AngleChangePointMAP[$1][$2] - AngleChangePointMAP[$1][($2 - 1)]) / (RealValChangePointMAP[$1][$2] - RealValChangePointMAP[$1][($2 - 1)]))
  mcMeterHari[GetFsView[0]][$1]._rotation = (AngleChangePointMAP[$1][($2 - 1)] + (($3[$1] - RealValChangePointMAP[$1][($2 - 1)]) * $4))
  $5[$1] = true
```
