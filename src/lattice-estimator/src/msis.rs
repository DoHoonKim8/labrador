use std::fmt;
use std::fmt::{Debug, Display};

use crate::errors::LatticeEstimatorError;
use crate::norms::Norm;
use crate::sis::SIS;

/// An MSIS instance is a matrix A in R_q^{m x n}, R_q = Z_q[X]/(X^d+1) such that A * s = 0 for some s in R_q^n with ||s||_norm <= length_bound.
pub struct MSIS {
    pub n: usize,
    pub d: usize,
    pub q: u64,
    pub length_bound: f64,
    pub m: usize,
    pub norm: Norm,
}

impl Display for MSIS {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "MSIS_{{n={}, d={}, q={}, length_bound={}, m={}, norm={}}}", self.n, self.d, self.q, self.length_bound, self.m, self.norm)
    }
}

impl Debug for MSIS {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "MSIS_{{n={}, d={}, q={}, length_bound={}, m={}, norm={}}}", self.n, self.d, self.q, self.length_bound, self.m, self.norm)
    }
}

impl MSIS {
    pub fn with_n(&self, n: usize) -> Self {
        MSIS { n, d: self.d, q: self.q, length_bound: self.length_bound, m: self.m, norm: self.norm }
    }

    pub const fn with_length_bound(&self, length_bound: f64) -> Self {
        MSIS { n: self.n, d: self.d, q: self.q, length_bound, m: self.m, norm: self.norm }
    }

    pub fn to_sis(&self) -> SIS {
        SIS::new(self.n * self.d, self.q, self.length_bound, self.m * self.d, self.norm)
    }

    /// Return lambda such that MSIS_{n, d, q, length_bound, m} is 2^lambda-hard (for a given norm).
    /// We estimate the security by reducing to SIS_{n*d, q, length_bound, m*d} and calling the SIS security estimator.
    pub fn security_level(&self) -> f64 {
        self.to_sis().security_level()
    }

    /// Return the smallest m such that MSIS_{n, d, q, length_bound, m} is 2^lambda-hard (for a given norm).
    pub fn find_optimal_n(&self, lambda: usize) -> Result<usize, LatticeEstimatorError> {
        let mut hi: usize = self.m; // (m as f64 / (q as f64).log2()).floor() as usize;
        let mut lo: usize = 1;

        let msis = self.with_n(hi);
        let lambda_hi = msis.security_level();
        debug_assert!(lambda_hi >= lambda as f64, "{msis} has sec. param. {lambda_hi}  < target lambda = {lambda}");
        // Loop invariant: SIS_{hi, q, length_bound, m} is 2^lambda_hi-hard with lambda_hi >= lambda
        while hi > lo {
            let mid = lo + (hi - lo) / 2;
            let msis = self.with_n(mid);
            let mid_lambda = msis.security_level();
            if mid_lambda >= lambda as f64 { // Search for smaller n in [lo, mid]
                hi = mid;
            } else { // Search for smaller n in [mid+1, hi]
                lo = mid + 1;
            }
        }
        assert_eq!(hi, lo);
        Ok(hi)
    }

    /// Return the smallest n such that MSIS_{n, d, q, length_bound(n), m} is 2^lambda-hard (for a given norm), where length_bound is a function of n.
    pub fn find_optimal_n_dynamic<F>(&self, length_bound: F, lambda: usize) -> Result<usize, LatticeEstimatorError>
        where F: Fn(usize) -> f64
    {
        // We can't assume that length_bound is monotonic, so we can't use binary search.
        // Instead, exhaustively search powers of 2 until we find a suitable n.
        // TODO: use a better search algorithm / return a more fine-grained result
        let log2_m = self.m.ilog2();
        let candidates = (1..log2_m).map(|i| 2usize.pow(i));
        candidates.map(|n| self.with_n(n).with_length_bound(length_bound(n)))
            .find(|sis| sis.security_level() >= lambda as f64)
            .map(|sis| sis.n).ok_or(LatticeEstimatorError::from("no suitable n found".to_string()))
    }
}
