use zei_algebra::ristretto::RistrettoPoint;
use zei_algebra::{
    hash::{Hash, Hasher},
    prelude::*,
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ElGamalEncKey<G>(pub G); // pk = sk * G

impl<G: Clone> ElGamalEncKey<G> {
    pub fn get_point(&self) -> G {
        self.0.clone()
    }
    pub fn get_point_ref(&self) -> &G {
        &self.0
    }
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ElGamalDecKey<S>(pub(crate) S); // sk

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ElGamalCiphertext<G> {
    pub e1: G, // r * G
    pub e2: G, // m * G + r * pk
}

impl Hash for ElGamalEncKey<RistrettoPoint> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.to_compressed_bytes().as_slice().hash(state);
    }
}

impl ZeiFromToBytes for ElGamalCiphertext<RistrettoPoint> {
    fn zei_to_bytes(&self) -> Vec<u8> {
        let mut v = vec![];
        v.extend_from_slice(self.e1.to_compressed_bytes().as_slice());
        v.extend_from_slice(self.e2.to_compressed_bytes().as_slice());
        v
    }
    fn zei_from_bytes(bytes: &[u8]) -> Result<Self> {
        let e1 = RistrettoPoint::from_compressed_bytes(&bytes[0..RistrettoPoint::COMPRESSED_LEN])
            .c(d!(ZeiError::DeserializationError))?;
        let e2 = RistrettoPoint::from_compressed_bytes(&bytes[RistrettoPoint::COMPRESSED_LEN..])
            .c(d!(ZeiError::DeserializationError))?;
        Ok(ElGamalCiphertext { e1, e2 })
    }
}

/// Return an ElGamal key pair as `(sk, pk = sk * G)`
pub fn elgamal_key_gen<R: CryptoRng + RngCore, G: Group>(
    prng: &mut R,
) -> (ElGamalDecKey<G::ScalarType>, ElGamalEncKey<G>) {
    let base = G::get_base();
    let secret_key = ElGamalDecKey(G::ScalarType::random(prng));
    let public_key = ElGamalEncKey(base.mul(&secret_key.0));
    (secret_key, public_key)
}

/// Return an ElGamal ciphertext pair as `(r * G, m * G + r * pk)`, where `G` is a base point on the curve
pub fn elgamal_encrypt<G: Group>(
    m: &G::ScalarType,
    r: &G::ScalarType,
    pub_key: &ElGamalEncKey<G>,
) -> ElGamalCiphertext<G> {
    let base = G::get_base();
    let e1 = base.mul(r);
    let e2 = base.mul(m).add(&(pub_key.0).mul(r));

    ElGamalCiphertext::<G> { e1, e2 }
}

/// Verify that the ElGamal ciphertext encrypts m by checking `ctext.e2 - ctext.e1 * sk = m * G`
pub fn elgamal_verify<G: Group>(
    m: &G::ScalarType,
    ctext: &ElGamalCiphertext<G>,
    sec_key: &ElGamalDecKey<G::ScalarType>,
) -> Result<()> {
    let base = G::get_base();
    if base.mul(m).add(&ctext.e1.mul(&sec_key.0)) == ctext.e2 {
        Ok(())
    } else {
        Err(eg!(ZeiError::ElGamalVerificationError))
    }
}

/// Perform a partial decryption for the ElGamal ciphertext that returns `m * G`
pub fn elgamal_partial_decrypt<G: Group>(
    ctext: &ElGamalCiphertext<G>,
    sec_key: &ElGamalDecKey<G::ScalarType>,
) -> G {
    ctext.e2.sub(&ctext.e1.mul(&sec_key.0))
}

#[cfg(test)]
mod elgamal_test {
    use crate::basic::elgamal::{ElGamalCiphertext, ElGamalDecKey, ElGamalEncKey};
    use rand_chacha::ChaChaRng;
    use rmp_serde::Deserializer;
    use serde::de::Deserialize;
    use serde::ser::Serialize;
    use zei_algebra::bls12_381::{BLSGt, BLSG1, BLSG2};
    use zei_algebra::jubjub::JubjubPoint;
    use zei_algebra::prelude::*;
    use zei_algebra::ristretto::RistrettoPoint;

    fn verification<G: Group>() {
        let mut prng = ChaChaRng::from_seed([0u8; 32]);

        let (secret_key, public_key) = super::elgamal_key_gen::<_, G>(&mut prng);

        let m = G::ScalarType::from(100u32);
        let r = G::ScalarType::random(&mut prng);
        let ctext = super::elgamal_encrypt::<G>(&m, &r, &public_key);
        pnk!(super::elgamal_verify::<G>(&m, &ctext, &secret_key));

        let wrong_m = G::ScalarType::from(99u32);
        let err = super::elgamal_verify(&wrong_m, &ctext, &secret_key)
            .err()
            .unwrap();
        msg_eq!(ZeiError::ElGamalVerificationError, err);
    }

