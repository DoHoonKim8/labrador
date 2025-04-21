use humansize::DECIMAL;
use log::{debug, info, LevelFilter};
use nimue::IOPattern;

use lattirust_arithmetic::ring::Z2_64;
use relations::Relation;

use crate::prover::Prover;
use crate::util::{
    rand_matrix_with_bounded_column_norms, BaseRelation, Instance, LovaIOPattern, OptimizationMode,
    PublicParameters,
};
use crate::verifier::Verifier;

type F = Z2_64;

const N: usize = 1 << 17;

fn init() {
    tui_logger::init_logger(LevelFilter::Debug).unwrap();
}

#[test]
fn test() {
    init();
    const SECURITY_PARAMETER: usize = 128;
    const LOG_FIAT_SHAMIR: usize = 64;

    let pp = PublicParameters::<F>::new(
        N,
        OptimizationMode::OptimizeForSpeed,
        SECURITY_PARAMETER,
        LOG_FIAT_SHAMIR,
    );

    let witness_1 = rand_matrix_with_bounded_column_norms(
        N,
        pp.inner_security_parameter,
        pp.norm_bound as i128,
    );
    let instance_1 = Instance::new(&pp, &witness_1);
    debug!("1. generated, checking relation");
    debug_assert!(BaseRelation::is_satisfied(&pp, &instance_1, &witness_1));

    let witness_2 = rand_matrix_with_bounded_column_norms(
        N,
        pp.inner_security_parameter,
        pp.norm_bound as i128,
    );
    let instance_2 = Instance::new(&pp, &witness_2);
    debug!("2. generated, checking relation");
    debug_assert!(BaseRelation::is_satisfied(&pp, &instance_2, &witness_2));

    let io = IOPattern::new("lova").fold(&pp);

    // Prove folding
    let mut merlin = io.to_merlin();
    let new_witness = Prover::fold(&mut merlin, &pp, witness_1.clone(), witness_2.clone()).unwrap();
    let folding_proof = merlin.transcript();

    info!(
        "Actual proof size:      {}",
        humansize::format_size(folding_proof.len(), DECIMAL)
    );

    // Verify folding
    let mut arthur = io.to_arthur(folding_proof);
    let new_instance = Verifier::fold(&mut arthur, &pp, instance_1, instance_2).unwrap();

    // Check that the folded instance and witness are in the relation
    assert!(BaseRelation::is_satisfied(&pp, &new_instance, &new_witness));
}
