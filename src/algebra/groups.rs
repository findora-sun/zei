use rand::{Rng, CryptoRng};
use digest::generic_array::typenum::U64;
use digest::Digest;
use std::fmt::Debug;


pub trait Scalar: Debug + Sized + PartialEq + Eq + Clone {
    // generation
    fn random_scalar<R: CryptoRng + Rng>(rng: &mut R) -> Self;
    fn from_u32(value: u32) -> Self;
    fn from_u64(value: u64) -> Self;
    fn from_hash<D>(hash: D) -> Self
    where D: Digest<OutputSize = U64> + Default;

    //arithmetic
    fn add(&self, b: &Self) -> Self;
    fn mul(&self, b: &Self) -> Self;

    // serialization
    fn to_bytes(a: &Self) -> Vec<u8>;
    fn from_bytes(bytes: &[u8]) -> Self;
}


pub trait Group: Debug + Sized + PartialEq + Eq + Clone{
    type ScalarType: Scalar;
    const COMPRESSED_LEN: usize;
    const SCALAR_BYTES_LEN: usize;
    fn get_identity() -> Self;
    fn get_base() -> Self;

    // compression/serialization helpers
    fn to_compressed_bytes(&self) -> Vec<u8>;
    fn from_compressed_bytes(bytes: &[u8]) -> Option<Self>;

    //arithmetic
    fn mul(&self, scalar: &Self::ScalarType) -> Self;
    fn add(&self, other: &Self) -> Self;
    fn sub(&self, other: &Self) -> Self;
}
