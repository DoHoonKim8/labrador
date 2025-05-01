use lattirust_arithmetic::{linear_algebra::Vector, ring::PolyRing};
use relations::principal_relation::{
    Index, Instance, PrincipalRelation, QuadraticConstraint, Size, Witness,
};

const FALCON_RING_MODULUS: u32 = 12289;

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
            norm_bound_sq: 0.,
            num_constraints: N,
            num_constant_constraints: 0,
        }
    }

    fn generate_falcon_verification_principal_relation(&self) -> (Index<R>, Instance<R>) {
        let size = self.size();
        let index = Index::<R>::new(&size);

        let falcon_ring_modulus = R::try_from(FALCON_RING_MODULUS).unwrap();
        let instance = Instance::<R> {
            quad_dot_prod_funcs: (0..index.num_constraints)
                .map(|i| {
                    let phi = if self.is_same_pk {
                        (0..size.num_witnesses)
                            .map(|_| {
                                Vector::<R>::from_vec(vec![
                                    R::ONE,
                                    self.public_key[0],
                                    falcon_ring_modulus,
                                ])
                            })
                            .collect()
                    } else {
                        self.public_key
                            .iter()
                            .map(|pk| Vector::<R>::from_vec(vec![R::ONE, *pk, falcon_ring_modulus]))
                            .collect()
                    };
                    // replace `b` with H(r, m)
                    let constraint = QuadraticConstraint::new_linear(phi, self.messages[i]);
                    constraint
                })
                .collect(),
            ct_quad_dot_prod_funcs: vec![],
        };

        (index, instance)
    }
}

#[cfg(test)]
mod tests {
    use super::FalconVerificationWitness;

    const NUM_SIGS: usize = 1000;
    #[test]
    fn test_falcon_aggregation_with_same_pk() {
        // Prepare `FalconVerificationWitnesses`
        let falcon_verification_witnesses = (0..NUM_SIGS).map(|_| {
            FalconVerificationWitness {
            }
        });

        // Prepare `FalconAggregator`

        // Generate PrincipalRelation for Falcon verification

        // Start Labrador proving
    }
}
