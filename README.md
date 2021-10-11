# Water cellar automaton

## Oweview

<img src="https://github.com/vesord/HeavyStuffForOtherRepos/blob/master/water_cellular_automaton/water_rain_mac.gif" width="321" height="250" />

More examples [bellow this page](#more-examples)

## Info

No-engine OpenGL CPU Rust project.

Main goal: make surface around points provided by user and then approximately simulate water behavior on it.

Surface interpolation is made by Krigging or Radial basis function algorithm (by users's choice).

Water simulation is made by celluar automaton algorithm.

It is possible to simulate rain, waves from sides and increasing water level over the whole surface.

## Instalation

Install [rust](https://www.rust-lang.org/tools/install)

### macOS

Install [SDL2](https://www.libsdl.org/download-2.0.php) and make sure that cargo has access to libSDL2.a

`cargo run`

### Linux

`cargo run`

## Controls

Point data file should locate in `assets/grids` dir

By default its `assets/grids/grid.mod1`

- `mouse move with left button pushed` : model *rotation*
- `W` `A` `S` `D` : add water *waves* from North, West, South, East accordingly
- `R` : enable *rain*
- `F` : *flush*
- `1` : *Krigging* surface modulation
- `2` : *Radial basis function* surface modulation

## More examples

<table>
  <tr>
    <td> Rain </td>
    <td> Water level up </td>
  </tr>
  <tr>
    <td> <img src="https://github.com/vesord/HeavyStuffForOtherRepos/blob/master/water_cellular_automaton/water_rain_mac.gif" width="300" height="233" /> </td>
    <td> <img src="https://github.com/vesord/HeavyStuffForOtherRepos/blob/master/water_cellular_automaton/water_lvlup_mac.gif" width="300" height="233" /> </td>
  </tr>

  <tr>
    <td> Wave </td>
    <td> Another wave </td>
  </tr>
  <tr>
    <td> <img src="https://github.com/vesord/HeavyStuffForOtherRepos/blob/master/water_cellular_automaton/water_wave_0_mac.gif" width="300" height="233" /> </td>
    <td> <img src="https://github.com/vesord/HeavyStuffForOtherRepos/blob/master/water_cellular_automaton/water_wave_1_mac.gif" width="300" height="233" /> </td>
  </tr>
  
  <tr>
    <td> Krigging rain </td>
    <td> Krigging water level up </td>
  </tr>
  <tr>
    <td> <img src="https://github.com/vesord/HeavyStuffForOtherRepos/blob/master/water_cellular_automaton/water_rain_krigging_mac.gif" width="300" height="233" /> </td>
    <td> <img src="https://github.com/vesord/HeavyStuffForOtherRepos/blob/master/water_cellular_automaton/water_lvlup__krigging_mac.gif" width="300" height="233" /> </td>
  </tr>
</table>
