use anyhow::Result;

use super::Context;
use crate::{
    io,
    command::{X25519, Absorb, Commit, Mask},
    types::NBytes,
};
use iota_streams_core::{
    sponge::prp::PRP,
};
use iota_streams_core_edsig::key_exchange::x25519;

impl<'a, F: PRP, IS: io::IStream> X25519<&'a x25519::StaticSecret, &'a x25519::PublicKey> for Context<F, IS>
{
    fn x25519(&mut self, sk: &x25519::StaticSecret, pk: &x25519::PublicKey) -> Result<&mut Self> {
        let shared = sk.diffie_hellman(pk);
        self.spongos.absorb(shared.as_bytes());
        Ok(self)
    }
}

impl<'a, F: PRP, IS: io::IStream> X25519<x25519::EphemeralSecret, &'a x25519::PublicKey> for Context<F, IS>
{
    fn x25519(&mut self, sk: x25519::EphemeralSecret, pk: &x25519::PublicKey) -> Result<&mut Self> {
        let shared = sk.diffie_hellman(pk);
        self.spongos.absorb(shared.as_bytes());
        Ok(self)
    }
}

impl<'a, F: PRP, IS: io::IStream> X25519<&'a x25519::StaticSecret, &'a mut NBytes> for Context<F, IS>
{
    fn x25519(&mut self, sk: &x25519::StaticSecret, key: &mut NBytes) -> Result<&mut Self> {
        let mut ephemeral_ke_pk = x25519::PublicKey::from([0_u8; 32]);
        (*self)
            .absorb(&mut ephemeral_ke_pk)?
            .x25519(sk, &ephemeral_ke_pk)?
            .commit()?
            .mask(key)
    }
}