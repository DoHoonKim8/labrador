use ark_serialize::{CanonicalSerialize, Compress};
use bincode;
use crypto_bigint::rand_core::{CryptoRng, RngCore};
use nalgebra::Scalar;
use nimue::{Arthur, ByteWriter, DefaultHash, DefaultRng, DuplexHash, IOPatternError};
use serde::Serialize;

use crate::lattice_arithmetic::matrix::{Matrix, SymmetricMatrix, Vector};

pub trait SerArthur<H = DefaultHash, R = DefaultRng>
    where
        H: DuplexHash<u8>,
        R: RngCore + CryptoRng,
        Self: ByteWriter
{
    fn absorb_serializable<S: serde::Serialize>(&mut self, msg: &S) -> Result<(), IOPatternError> {
        match bincode::serialize(&msg) {
            Ok(bytes) => self.add_bytes(bytes.as_slice()),
            Err(e) => Err(IOPatternError::from(e.to_string()))
        }
    }

    fn absorb_canonical_serializable<S: CanonicalSerialize>(&mut self, msg: &S) -> Result<(), IOPatternError> {
        let mut bytes = vec![];
        match msg.serialize_compressed(&mut bytes) {
            Ok(()) => self.add_bytes(bytes.as_slice()),
            Err(e) => Err(IOPatternError::from(e.to_string()))
        }
    }

    fn absorb_vector<F: CanonicalSerialize>(&mut self, vec: &Vector<F>) -> Result<(), IOPatternError> {
        for elem in vec.iter() {
            self.absorb_canonical_serializable(elem)?;
        }
        Ok(())
    }

    fn absorb_symmetric_matrix<F: Clone>(&mut self, mat: &SymmetricMatrix<F>) -> Result<(), IOPatternError>
        where SymmetricMatrix<F>: Serialize
    {
        self.absorb_serializable(&mat)
    }

    fn absorb_matrix<F: CanonicalSerialize>(&mut self, mat: &Matrix<F>) -> Result<(), IOPatternError> {
        for row in mat.row_iter() {
            for elem in row.iter() {
                self.absorb_canonical_serializable(elem)?;
            }
        };
        Ok(())
    }
}

impl<H: DuplexHash<u8>, R: RngCore + CryptoRng> SerArthur<H, R> for Arthur<H, u8, R> {}
