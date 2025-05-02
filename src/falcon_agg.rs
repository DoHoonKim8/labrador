use lattirust_arithmetic::{
    linear_algebra::{SymmetricMatrix, Vector},
    ring::PolyRing,
};
use num_traits::Zero;
use relations::principal_relation::{
    ConstantQuadraticConstraint, Index, Instance, PrincipalRelation, QuadraticConstraint, Size,
    Witness,
};

const FALCON_RING_MODULUS: u32 = 12289;

#[derive(Clone, Copy)]
struct FalconVerificationWitness<R: PolyRing> {
    signature: (R, R),
    v: R,
}

struct FalconAggregator<R: PolyRing, const N: usize> {
    witnesses: [FalconVerificationWitness<R>; N],
    messages: [R; N],
    is_same_pk: bool,
    public_key: Vec<R>,
}

impl<R: PolyRing, const N: usize> FalconAggregator<R, N> {
    fn new(
        witnesses: [FalconVerificationWitness<R>; N],
        messages: [R; N],
        is_same_pk: bool,
        public_key: Vec<R>,
    ) -> Self {
        if is_same_pk {
            assert_eq!(public_key.len(), 1);
        } else {
            assert_eq!(public_key.len(), N);
        }
        Self {
            witnesses,
            messages,
            is_same_pk,
            public_key,
        }
    }

    fn size(&self) -> Size {
        Size {
            num_witnesses: N,
            witness_len: 3,
            norm_bound_sq: N as f64 * 970218478., // garbage number
            num_constraints: N,
            num_constant_constraints: 1, // if this is set to 0, then the test fails, so just set to 1
        }
    }

    pub fn generate_falcon_verification_principal_relation(&self) -> (Index<R>, Instance<R>) {
        let size = self.size();
        let index = Index::<R>::new(&size);

        let falcon_ring_modulus = R::try_from(FALCON_RING_MODULUS).unwrap();
        let witness = Witness::new(
            self.witnesses
                .iter()
                .map(|w| Vector::from_vec(vec![w.signature.0, w.signature.1, w.v]))
                .collect(),
        );
        let instance = Instance::<R> {
            quad_dot_prod_funcs: (0..index.num_constraints)
                .map(|i| {
                    let mut phi = (0..size.num_witnesses)
                        .map(|_| Vector::<R>::from_element(3, R::ZERO))
                        .collect::<Vec<_>>();
                    phi[i] = if self.is_same_pk {
                        Vector::<R>::from_vec(vec![R::ONE, self.public_key[0], falcon_ring_modulus])
                    } else {
                        Vector::<R>::from_vec(vec![R::ONE, self.public_key[i], falcon_ring_modulus])
                    };
                    // replace `b` with H(r, m)
                    let constraint = QuadraticConstraint::new_linear(phi, self.messages[i]);
                    assert_eq!(R::ZERO, constraint.eval(&witness));
                    constraint
                })
                .collect(),
            ct_quad_dot_prod_funcs: vec![ConstantQuadraticConstraint::new_linear(
                (0..size.num_witnesses)
                    .map(|_| Vector::<R>::from_element(3, R::ZERO))
                    .collect::<Vec<_>>(),
                R::BaseRing::zero(),
            )],
        };

        (index, instance)
    }
}

#[cfg(test)]
mod tests {
    use std::time::Instant;

    use lattirust_arithmetic::{
        linear_algebra::Vector,
        ring::{PolyRing, Pow2CyclotomicPolyRingNTT, Ring, Zq2},
    };
    use relations::{principal_relation::Witness, reduction::Reduction};

    use crate::{
        common_reference_string::CommonReferenceString,
        prover::{prove_principal_relation, prove_principal_relation_oneround},
        test::Labrador,
        verifier::{verify_principal_relation, verify_principal_relation_oneround},
    };

    use super::{FalconAggregator, FalconVerificationWitness};

    const NUM_SIGS: usize = 100;
    const Q1: u64 = 274177;
    const Q2: u64 = 67280421310721;
    pub type Z64 = Zq2<Q1, Q2>;
    const D: usize = 64;

    type R = Pow2CyclotomicPolyRingNTT<Z64, D>;

    #[test]
    fn test_falcon_aggregation_with_same_pk() {
        // Prepare `FalconVerificationWitnesses`
        let falcon_verification_witnesses = [(); NUM_SIGS].map(|_| FalconVerificationWitness {
            signature: (R::ONE, R::ONE),
            v: R::ONE,
        });

        // garbage value
        let public_key = R::try_from(2 as u32).unwrap();

        // Prepare `FalconAggregator`
        let falcon_aggregator = FalconAggregator::<R, NUM_SIGS>::new(
            falcon_verification_witnesses,
            [R::try_from(12292 as u32).unwrap(); NUM_SIGS],
            true,
            vec![public_key],
        );

        // Generate PrincipalRelation for Falcon verification
        let (index, instance) = falcon_aggregator.generate_falcon_verification_principal_relation();

        // Start Labrador proving
        let size = falcon_aggregator.size();
        let pp = CommonReferenceString::new_for_size(size);
        let io = Labrador::iopattern(&pp, &index, &instance);

        let mut merlin = io.to_merlin();

        let witness = Witness::new(
            falcon_verification_witnesses
                .into_iter()
                .map(|w| Vector::from_vec(vec![w.signature.0, w.signature.1, w.v]))
                .collect(),
        );
        let now = Instant::now();
        prove_principal_relation_oneround(&mut merlin, &pp, &index, &instance, &witness).unwrap();
        println!("Time elapsed for proving : {:?}", now.elapsed());

        let proof = merlin.transcript();
        let mut arthur = io.to_arthur(proof);
        verify_principal_relation_oneround(&mut arthur, &pp, &index, &instance).unwrap();
        ()
    }
}
