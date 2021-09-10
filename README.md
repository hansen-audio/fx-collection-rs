# FX Colletion Library for Rust

## Summary

The ```fx-collection``` library combines basic modules from ```dsp-tool-box-rs``` to audio effects. The trance gate effect for example uses three modulation phases and a one pole filter from the ```dsp-tool-box-rs```. 

### Dependency map

```
fx-collection-rs
+-- dsp-tool-box-rs
```

## Building the project

Execute the following commands on cli.

```
git clone https://www.github.com/hansen-audio/fx-collection-rs.git
cd fx-collection-rs
cargo build
cargo test
```

## Effects

Currently the following effects are avaiable:

* Trance Gate

### Using the effects

All effect classes in this library contain a ```Context``` and a ```trait``` in order to modify the ```Context```.

#### Setting parameters of the context

For the trance gate for instance, use the ```trance_gate::Context::new()``` method in order to get a valid ```Context```.

#### Processing an effect

TODO
```

## License

Copyright 2021 Hansen Audio

Licensed under the MIT: https://mit-license.org/
