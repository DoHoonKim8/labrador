use std::mem::{self, MaybeUninit};

use crate::prover::c_prover::{
    labrador24_commitment, labrador24_composite, labrador24_free_comkey, labrador24_free_commitment, labrador24_free_composite, labrador24_free_smplstmnt, labrador24_free_witness, labrador24_init_smplstmnt_raw, labrador24_init_witness_raw, labrador24_smplstmnt, labrador24_witness
};

pub(crate) struct ProofStatement {
    cur_witness_num: usize,
    cur_constraint_num: usize,
    witness_polys: usize,
    witness_ptr: Box<labrador24_witness>,
    smplstmnt_ptr: Box<labrador24_smplstmnt>,
    commitment_ptr: Box<labrador24_commitment>,
    composite_ptr: Box<labrador24_composite>,
}

impl ProofStatement {
    pub(crate) fn new(
        deg_list: &[usize],
        num_pols_list: &[usize],
        norm_list: &[f64],
        num_constraints: usize,
    ) -> Self {
        unsafe {
            assert_eq!(deg_list.len(), num_pols_list.len());
            assert_eq!(num_pols_list.len(), norm_list.len());
            // 2) Allocate & initialize the raw C witness & statement.
            // This corresponds to initializing `proof_statement` instance in LaZer
            let mut witness_ptr: Box<labrador24_witness> = Box::new(mem::zeroed());
            let mut smplstmnt_ptr: Box<labrador24_smplstmnt> = Box::new(mem::zeroed());
            let commitment_ptr: Box<labrador24_commitment> =
                Box::new(mem::zeroed());
            let composite_ptr: Box<labrador24_composite> = Box::new(mem::zeroed());

            let mut dim_ar: Vec<usize> = deg_list
                .iter()
                .zip(num_pols_list.iter())
                .map(|(&deg, &npol)| (npol * deg) / 64)
                .collect();

            labrador24_init_witness_raw(
                &mut *witness_ptr,
                num_pols_list.len(),
                dim_ar.as_mut_ptr(),
            );

            let mut norms_ar: Vec<u64> = vec![0; norm_list.len()];
            labrador24_init_smplstmnt_raw(
                &mut *smplstmnt_ptr,
                num_pols_list.len(),
                dim_ar.as_mut_ptr(),
                norms_ar.as_mut_ptr(),
                num_constraints,
            );

            ProofStatement {
                witness_ptr,
                smplstmnt_ptr,
                commitment_ptr,
                composite_ptr,
                cur_witness_num: 0,
                cur_constraint_num: 0,
                witness_polys: 0,
            }
        }
    }
}

impl Drop for ProofStatement {
    fn drop(&mut self) {
        unsafe {
            labrador24_free_commitment(&mut *self.commitment_ptr);
            labrador24_free_witness(&mut *self.witness_ptr);
            labrador24_free_composite(&mut *self.composite_ptr);
            labrador24_free_smplstmnt(&mut *self.smplstmnt_ptr);
            labrador24_free_comkey();
        }
    }
}
