use algebra::{biginteger::BigInteger256 as BigInteger, field_new, Field, ProjectiveCurve};

use primitives::{
    crh::pedersen::PedersenWindow, signature::schnorr::field_based_schnorr::FieldBasedSchnorrPk,
};

use crate::type_mapping::*;

pub mod constants;

pub struct NaiveThresholdSigParams {
    pub null_sig: SchnorrSig,
    pub null_pk: FieldBasedSchnorrPk<G2Projective>,
}

impl NaiveThresholdSigParams {
    pub fn new() -> Self {
        let e = FieldElement::one();
        let s = e;
        let null_sig = SchnorrSig::new(e, s);

        let x = field_new!(
            FieldElement,
            BigInteger([
                817531083298639342,
                16113810631348879462,
                1306238005170570794,
                917352325188691328,
            ],)
        );

        let y = field_new!(
            FieldElement,
            BigInteger([
                7051092939593429249,
                9720513155186830666,
                6359574609400546156,
                2888851165431378812,
            ],)
        );

        let z = field_new!(
            FieldElement,
            BigInteger([
                2035294266095304701,
                17681163514934325971,
                18446744073709551615,
                4611686018427387903,
            ],)
        );

        let null_pk = FieldBasedSchnorrPk(G2Projective::new(x, y, z));

        Self { null_sig, null_pk }
    }
}

#[derive(Clone)]
pub struct VRFWindow {}
impl PedersenWindow for VRFWindow {
    const WINDOW_SIZE: usize = 128;
    const NUM_WINDOWS: usize = 2;
}

pub struct VRFParams {
    pub group_hash_generators: Vec<Vec<G2Projective>>,
}

impl VRFParams {
    pub fn new() -> Self {
        let gen_1 = G2Projective::new(
            field_new!(
                FieldElement,
                BigInteger([
                    12926485790763496744,
                    7301812230106132899,
                    5868404524855748477,
                    606499321461550871,
                ])
            ),
            field_new!(
                FieldElement,
                BigInteger([
                    12459762756615730720,
                    7373659181971905397,
                    417161890334020390,
                    1065371458433835676,
                ])
            ),
            field_new!(
                FieldElement,
                BigInteger([
                    2035294266095304701,
                    17681163514934325971,
                    18446744073709551615,
                    4611686018427387903,
                ])
            ),
        );

        let gen_2 = G2Projective::new(
            field_new!(
                FieldElement,
                BigInteger([
                    15342121784330514541,
                    9118652640516123238,
                    17509069746383483632,
                    1285742610361624126,
                ])
            ),
            field_new!(
                FieldElement,
                BigInteger([
                    5093526993895361515,
                    7436829771986140347,
                    8066708127376422025,
                    1382517365996680842,
                ])
            ),
            field_new!(
                FieldElement,
                BigInteger([
                    2035294266095304701,
                    17681163514934325971,
                    18446744073709551615,
                    4611686018427387903,
                ])
            ),
        );

        let group_hash_generators = Self::compute_group_hash_table([gen_1, gen_2].to_vec());

        Self {
            group_hash_generators,
        }
    }

    pub(crate) fn compute_group_hash_table(
        generators: Vec<G2Projective>,
    ) -> Vec<Vec<G2Projective>> {
        let mut gen_table = Vec::new();
        for generator in generators.iter().take(VRFWindow::NUM_WINDOWS) {
            let mut generators_for_segment = Vec::new();
            let mut base = *generator;
            for _ in 0..VRFWindow::WINDOW_SIZE {
                generators_for_segment.push(base);
                for _ in 0..4 {
                    base.double_in_place();
                }
            }
            gen_table.push(generators_for_segment);
        }
        gen_table
    }
}

//TODO: Move these constants in cctp-lib
pub const SC_PUBLIC_KEY_LENGTH: usize = 32;
pub const SC_SECRET_KEY_LENGTH: usize = 32;
pub const SC_TX_HASH_LENGTH: usize = 32;
pub const SC_CUSTOM_HASH_LENGTH: usize = 32;
pub const MC_RETURN_ADDRESS_BYTES: usize = 20;
pub const FIELD_MODULUS: usize = FIELD_CAPACITY + 1;

pub const MST_MERKLE_TREE_HEIGHT: usize = 22;

pub const PHANTOM_FIELD_ELEMENT: FieldElement = field_new!(
    FieldElement,
    BigInteger256([
        4113438167814256341,
        6017662335620633663,
        3729794390834568355,
        4611686018427387136
    ])
);

pub const CSW_PHANTOM_PUB_KEY_BYTES: [u8; 32] = [217, 127, 224, 199, 8, 45, 179, 51, 115, 161, 177, 30, 203, 183, 46, 176, 168, 185, 222, 243, 130, 216, 130, 102, 88, 154, 253, 135, 199, 233, 73, 48];

#[cfg(test)]
mod test {
    use crate::read_field_element_from_buffer_with_padding;
    use algebra::{AffineCurve, FpParameters, FromCompressedBits, PrimeField};
    use cctp_primitives::utils::serialization::serialize_to_buffer;

    use super::*;
    use bit_vec::BitVec;
    use blake2s_simd::{Hash, Params};

    use serial_test::*;

