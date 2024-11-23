use std::ops::Mul;

use crate::linear_algebra::Vector;
use crate::ring::Ring;
use crate::traits::{FromRandomBytes, WithConjugationAutomorphism, WithL2Norm, WithLinfNorm};

pub trait PolyRing:
    Ring
    + Mul<Self::BaseRing, Output = Self>
    + From<Vec<Self::BaseRing>>
    + WithConjugationAutomorphism
    + WithL2Norm
    + WithLinfNorm
    + FromRandomBytes<Self>

    + From<Self::BaseRing>
{
    type BaseRing: Ring;

    fn coeffs(&self) -> Vec<Self::BaseRing>;
    fn flattened(vec: &Vector<Self>) -> Vector<Self::BaseRing> {
        Self::flattened_coeffs(vec).into()
    }
    fn flattened_coeffs(vec: &Vector<Self>) -> Vec<Self::BaseRing> {
        vec.into_iter()
            .flat_map(|x| x.coeffs())
            .collect::<Vec<Self::BaseRing>>()
    }
    fn dimension() -> usize;

    fn from_scalar(scalar: Self::BaseRing) -> Self;

    #[inline]
    fn x() -> Self {
        Self::from(vec![Self::BaseRing::ZERO, Self::BaseRing::ONE])
    }
}
