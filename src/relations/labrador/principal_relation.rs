#![allow(non_snake_case)]

use ark_std::UniformRand;
use rand::thread_rng;
use serde::Serialize;
use crate::labrador::prover::Witness;
use crate::labrador::setup::CommonReferenceString;
use crate::labrador::util::inner_products;

use crate::lattice_arithmetic::matrix::{Matrix, sample_uniform_mat_symmetric, sample_uniform_vec, Vector};
use crate::lattice_arithmetic::poly_ring::PolyRing;

#[derive(Clone, Serialize)]
pub struct QuadDotProdFunction<R: PolyRing> {
    // TODO: A is always symmetric, so we could at least use a symmetric matrix type. A is also very sparse in some cases.
    pub A: Option<Matrix<R>>,
    // TODO: phi can be quite sparse
    pub phi: Vec<Vector<R>>,
    pub b: R,
    _private: (), // Forbid direct initialization, force users to use new(), which does some basis debug_asserts
}

impl<R: PolyRing> QuadDotProdFunction<R> {
    pub fn new(A: Matrix<R>, phi: Vec<Vector<R>>, b: R) -> Self {
        let (r, n) = (A.nrows(), phi[0].len());
        debug_assert_eq!(A.ncols(), r, "A should be square");
        debug_assert_eq!(A.transpose(), A, "A should be symmetric");

        debug_assert_eq!(phi.len(), r, "phi should have the same length as the dimensions of A");
        debug_assert!(phi.iter().all(|phi_i| phi_i.len() == n), "each phi_i should have the same length");
        Self { A: Some(A), phi, b, _private: () }
    }

    pub fn new_linear(phi: Vec<Vector<R>>, b: R) -> Self {
        let n = phi[0].len();
        debug_assert!(phi.iter().all(|phi_i| phi_i.len() == n), "each phi_i should have the same length");
        Self { A: None, phi, b, _private: () }
    }

    pub fn new_dummy(r: usize, n: usize) -> Self {
        Self::new(sample_uniform_mat_symmetric(r, r), vec![sample_uniform_vec(n); r], R::rand(&mut thread_rng()))
    }

    pub fn new_empty(r: usize, n: usize) -> Self {
        Self::new(Matrix::<R>::zeros(r, r), vec![Vector::<R>::zeros(n); r], R::zero())
    }

    pub fn is_valid_witness(&self, witness: &Witness<R>) -> bool {
        let inner_prods = inner_products(&witness.s);

        let mut res = R::zero();
        if let Some(A) = &self.A {
            let r = A.nrows();
            for i in 0..r {
                for j in 0..i + 1 {
                    res += A[(i, j)] * inner_prods[i][j];
                }
                for j in i + 1..r {
                    res += A[(i, j)] * inner_prods[j][i];
                }
            }
        }

        for i in 0..self.phi.len() {
            res += self.phi[i].dot(&witness.s[i]);
        }

        res == self.b
    }
}

#[derive(Clone, Serialize)]
pub struct ConstantQuadDotProdFunction<R: PolyRing> {
    pub A: Option<Matrix<R>>,
    pub phi: Vec<Vector<R>>,
    pub b: R::BaseRing,
    _private: (), // Forbid direct initialization, force users to use new(), which does some basis debug_asserts
}

impl<R: PolyRing> ConstantQuadDotProdFunction<R> {
    pub fn new(A: Matrix<R>, phi: Vec<Vector<R>>, b: R::BaseRing) -> Self {
        let (r, n) = (A.nrows(), phi[0].len());
        debug_assert_eq!(A.ncols(), r, "A should be square");
        debug_assert_eq!(A.transpose(), A, "A should be symmetric");

        debug_assert_eq!(phi.len(), r, "phi should have the same length as the dimensions of A");
        debug_assert!(phi.iter().all(|phi_i| phi_i.len() == n), "each phi_i should have the same length");
        Self { A: Some(A), phi, b, _private: () }
    }

    pub fn new_linear(phi: Vec<Vector<R>>, b: R::BaseRing) -> Self {
        let n = phi[0].len();
        debug_assert!(phi.iter().all(|phi_i| phi_i.len() == n), "each phi_i should have the same length");
        Self { A: None, phi, b, _private: () }
    }

    pub fn new_dummy(r: usize, n: usize) -> Self {
        Self::new(sample_uniform_mat_symmetric(r, r), vec![sample_uniform_vec(n); r], R::BaseRing::rand(&mut thread_rng()))
    }

    pub fn is_valid_witness(&self, witness: &Witness<R>) -> bool {
        let inner_prods = inner_products(&witness.s);

        let mut res = R::zero();
        if let Some(A) = &self.A {
            let r = A.nrows();
            for i in 0..r {
                for j in 0..i + 1 {
                    res += A[(i, j)] * inner_prods[i][j];
                }
                for j in i + 1..r {
                    res += A[(i, j)] * inner_prods[j][i];
                }
            }
        }

        for i in 0..self.phi.len() {
            res += self.phi[i].dot(&witness.s[i]);
        }

        res.coeffs()[0] == self.b
    }
}

#[derive(Clone, Serialize)]
pub struct PrincipalRelation<R: PolyRing> {
    pub quad_dot_prod_funcs: Vec<QuadDotProdFunction<R>>,
    pub ct_quad_dot_prod_funcs: Vec<ConstantQuadDotProdFunction<R>>,
}

impl<R: PolyRing> PrincipalRelation<R> {
    pub fn new_empty(crs: &CommonReferenceString<R>) -> Self {
        Self {
            quad_dot_prod_funcs: vec![QuadDotProdFunction::new_dummy(crs.r, crs.n); crs.num_constraints],
            ct_quad_dot_prod_funcs: vec![ConstantQuadDotProdFunction::new_dummy(crs.r, crs.n); crs.num_constraints],
        }
    }

    pub fn new_dummy(r: usize, n: usize, norm_bound: f64, num_constraints: usize, num_ct_constraints: usize) -> PrincipalRelation<R> {
        Self {
            quad_dot_prod_funcs: vec![QuadDotProdFunction::new_dummy(r, n); num_constraints],
            ct_quad_dot_prod_funcs: vec![ConstantQuadDotProdFunction::new_dummy(r, n); num_ct_constraints],
        }
    }

    pub fn is_valid_witness(&self, witness: &Witness<R>) -> bool {
        self.quad_dot_prod_funcs.iter().all(|c| c.is_valid_witness(&witness)) &&
            self.ct_quad_dot_prod_funcs.iter().all(|c| c.is_valid_witness(&witness))
    }
}