    fn hash_to_curve<F: PrimeField, G: AffineCurve + FromCompressedBits>(
        tag: &[u8],
        personalization: &[u8],
    ) -> Option<G> {
        let compute_chunk = |input: &[u8], personalization: &[u8]| -> Hash {
            Params::new()
                .hash_length(32)
                .personal(personalization)
                .to_state()
                .update(constants::GH_FIRST_BLOCK)
                .update(input)
                .finalize()
        };

        // Append counter byte to tag
        let tag_len = tag.len();
        let mut tag = tag.to_vec();
        tag.push(0u8);

        // Compute number of hashes to be concatenated in order to obtain a field element
        let field_size = F::size_in_bits();
        let bigint_size = (field_size + F::Params::REPR_SHAVE_BITS as usize) / 8;
        let chunk_num = if bigint_size % 32 == 0 {
            bigint_size / 32
        } else {
            (bigint_size / 32) + 1
        };
        let max_value = u8::max_value();
        let mut g = None;

        while tag[tag_len] <= max_value {
            let mut chunks = vec![];

            //chunk_0 = H(tag), chunk_1 = H(chunk_0) = H(H(tag)), ..., chunk_i = H(chunk_i-1)
            let mut prev_hash = tag.clone();
            for _ in 0..chunk_num {
                let hash = compute_chunk(prev_hash.as_slice(), personalization);
                chunks.extend_from_slice(hash.as_ref());
                prev_hash = hash.as_ref().to_vec();
            }

            tag[tag_len] += 1u8;

            //Mask away REPR_SHAVE_BITS
            let mut chunk_bits = BitVec::from_bytes(chunks.as_slice());
            for i in field_size..(bigint_size * 8) {
                chunk_bits.set(i, false);
            }

            //Get field element from `chunks`
            let chunk_bytes = chunk_bits.to_bytes();
            let fe = match F::from_random_bytes(&chunk_bytes[..bigint_size]) {
                Some(fe) => fe,
                None => continue,
            };

            //Get point from chunks
            let mut fe_bits = fe.write_bits();
            fe_bits.push(false); //We don't want an infinity point
            fe_bits.push(false); //We decide to choose the even y coordinate
            match G::decompress(fe_bits) {
                Ok(point) => {
                    g = Some(point);
                    break;
                }
                Err(_) => continue,
            };
        }
        g
    }

    #[serial]
    #[test]
    fn test_pk_null_gen() {
        let tag = b"Strontium Sr 90";
        let personalization = constants::NULL_PK_PERSONALIZATION;
        let htc_out = hash_to_curve::<FieldElement, G2>(tag, personalization)
            .unwrap()
            .into_projective();
        println!("{:#?}", htc_out);
        let null_pk = NaiveThresholdSigParams::new().null_pk.0;
        assert_eq!(htc_out, null_pk);
    }

    #[serial]
    #[test]
    fn test_vrf_group_hash_gen() {
        let personalization = constants::VRF_GROUP_HASH_GENERATORS_PERSONALIZATION;

        //Gen1
        let tag = b"Magnesium Mg 12";
        let htc_g1_out = hash_to_curve::<FieldElement, G2>(tag, personalization)
            .unwrap()
            .into_projective();

        //Gen2
        let tag = b"Gold Au 79";
        let htc_g2_out = hash_to_curve::<FieldElement, G2>(tag, personalization)
            .unwrap()
            .into_projective();

        //Check GH generators
        let gh_generators = VRFParams::compute_group_hash_table([htc_g1_out, htc_g2_out].to_vec());
        println!("{:#?}", htc_g1_out);
        println!("{:#?}", htc_g2_out);
        assert_eq!(gh_generators, VRFParams::new().group_hash_generators);
    }

    #[serial]
    #[test]
    fn test_csw_phantom_field_element() {
        let tag = b"Krypton 36";
        let field_element = read_field_element_from_buffer_with_padding(tag).unwrap();
        println!("Phantom field element: {:?}", field_element);
        assert_eq!(field_element, PHANTOM_FIELD_ELEMENT);
    }

    #[serial]
    #[test]
    fn test_csw_phantom_public_key() {
        let x = SimulatedFieldElement::from(BigInteger256([15877199453377760308, 458271239891050623, 5539075294202620951, 5404726604943382876]));
        let y = SimulatedFieldElement::from(BigInteger256([3725370832501899225, 12695286482625208691, 7386704395789842856, 3479569230309726808]));
        let simulated_te_point = SimulatedTEGroup::new(x, y);
        assert_eq!(simulated_te_point.group_membership_test(), false);

        // Store the sign (last bit) of the X coordinate
        // The value is left-shifted to be used later in an OR operation
        let x_sign = if simulated_te_point.x.is_odd() { 1 << 7 } else { 0u8 };

        // Extract the public key bytes as Y coordinate
        let y_coordinate = simulated_te_point.y;
        let mut pk_bytes = serialize_to_buffer(&y_coordinate, None).unwrap();

        // Use the last (null) bit of the public key to store the sign of the X coordinate
        // Before this operation, the last bit of the public key (Y coordinate) is always 0 due to the field modulus
        let len = pk_bytes.len();
        pk_bytes[len - 1] |= x_sign;

        assert_eq!(pk_bytes, CSW_PHANTOM_PUB_KEY_BYTES);
    }
}
