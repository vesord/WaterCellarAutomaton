# Water cellar automaton

## Oweview

<img src="" width="200" height="200" />

// TODO: add gifs

More examples [bellow this page](#more-examples)

## Info

No-engine OpenGL CPU Rust project.

Main goal: make surface around points provided by user and then approximately simulate water behavior on it.

Surface interpolation is made by Krigging or Radial basis function algorithm (by users's choice).

Water simulation is made by celluar automaton algorithm.

It is possible to simulate rain, waves from sides and increasing water level over the whole surface.

## Instalation

Install [rust](https://www.rust-lang.org/tools/install)

// TODO: install project for mac

// TODO: install project for linux

## Controls

- `mouse move with left button pushed` : model *rotation*
- `W` `A` `S` `D` : add water *waves* from North, West, South, East accordingly
- `R` : enable *rain*
- `F` : *flush*
- `1` : *Krigging* surface modulation
- `2` : *Radial basis function* surface modulation

## Demonstrations

// TODO: add gifs

<table>
  <tr>
    <td> <img src="" width="200" height="200" /> </td>
    <td> <img src="" width="200" height="200" /> </td>
    <td> <img src="" width="200" height="200" /> </td>
  </tr>
</table>
