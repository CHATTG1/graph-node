use ethabi::{Contract, Event};
use tiny_keccak::Keccak;
use web3::types::H256;

/// Hashes a string to a H256 hash.
pub fn string_to_h256(s: &str) -> H256 {
    let mut result = [0u8; 32];
    let data = s.replace(" ", "").into_bytes();
    let mut sponge = Keccak::new_keccak256();
    sponge.update(&data);
    sponge.finalize(&mut result);

    // This was deprecated but the replacement seems to not be availible in the
    // version web3 uses.
    #[allow(deprecated)]
    H256::from_slice(&result)
}

/// Returns the contract event with the given signature, if it exists.
pub fn contract_event_with_signature<'a>(
    contract: &'a Contract,
    signature: &str,
) -> Option<&'a Event> {
    contract
        .events()
        .find(|event| event.signature() == string_to_h256(signature))
}