    fn decryption<G: Group>() {
        let mut prng = ChaChaRng::from_seed([0u8; 32]);
        let (secret_key, public_key) = super::elgamal_key_gen::<_, G>(&mut prng);

        let mu32 = 100u32;
        let m = G::ScalarType::from(mu32);
        let r = G::ScalarType::random(&mut prng);
        let ctext = super::elgamal_encrypt(&m, &r, &public_key);
        pnk!(super::elgamal_verify(&m, &ctext, &secret_key));

        let m = G::ScalarType::from(u64::MAX);
        let ctext = super::elgamal_encrypt(&m, &r, &public_key);
        pnk!(super::elgamal_verify(&m, &ctext, &secret_key));
    }

    fn serialize_to_json<G: Group>() {
        let mut prng = ChaChaRng::from_seed([0u8; 32]);
        let (secret_key, public_key) = super::elgamal_key_gen::<_, G>(&mut prng);

        // key serialization
        let json_str = serde_json::to_string(&secret_key).unwrap();
        let sk_de: ElGamalDecKey<G::ScalarType> = serde_json::from_str(&json_str).unwrap();
        assert_eq!(secret_key, sk_de);

        let json_str = serde_json::to_string(&public_key).unwrap();
        let pk_de: ElGamalEncKey<G> = serde_json::from_str(&json_str).unwrap();
        assert_eq!(public_key, pk_de);

        // ciphertext serialization
        let m = G::ScalarType::from(100u32);
        let r = G::ScalarType::random(&mut prng);

        let ctext = super::elgamal_encrypt(&m, &r, &public_key);
        let json_str = serde_json::to_string(&ctext).unwrap();
        let ctext_de: ElGamalCiphertext<G> = serde_json::from_str(&json_str).unwrap();

        assert_eq!(ctext, ctext_de);
    }

    fn serialize_to_message_pack<G: Group>() {
        let mut prng = ChaChaRng::from_seed([0u8; 32]);
        let (secret_key, public_key) = super::elgamal_key_gen::<_, G>(&mut prng);

        // key serialization
        let mut vec = vec![];
        secret_key
            .serialize(&mut rmp_serde::Serializer::new(&mut vec))
            .unwrap();
        let mut de = Deserializer::new(&vec[..]);
        let sk_de: ElGamalDecKey<G::ScalarType> = Deserialize::deserialize(&mut de).unwrap();
        assert_eq!(secret_key, sk_de);

        // public key serialization
        let mut vec = vec![];
        public_key
            .serialize(&mut rmp_serde::Serializer::new(&mut vec))
            .unwrap();
        let mut de = Deserializer::new(&vec[..]);
        let pk_de: ElGamalEncKey<G> = Deserialize::deserialize(&mut de).unwrap();
        assert_eq!(public_key, pk_de);

        // ciphertext serialization
        let m = G::ScalarType::from(100u32);
        let r = G::ScalarType::random(&mut prng);

        let ctext = super::elgamal_encrypt(&m, &r, &public_key);

        let mut vec = vec![];
        ctext
            .serialize(&mut rmp_serde::Serializer::new(&mut vec))
            .unwrap();

        let mut de = Deserializer::new(&vec[..]);
        let ctext_de: ElGamalCiphertext<G> = Deserialize::deserialize(&mut de).unwrap();
        assert_eq!(ctext, ctext_de);
    }

    #[test]
    fn verify() {
        verification::<RistrettoPoint>();
        verification::<BLSG1>();
        verification::<BLSG2>();
        verification::<BLSGt>();
        verification::<JubjubPoint>();
    }

    #[test]
    fn decrypt() {
        decryption::<RistrettoPoint>();
        decryption::<BLSG1>();
        decryption::<BLSG2>();
        decryption::<BLSGt>();
        decryption::<JubjubPoint>();
    }

    #[test]
    fn to_json() {
        serialize_to_json::<RistrettoPoint>();
        serialize_to_json::<BLSG1>();
        serialize_to_json::<BLSG2>();
        serialize_to_json::<JubjubPoint>();
    }

    #[test]
    fn to_message_pack() {
        serialize_to_message_pack::<RistrettoPoint>();
        serialize_to_message_pack::<BLSG1>();
        serialize_to_message_pack::<BLSG2>();
        serialize_to_message_pack::<JubjubPoint>();
    }
}