use num_traits::{One, Zero};

use crate::pow2_cyclotomic_poly_ring::Pow2CyclotomicPolyRing;
use crate::pow2_cyclotomic_poly_ring_ntt::Pow2CyclotomicPolyRingNTT;
use crate::ring::Fq;
use crate::traits::FromRandomBytes;

/// Challenge set {-1, 0, 1} for Fq, where Pr[0] = 1/2, Pr[1] = Pr[-1] = 1/4
pub struct WeightedTernaryChallengeSet<R> {
    _marker: std::marker::PhantomData<R>,
}

impl<const Q: u64> FromRandomBytes<Fq<Q>> for WeightedTernaryChallengeSet<Fq<Q>> {
    fn byte_size() -> usize {
        1
    }

    fn try_from_random_bytes(bytes: &[u8]) -> Option<Fq<Q>> {
        assert_eq!(bytes.len(), 1);
        let val = bytes[0] & 0b11; // Technically a u4 now
        if val == 0 || val == 3 {
            Some(Fq::<Q>::zero())
        } else if val == 1 {
            Some(Fq::<Q>::one())
        } else if val == 2 {
            Some(-Fq::<Q>::one())
        } else {
            unreachable!()
        }
    }
}

impl<const Q: u64, const N: usize> FromRandomBytes<Pow2CyclotomicPolyRing<Fq<Q>, N>> for WeightedTernaryChallengeSet<Pow2CyclotomicPolyRing<Fq<Q>, N>> {
    fn byte_size() -> usize {
        N * WeightedTernaryChallengeSet::<Fq<Q>>::byte_size()
    }

    fn try_from_random_bytes(bytes: &[u8]) -> Option<Pow2CyclotomicPolyRing<Fq<Q>, N>> {
        assert_eq!(bytes.len(), Self::byte_size());
        let b = WeightedTernaryChallengeSet::<Fq<Q>>::byte_size();
        Some(
            Pow2CyclotomicPolyRing::<Fq<Q>, N>::from_fn(|i|
                WeightedTernaryChallengeSet::<Fq<Q>>::try_from_random_bytes(&bytes[i * b..(i + 1) * b]).unwrap()
            )
        )
    }
}

impl<const Q: u64, const N: usize> FromRandomBytes<Pow2CyclotomicPolyRingNTT<Q, N>> for WeightedTernaryChallengeSet<Pow2CyclotomicPolyRingNTT<Q, N>> {
    fn byte_size() -> usize { Pow2CyclotomicPolyRing::<Fq<Q>, N>::byte_size() }

    fn try_from_random_bytes(bytes: &[u8]) -> Option<Pow2CyclotomicPolyRingNTT<Q, N>> { Pow2CyclotomicPolyRing::<Fq<Q>, N>::try_from_random_bytes(bytes).map(|x| x.into()) }
}