use ark_std::test_rng;
use nimue::IOPattern;
use pretty_env_logger::env_logger;

use lattirust_arithmetic::ring::Z2_64;
use relations::traits::Relation;

use crate::prover::Prover;
use crate::util::{BaseRelation, Instance, LovaIOPattern, OptimizationMode, PublicParameters, rand_matrix_with_bounded_column_norms};
use crate::verifier::Verifier;

type F = Z2_64;

const N: usize = 1 << 20;

fn init() {
    let _ = env_logger::builder().is_test(true).try_init();
}

#[test]
fn test() {
    init();

    let rng = &mut test_rng();
    let pp = PublicParameters::<F>::new(N, OptimizationMode::OptimizeForSize );

    let witness_1 =
        rand_matrix_with_bounded_column_norms(N, pp.security_parameter, rng, pp.norm_bound as i128);
    let instance_1 = Instance::new(&pp, &witness_1);
    debug_assert!(BaseRelation::is_satisfied(&pp, &instance_1, &witness_1));

    let witness_2 =
        rand_matrix_with_bounded_column_norms(N, pp.security_parameter, rng, pp.norm_bound as i128);
    let instance_2 = Instance::new(&pp, &witness_2);
    debug_assert!(BaseRelation::is_satisfied(&pp, &instance_2, &witness_2));

    let io = IOPattern::new("lova").fold(&pp);

    // Prove folding
    let mut arthur = io.to_arthur();
    let new_witness = Prover::fold(&mut arthur, &pp, witness_1.clone(), witness_2.clone()).unwrap();
    let folding_proof = arthur.transcript();

    // Verify folding
    let mut merlin = io.to_merlin(folding_proof);
    let new_instance = Verifier::fold(&mut merlin, &pp, instance_1, instance_2).unwrap();

    // Check that the folded instance and witness are in the relation
    assert!(BaseRelation::is_satisfied(&pp, &new_instance, &new_witness));
}
