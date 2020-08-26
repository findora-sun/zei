use super::MTHash;
use curve25519_dalek::scalar::Scalar;
use digest::Digest;
use rand_chacha::ChaChaRng;
use rand_core::SeedableRng;

pub struct MiMCHash {
  c: [Scalar; MIMC_ROUNDS],
}
pub(crate) const MIMC_ROUNDS: usize = 159;

impl MTHash for MiMCHash {
  type S = Scalar;
  fn new(level: usize) -> MiMCHash {
    MiMCHash { c: compute_mimc_constants(level) }
  }
  fn digest(&self, values: &[&Scalar]) -> Scalar {
    let mut sa = Scalar::from(0u8);
    let mut sc = Scalar::from(0u8);
    for value in values.iter() {
      let x = mimc_feistel(&(*value + sa), &sc, &self.c[..]);
      sa = x.0;
      sc = x.1;
    }
    sa
  }

  fn digest_root(&self, size: usize, values: &[&Scalar]) -> Scalar {
    let x = Scalar::from(size as u64);
    let mut vec = Vec::with_capacity(values.len() + 1);
    vec.push(&x);
    vec.extend_from_slice(values);
    self.digest(&vec[..])
  }
}

pub(crate) fn mimc_f(s: &Scalar, c: &Scalar) -> Scalar {
  let x = s + c;
  let x2 = x * x;
  (x2 * x2) * x
}

#[allow(clippy::needless_range_loop)]
pub(crate) fn compute_mimc_constants(level: usize) -> [Scalar; MIMC_ROUNDS] {
  let mut c = [Scalar::from(0u32); MIMC_ROUNDS];
  let mut hash = sha2::Sha256::new();
  hash.input(level.to_string());
  let mut seed = [0u8; 32];
  seed.copy_from_slice(&hash.result()[..]);
  let mut prng = ChaChaRng::from_seed(seed);
  for i in 1..MIMC_ROUNDS - 1 {
    c[i] = Scalar::random(&mut prng);
  }
  c
}

pub(crate) fn mimc_feistel(left: &Scalar, right: &Scalar, c: &[Scalar]) -> (Scalar, Scalar) {
  let mut xl = *left;
  let mut xr = *right;
  for ci in c {
    let aux = xl;
    xl = xr + mimc_f(&xl, ci);
    xr = aux;
  }
  (xl, xr)
}

// TODO tests