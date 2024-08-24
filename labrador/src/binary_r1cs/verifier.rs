#![allow(non_snake_case)]

use ark_relations::r1cs::ConstraintSystemRef;
use log::debug;
use nimue::{Arthur, ProofError, ProofResult};

use lattirust_arithmetic::challenge_set::labrador_challenge_set::LabradorChallengeSet;
use lattirust_arithmetic::challenge_set::weighted_ternary::WeightedTernaryChallengeSet;
use lattirust_arithmetic::nimue::arthur::SerArthur;
use lattirust_arithmetic::nimue::traits::ChallengeFromRandomBytes;
use lattirust_arithmetic::ring::{PolyRing, UnsignedRepresentative};
use lattirust_arithmetic::traits::FromRandomBytes;
use lattirust_util::{check, check_eq};
use relations::principal_relation::PrincipalRelation;

use crate::binary_r1cs::util::{reduce, BinaryR1CSCRS, BinaryR1CSTranscript, Z2};
use crate::util::ark_sparse_matrices;
use crate::verifier::verify_principal_relation;

pub fn verify_reduction_binaryr1cs_labradorpr<R: PolyRing>(
    arthur: &mut Arthur,
    cs: &ConstraintSystemRef<Z2>,
    crs: &BinaryR1CSCRS<R>,
) -> ProofResult<PrincipalRelation<R>> {
    let (A, B, C) = ark_sparse_matrices(cs);

    let d = R::dimension();
    let (k, n) = (
        cs.num_constraints(),
        cs.num_instance_variables() + cs.num_witness_variables(),
    );

    let t = arthur.next_vector(crs.m.div_ceil(d))?;

    let alpha = arthur.challenge_binary_matrix(crs.security_parameter, k)?;
    let beta = arthur.challenge_binary_matrix(crs.security_parameter, n)?;
    let gamma = arthur.challenge_binary_matrix(crs.security_parameter, n)?;

    // delta_i is computed mod 2, i.e., over Z2
    let delta = &alpha * &A + &beta * &B + &gamma * &C;

    let g = arthur.next_vector_canonical::<R::BaseRing>(crs.security_parameter)?;

    for i in 0..g.len() {
        // Check that all g_i's are even
        check_eq!(Into::<UnsignedRepresentative>::into(g[i]).0 % 2, 0);
    }

    let transcript = BinaryR1CSTranscript {
        t,
        alpha,
        beta,
        gamma,
        g,
        delta,
    };
    let instance_pr = reduce(crs, cs, &transcript);
    Ok(instance_pr)
}

pub fn verify_binary_r1cs<R: PolyRing>(
    arthur: &mut Arthur,
    cs: &ConstraintSystemRef<Z2>,
    crs: &BinaryR1CSCRS<R>,
) -> Result<(), ProofError>
where
    LabradorChallengeSet<R>: FromRandomBytes<R>,
    WeightedTernaryChallengeSet<R>: FromRandomBytes<R>,
{
    //TODO: add crs and statement to transcript
    let instance_pr = verify_reduction_binaryr1cs_labradorpr(arthur, cs, crs)?;

    arthur.ratchet()?;

    verify_principal_relation(arthur, &instance_pr, &crs.core_crs)
}
