#[cfg(feature = "c-binding")]
use std::fmt::Debug;

#[cfg(feature = "c-binding")]
use lattirust_arithmetic::{
    challenge_set::{
        labrador_challenge_set::LabradorChallengeSet, weighted_ternary::WeightedTernaryChallengeSet,
    },
    decomposition::DecompositionFriendlySignedRepresentative,
    ring::{representatives::WithSignedRepresentative, PolyRing},
    traits::FromRandomBytes,
};
#[cfg(feature = "c-binding")]
use nimue::{Merlin, ProofResult};
#[cfg(feature = "c-binding")]
use relations::principal_relation::{Index, Instance, Witness};

#[cfg(feature = "c-binding")]
use crate::common_reference_string::CommonReferenceString;

pub mod types;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

#[cfg(feature = "c-binding")]
pub fn prove_principal_relation<'a, R: PolyRing>(
    _merlin: &'a mut Merlin,
    mut crs: &CommonReferenceString<R>,
    index: &Index<R>,
    instance: &Instance<R>,
    witness: &Witness<R>,
) -> ProofResult<&'a [u8]>
where
    LabradorChallengeSet<R>: FromRandomBytes<R>,
    WeightedTernaryChallengeSet<R>: FromRandomBytes<R>,
    <R as PolyRing>::BaseRing: WithSignedRepresentative,
    <R::BaseRing as WithSignedRepresentative>::SignedRepresentative:
        DecompositionFriendlySignedRepresentative,
    <R as TryFrom<u128>>::Error: Debug,
{
    use std::mem::MaybeUninit;
    use std::slice;

    unsafe {
        // 1) Initialize the commitment key (comkey) from the CRS size.
        //    In Lazer this is: lib.labrador32_init_comkey(n)

        use crate::prover::c_prover::types::ProofStatement;
        labrador24_init_comkey(crs.max_witness_polys()); // ❓ exact `n` needs confirmation :contentReference[oaicite:9]{index=9}

        let ps = ProofStatement::new(deg_list, num_pols_list, norm_list, num_constraints);

        // 3) Populate each witness vector into the C struct.
        for (i, vec) in witness.vectors().enumerate() {
            let coeffs: Vec<i64> = vec.to_int64_array();
            let ret = labrador24_set_witness_vector_raw(
                c_witness.as_mut_ptr(),
                i,
                vec.num_polys(),
                vec.len_in_blocks(),
                coeffs.as_ptr(),
            ); // :contentReference[oaicite:12]{index=12}
            assert!(ret == 0, "setting witness vector failed");
        }

        // 4) Populate each linear constraint into the C stmt.
        for (i, cnst) in index.constraints().enumerate() {
            let a_arr: Vec<i64> = cnst.a.to_int64_array();
            let b_arr: Vec<i64> = cnst.b.to_int64_array();
            let ret = labrador24_set_smplstmnt_lincnst_raw(
                c_statement.as_mut_ptr(),
                i,
                cnst.num_terms(),
                cnst.witness_indices().as_ptr(),
                cnst.term_dims().as_ptr(),
                cnst.poly_deg_in_blocks(),
                a_arr.as_ptr(),
                b_arr.as_ptr(),
            );
            assert!(ret == 0, "setting constraint failed");
        }

        // 5) Call into the C prover.
        let mut c_prf = MaybeUninit::<proof>::zeroed();
        let mut c_comm = MaybeUninit::<witness>::zeroed();
        let rc = prove(
            c_statement.as_mut_ptr(),
            c_witness.as_mut_ptr(),
            c_prf.as_mut_ptr(),
            c_statement.as_ptr(),
            c_witness.as_ptr(),
            /* tail = */ 0,
        ); // :contentReference[oaicite:13]{index=13}

        if rc != 0 {
            return Err(ProofError::ProverFailed(rc));
        }

        // 6) Serialize the proof struct into a byte slice.
        //    TODO: we need a C API (e.g. `serialize_proof`) or define
        //          how `proof` should be turned into &[u8].
        //
        // For now, we simply expose the raw bytes of the `proof` struct:
        let prf_ptr = c_prf.as_ptr() as *const u8;
        let prf_len = std::mem::size_of::<proof>();
        let proof_bytes = slice::from_raw_parts(prf_ptr, prf_len);

        // 7) Clean up if needed (e.g. free_comkey, free_witness, free_statement).
        labrador24_free_comkey(); // :contentReference[oaicite:14]{index=14}

        Ok(proof_bytes)
    }
}
