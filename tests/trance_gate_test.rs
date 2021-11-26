// Copyright(c) 2021 Hansen Audio.

use fx_collection_rs::trance_gate::*;

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    #[ignore]
    fn test_trance_gate_debug_print() {
        let trance_gate = TranceGate::new();
        println!("{:#?}", trance_gate);
    }
}
