pub(crate) mod constants;

use crate::constraint_system::{Variable, WireData};
use ark_ff::FftField;
use ark_poly::{
    domain::{EvaluationDomain, GeneralEvaluationDomain},
    univariate::DensePolynomial,
    UVPolynomial,
};
use constants::*;
use hashbrown::HashMap;
use itertools::izip;


#[derive(derivative::Derivative)]
#[derivative(Debug)]
pub(crate) struct Permutation {
    // Maps a variable to the wires that it is associated to.
    pub variable_map: HashMap<Variable, Vec<WireData>>,
}

impl Permutation {
    // Creates a Permutation struct with an expected capacity of zero.
    pub fn new() -> Self {
        Permutation::with_capacity(0)
    }

    // Creates a Permutation struct with an expected capacity of `n`.
    pub fn with_capacity(expected_size: usize) -> Self {
        Self {
            variable_map: HashMap::with_capacity(expected_size),
        }
    }

    // Creates a new [`Variable`] by incrementing the index of the
    // `variable_map`. This is correct as whenever we add a new [`Variable`]
    // into the system It is always allocated in the `variable_map`.
    pub fn new_variable(&mut self) -> Variable {
        // Generate the Variable
        let var = Variable(self.variable_map.keys().len());

        // Allocate space for the Variable on the variable_map
        // Each vector is initialised with a capacity of 16.
        // This number is a best guess estimate.
        self.variable_map.insert(var, Vec::with_capacity(16usize));

        var
    }

    // Checks that the [`Variable`]s are valid by determining if they have been
    // added to the system.
    fn valid_variables(&self, variables: &[Variable]) -> bool {
        variables
            .iter()
            .all(|var| self.variable_map.contains_key(var))
    }

    // Maps a set of [`Variable`]s (a,b,c,d) to a set of [`Wire`](WireData)s
    // (left, right, out, fourth) with the corresponding gate index
    pub fn add_variables_to_map(
        &mut self,
        a: Variable,
        b: Variable,
        c: Variable,
        d: Variable,
        gate_index: usize,
    ) {
        let left: WireData = WireData::Left(gate_index);
        let right: WireData = WireData::Right(gate_index);
        let output: WireData = WireData::Output(gate_index);
        let fourth: WireData = WireData::Fourth(gate_index);

        // Map each variable to the wire it is associated with
        // This essentially tells us that:
        self.add_variable_to_map(a, left);
        self.add_variable_to_map(b, right);
        self.add_variable_to_map(c, output);
        self.add_variable_to_map(d, fourth);
    }

    pub fn add_variable_to_map(&mut self, var: Variable, wire_data: WireData) {
        assert!(self.valid_variables(&[var]));

        // NOTE: Since we always allocate space for the Vec of WireData when a
        // `Variable` is added to the variable_map, this should never fail.
        let vec_wire_data = self.variable_map.get_mut(&var).unwrap();
        vec_wire_data.push(wire_data);
    }
    // Performs shift by one permutation and computes `sigma_1`, `sigma_2` and
    // `sigma_3`, `sigma_4` permutations from the variable maps.
    pub(super) fn compute_sigma_permutations(
        &mut self,
        n: usize,
    ) -> [Vec<WireData>; 4] {
        let sigma_1 = (0..n).map(WireData::Left).collect::<Vec<_>>();
        let sigma_2 = (0..n).map(WireData::Right).collect::<Vec<_>>();
        let sigma_3 = (0..n).map(WireData::Output).collect::<Vec<_>>();
        let sigma_4 = (0..n).map(WireData::Fourth).collect::<Vec<_>>();

        let mut sigmas = [sigma_1, sigma_2, sigma_3, sigma_4];

        for (_, wire_data) in self.variable_map.iter() {
            // Gets the data for each wire assosciated with this variable
            for (wire_index, current_wire) in wire_data.iter().enumerate() {
                // Fetch index of the next wire, if it is the last element
                // We loop back around to the beginning
                let next_index = match wire_index == wire_data.len() - 1 {
                    true => 0,
                    false => wire_index + 1,
                };

                // Fetch the next wire
                let next_wire = &wire_data[next_index];

                // Map current wire to next wire
                match current_wire {
                    WireData::Left(index) => sigmas[0][*index] = *next_wire,
                    WireData::Right(index) => sigmas[1][*index] = *next_wire,
                    WireData::Output(index) => sigmas[2][*index] = *next_wire,
                };
            }
        }

        sigmas
    }
}
impl Permutation {
    fn compute_permutation_lagrange<F: FftField>(
        &self,
        sigma_mapping: &[WireData],
        domain: &GeneralEvaluationDomain<F>,
    ) -> Vec<F> {
        let roots: Vec<_> = domain.elements().collect();

        let lagrange_poly: Vec<F> = sigma_mapping
            .iter()
            .map(|x| match x {
                WireData::Left(index) => {
                    let root = &roots[*index];
                    *root
                }
                WireData::Right(index) => {
                    let root = &roots[*index];
                    K1::<F>() * root
                }
                WireData::Output(index) => {
                    let root = &roots[*index];
                    K2::<F>() * root
                }
                WireData::Fourth(index) => {
                    let root = &roots[*index];
                    K3::<F>() * root
                }
            })
            .collect();

        lagrange_poly
    }

    // Computes the sigma polynomials which are used to build the permutation
    // polynomial.
    pub fn compute_sigma_polynomials<F: FftField>(
        &mut self,
        n: usize,
        domain: &GeneralEvaluationDomain<F>,
    ) -> (
        DensePolynomial<F>,
        DensePolynomial<F>,
        DensePolynomial<F>,
        DensePolynomial<F>,
    ) {
        // Compute sigma mappings
        let sigmas = self.compute_sigma_permutations(n);

        assert_eq!(sigmas[0].len(), n);
        assert_eq!(sigmas[1].len(), n);
        assert_eq!(sigmas[2].len(), n);
        assert_eq!(sigmas[3].len(), n);

        // define the sigma permutations using two non quadratic residues
        let left_sigma = self.compute_permutation_lagrange(&sigmas[0], domain);
        let right_sigma = self.compute_permutation_lagrange(&sigmas[1], domain);
        let out_sigma = self.compute_permutation_lagrange(&sigmas[2], domain);
        let fourth_sigma =
            self.compute_permutation_lagrange(&sigmas[3], domain);

        let left_sigma_poly =
            DensePolynomial::from_coefficients_vec(domain.ifft(&left_sigma));
        let right_sigma_poly =
            DensePolynomial::from_coefficients_vec(domain.ifft(&right_sigma));
        let out_sigma_poly =
            DensePolynomial::from_coefficients_vec(domain.ifft(&out_sigma));
        let fourth_sigma_poly =
            DensePolynomial::from_coefficients_vec(domain.ifft(&fourth_sigma));

        (
            left_sigma_poly,
            right_sigma_poly,
            out_sigma_poly,
            fourth_sigma_poly,
        )
    }