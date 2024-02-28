use sov_modules_api::digest::Digest;

use sov_state::Prefix;

pub(crate) fn get_token_address<C: sov_modules_api::Context>(
    token_name: &str,
    sender: &[u8],
    salt: u64,
) -> C::Address {
    let mut hasher = C::Hasher::new();
    hasher.update(token_name.as_bytes());
    hasher.update(sender);
    hasher.update(salt.to_le_bytes());

    let hash: [u8; 32] = hasher.finalize().into();
    C::Address::from(hash)
}

pub(crate) fn prefix_from_address_with_parent<C: sov_modules_api::Context>(
    parent_prefix: &Prefix,
    token_address: &C::Address,
) -> Prefix {
    let mut prefix = parent_prefix.as_aligned_vec().clone().into_inner();
    // どのような値がはいるか確認
    prefix.extend_from_slice(format!("{}", token_address).as_bytes());
    Prefix::new(prefix)
}
