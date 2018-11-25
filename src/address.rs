//Zei Network Address
//An account has an address that it is public key. 

//Create an address:
//  1. Convert PublicKey to base58, it has built in checksum
//  2. Append 'ZEI_' prefix 

//Get Publickey from Address
// 1. Remove 'ZEI_' prefix 
// 2.

use organism_utils::base58;
use schnorr::PublicKey;

//Account Address is just its encoded public key
pub type Address = String;


///Encode a Given Publickey to Zei Address
pub fn enc(pk: &PublicKey) -> Address {
    //convert to base58
    let enc: String = base58::check_encode_slice(&pk.to_bytes());
    //add prefix 
    let addy: String = "ZEI_".to_string();
    return addy + &enc;
}

///Decode a Given Zei Address to Publickey
pub fn dec(addy: &String) -> PublicKey {
    //remove prefix
    let pk = &addy[4..];
    //decode from base58
    let dec = base58::from_check(&pk).unwrap();

    return PublicKey::from_bytes(&dec).unwrap();

}


#[cfg(test)]
mod test {
    use super::*;
    use rand::ChaChaRng;
    use rand::SeedableRng;
    use schnorr::Keypair;


    #[test]
    fn test_address_encoding() {
        let mut csprng: ChaChaRng;
        csprng  = ChaChaRng::from_seed([0u8; 32]);
        let keypair: Keypair = Keypair::generate(&mut csprng);

        let enc = enc(&keypair.public);
        let dec = dec(&enc);
        assert_eq!(dec, keypair.public);
    }

}
