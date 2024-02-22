

pub struct WireEvaluations<F>
where
    F: Field,
{
    /// Evaluation of the witness polynomial for the left wire at `z`.
    pub a_eval: F,

    /// Evaluation of the witness polynomial for the right wire at `z`.
    pub b_eval: F,

    /// Evaluation of the witness polynomial for the output wire at `z`.
    pub c_eval: F,

    /// Evaluation of the witness polynomial for the fourth wire at `z`.
    pub d_eval: F,
}

/// Subset of the [`ProofEvaluations`]. Evaluations of the sigma and permutation
/// polynomials at `z`  or `z *w` where `w` is the nth root of unity.
#[derive(CanonicalDeserialize, CanonicalSerialize, derivative::Derivative)]
#[derivative(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct PermutationEvaluations<F>
where
    F: Field,
{
    /// Evaluation of the left sigma polynomial at `z`.
    pub left_sigma_eval: F,

    /// Evaluation of the right sigma polynomial at `z`.
    pub right_sigma_eval: F,

    /// Evaluation of the out sigma polynomial at `z`.
    pub out_sigma_eval: F,

    /// Evaluation of the permutation polynomial at `z * omega` where `omega`
    /// is a root of unity.
    pub permutation_eval: F,
}

// Probably all of these should go into CustomEvals
#[derive(CanonicalDeserialize, CanonicalSerialize, derivative::Derivative)]
#[derivative(Clone, Debug, Default, Eq, PartialEq)]
pub struct LookupEvaluations<F>
where
    F: Field,
{
    pub q_lookup_eval: F,
    // (Shifted) Evaluation of the lookup permutation polynomial at `z * root
    // of unity`
    pub z2_next_eval: F,

    /// Evaluations of the first half of sorted plonkup poly at `z`
    pub h1_eval: F,

    /// (Shifted) Evaluations of the even indexed half of sorted plonkup poly
    /// at `z root of unity
    pub h1_next_eval: F,

    /// Evaluations of the odd indexed half of sorted plonkup poly at `z
    /// root of unity
    pub h2_eval: F,

    /// Evaluations of the query polynomial at `z`
    pub f_eval: F,

    /// Evaluations of the table polynomial at `z`
    pub table_eval: F,

    /// Evaluations of the table polynomial at `z * root of unity`
    pub table_next_eval: F,
}

/// Subset of the [`ProofEvaluations`]. Evaluations at `z`  or `z *w` where `w`
/// is the nth root of unity of selectors polynomials needed for custom gates
#[derive(CanonicalDeserialize, CanonicalSerialize, derivative::Derivative)]
#[derivative(Clone, Debug, Default, Eq, PartialEq)]
pub struct CustomEvaluations<F>
where
    F: Field,
{
    pub vals: Vec<(String, F)>,
}

impl<F> CustomEvaluations<F>
where
    F: Field,
{
    /// Get the evaluation of the specified label.
    /// This funtions panics if the requested label is not found
    pub fn get(&self, label: &str) -> F {
        if let Some(result) = &self.vals.iter().find(|entry| entry.0 == label) {
            result.1
        } else {
            panic!("{} label not found in evaluations set", label)
        }
    }

    /// Add evaluation of poly at point if the label is not already
    /// in the set of evaluations
    pub fn add(&mut self, label: &str, poly: DensePolynomial<F>, point: F) {
        if let Some(_l) = &self.vals.iter().find(|entry| entry.0 == label) {
        } else {
            let eval = poly.evaluate(&point);
            let _ = &self.vals.push((label.to_string(), eval));
        }
    }
}

/// Set of evaluations that form the [`Proof`](super::Proof).
#[derive(CanonicalDeserialize, CanonicalSerialize, derivative::Derivative)]
#[derivative(Clone, Debug, Default, Eq, PartialEq)]
pub struct ProofEvaluations<F>
where
    F: Field,
{
    /// Wire evaluations
    pub wire_evals: WireEvaluations<F>,

    /// Permutation and sigma polynomials evaluations
    pub perm_evals: PermutationEvaluations<F>,

    /// Lookup evaluations
    pub lookup_evals: LookupEvaluations<F>,

    /// Evaluations needed for custom gates. This includes selector polynomials
    /// and evaluations of wire polynomials at an offset
    pub custom_evals: CustomEvaluations<F>,
}

/// Compute the linearisation polynomial.
pub fn compute<F, P>(
    domain: &GeneralEvaluationDomain<F>,
    prover_key: &ProverKey<F>,
    alpha: &F,
    beta: &F,
    gamma: &F,
    delta: &F,
    epsilon: &F,
    zeta: &F,
    range_separation_challenge: &F,
    logic_separation_challenge: &F,
    fixed_base_separation_challenge: &F,
    var_base_separation_challenge: &F,
    lookup_separation_challenge: &F,
    z_challenge: &F,
    w_l_poly: &DensePolynomial<F>,
    w_r_poly: &DensePolynomial<F>,
    w_o_poly: &DensePolynomial<F>,
    w_4_poly: &DensePolynomial<F>,
    t_1_poly: &DensePolynomial<F>,
    t_2_poly: &DensePolynomial<F>,
    t_3_poly: &DensePolynomial<F>,
    t_4_poly: &DensePolynomial<F>,
    t_5_poly: &DensePolynomial<F>,
    t_6_poly: &DensePolynomial<F>,
    t_7_poly: &DensePolynomial<F>,
    t_8_poly: &DensePolynomial<F>,
    z_poly: &DensePolynomial<F>,
    z2_poly: &DensePolynomial<F>,
    f_poly: &DensePolynomial<F>,
    h1_poly: &DensePolynomial<F>,
    h2_poly: &DensePolynomial<F>,
    table_poly: &DensePolynomial<F>,
) -> Result<(DensePolynomial<F>, ProofEvaluations<F>), Error>
where
    F: PrimeField,
    P: TEModelParameters<BaseField = F>,
{
    let n = domain.size();
    let omega = domain.group_gen();
    let shifted_z_challenge = *z_challenge * omega;

    // Wire evaluations
    let a_eval = w_l_poly.evaluate(z_challenge);
    let b_eval = w_r_poly.evaluate(z_challenge);
    let c_eval = w_o_poly.evaluate(z_challenge);
    let d_eval = w_4_poly.evaluate(z_challenge);
    let wire_evals = WireEvaluations {
        a_eval,
        b_eval,
        c_eval,
        d_eval,
    };
    // Permutation evaluations
    let left_sigma_eval =
        prover_key.permutation.left_sigma.0.evaluate(z_challenge);
    let right_sigma_eval =
        prover_key.permutation.right_sigma.0.evaluate(z_challenge);
    let out_sigma_eval =
        prover_key.permutation.out_sigma.0.evaluate(z_challenge);
    let permutation_eval = z_poly.evaluate(&shifted_z_challenge);

    let perm_evals = PermutationEvaluations {
        left_sigma_eval,
        right_sigma_eval,
        out_sigma_eval,
        permutation_eval,
    };

    // Arith selector evaluation
    let q_arith_eval = prover_key.arithmetic.q_arith.0.evaluate(z_challenge);

    // Lookup selector evaluation
    let q_lookup_eval = prover_key.lookup.q_lookup.0.evaluate(z_challenge);

    // Custom gate evaluations
    let q_c_eval = prover_key.arithmetic.q_c.0.evaluate(z_challenge);
    let q_l_eval = prover_key.arithmetic.q_l.0.evaluate(z_challenge);
    let q_r_eval = prover_key.arithmetic.q_r.0.evaluate(z_challenge);
    let a_next_eval = w_l_poly.evaluate(&shifted_z_challenge);
    let b_next_eval = w_r_poly.evaluate(&shifted_z_challenge);
    let d_next_eval = w_4_poly.evaluate(&shifted_z_challenge);

    // High degree selector evaluations
    let q_hl_eval = prover_key.arithmetic.q_hl.0.evaluate(z_challenge);
    let q_hr_eval = prover_key.arithmetic.q_hr.0.evaluate(z_challenge);
    let q_h4_eval = prover_key.arithmetic.q_h4.0.evaluate(z_challenge);

    let custom_evals = CustomEvaluations {
        vals: vec![
            label_eval!(q_arith_eval),
            label_eval!(q_c_eval),
            label_eval!(q_l_eval),
            label_eval!(q_r_eval),
            label_eval!(q_hl_eval),
            label_eval!(q_hr_eval),
            label_eval!(q_h4_eval),
            label_eval!(a_next_eval),
            label_eval!(b_next_eval),
            label_eval!(d_next_eval),
        ],
    };

    let z2_next_eval = z2_poly.evaluate(&shifted_z_challenge);
    let h1_eval = h1_poly.evaluate(z_challenge);
    let h1_next_eval = h1_poly.evaluate(&shifted_z_challenge);
    let h2_eval = h2_poly.evaluate(z_challenge);
    let f_eval = f_poly.evaluate(z_challenge);
    let table_eval = table_poly.evaluate(z_challenge);
    let table_next_eval = table_poly.evaluate(&shifted_z_challenge);

    // Compute the last term in the linearisation polynomial
    // (negative_quotient_term):
    // - Z_h(z_challenge) * [t_1(X) + z_challenge^n * t_2(X) + z_challenge^2n *
    //   t_3(X) + z_challenge^3n * t_4(X)]
    let vanishing_poly_eval =
        domain.evaluate_vanishing_polynomial(*z_challenge);
    let z_challenge_to_n = vanishing_poly_eval + F::one();
    let l1_eval = proof::compute_first_lagrange_evaluation(
        domain,
        &vanishing_poly_eval,
        z_challenge,
    );

    let lookup_evals = LookupEvaluations {
        q_lookup_eval,
        z2_next_eval,
        h1_eval,
        h1_next_eval,
        h2_eval,
        f_eval,
        table_eval,
        table_next_eval,
    };

    let gate_constraints = compute_gate_constraint_satisfiability::<F, P>(
        range_separation_challenge,
        logic_separation_challenge,
        fixed_base_separation_challenge,
        var_base_separation_challenge,
        &wire_evals,
        q_arith_eval,
        &custom_evals,
        prover_key,
    );

    let lookup = prover_key.lookup.compute_linearisation(
        l1_eval,
        a_eval,
        b_eval,
        c_eval,
        d_eval,
        f_eval,
        table_eval,
        table_next_eval,
        h1_next_eval,
        h2_eval,
        z2_next_eval,
        *delta,
        *epsilon,
        *zeta,
        z2_poly,
        h1_poly,
        *lookup_separation_challenge,
    );

    let permutation = prover_key.permutation.compute_linearisation(
        n,
        *z_challenge,
        (*alpha, *beta, *gamma),
        (a_eval, b_eval, c_eval, d_eval),
        (left_sigma_eval, right_sigma_eval, out_sigma_eval),
        permutation_eval,
        z_poly,
    )?;

    let quotient_term = &(&(&(&(&(&(&(&(&(&(&(&(&(&(t_8_poly
        * z_challenge_to_n)
        + t_7_poly)
        * z_challenge_to_n)
        + t_6_poly)
        * z_challenge_to_n)
        + t_5_poly)
        * z_challenge_to_n)
        + t_4_poly)
        * z_challenge_to_n)
        + t_3_poly)
        * z_challenge_to_n)
        + t_2_poly)
        * z_challenge_to_n)
        + t_1_poly)
        * vanishing_poly_eval;
    let negative_quotient_term = &quotient_term * (-F::one());

    let linearisation_polynomial =
        gate_constraints + permutation + lookup + negative_quotient_term;

    Ok((
        linearisation_polynomial,
        ProofEvaluations {
            wire_evals,
            perm_evals,
            lookup_evals,
            custom_evals,
        },
    ))
}





// ================ quotiont polys =============
pub fn compute_quo<F, P>(
    domain: &GeneralEvaluationDomain<F>,
    prover_key: &ProverKey<F>,
    z_poly: &DensePolynomial<F>,
    z2_poly: &DensePolynomial<F>,
    w_l_poly: &DensePolynomial<F>,
    w_r_poly: &DensePolynomial<F>,
    w_o_poly: &DensePolynomial<F>,
    w_4_poly: &DensePolynomial<F>,
    public_inputs_poly: &DensePolynomial<F>,
    f_poly: &DensePolynomial<F>,
    table_poly: &DensePolynomial<F>,
    h1_poly: &DensePolynomial<F>,
    h2_poly: &DensePolynomial<F>,
    alpha: &F,
    beta: &F,
    gamma: &F,
    delta: &F,
    epsilon: &F,
    zeta: &F,
    range_challenge: &F,
    logic_challenge: &F,
    fixed_base_challenge: &F,
    var_base_challenge: &F,
    lookup_challenge: &F,
) -> Result<DensePolynomial<F>, Error>
where
    F: PrimeField,
    P: TEModelParameters<BaseField = F>,
{
    let domain_8n = GeneralEvaluationDomain::<F>::new(8 * domain.size())
        .ok_or(Error::InvalidEvalDomainSize {
        log_size_of_group: (8 * domain.size()).trailing_zeros(),
        adicity:
            <<F as FftField>::FftParams as ark_ff::FftParameters>::TWO_ADICITY,
    })?;

    let l1_poly = compute_first_lagrange_poly_scaled(domain, F::one());
    let l1_eval_8n = domain_8n.coset_fft(&l1_poly);

    let mut z_eval_8n = domain_8n.coset_fft(z_poly);
    z_eval_8n.push(z_eval_8n[0]);
    z_eval_8n.push(z_eval_8n[1]);
    z_eval_8n.push(z_eval_8n[2]);
    z_eval_8n.push(z_eval_8n[3]);
    z_eval_8n.push(z_eval_8n[4]);
    z_eval_8n.push(z_eval_8n[5]);
    z_eval_8n.push(z_eval_8n[6]);
    z_eval_8n.push(z_eval_8n[7]);

    let mut wl_eval_8n = domain_8n.coset_fft(w_l_poly);
    wl_eval_8n.push(wl_eval_8n[0]);
    wl_eval_8n.push(wl_eval_8n[1]);
    wl_eval_8n.push(wl_eval_8n[2]);
    wl_eval_8n.push(wl_eval_8n[3]);
    wl_eval_8n.push(wl_eval_8n[4]);
    wl_eval_8n.push(wl_eval_8n[5]);
    wl_eval_8n.push(wl_eval_8n[6]);
    wl_eval_8n.push(wl_eval_8n[7]);

    let mut wr_eval_8n = domain_8n.coset_fft(w_r_poly);
    wr_eval_8n.push(wr_eval_8n[0]);
    wr_eval_8n.push(wr_eval_8n[1]);
    wr_eval_8n.push(wr_eval_8n[2]);
    wr_eval_8n.push(wr_eval_8n[3]);
    wr_eval_8n.push(wr_eval_8n[4]);
    wr_eval_8n.push(wr_eval_8n[5]);
    wr_eval_8n.push(wr_eval_8n[6]);
    wr_eval_8n.push(wr_eval_8n[7]);

    let wo_eval_8n = domain_8n.coset_fft(w_o_poly);

    let mut w4_eval_8n = domain_8n.coset_fft(w_4_poly);
    w4_eval_8n.push(w4_eval_8n[0]);
    w4_eval_8n.push(w4_eval_8n[1]);
    w4_eval_8n.push(w4_eval_8n[2]);
    w4_eval_8n.push(w4_eval_8n[3]);
    w4_eval_8n.push(w4_eval_8n[4]);
    w4_eval_8n.push(w4_eval_8n[5]);
    w4_eval_8n.push(w4_eval_8n[6]);
    w4_eval_8n.push(w4_eval_8n[7]);

    let mut z2_eval_8n = domain_8n.coset_fft(z2_poly);
    z2_eval_8n.push(z2_eval_8n[0]);
    z2_eval_8n.push(z2_eval_8n[1]);
    z2_eval_8n.push(z2_eval_8n[2]);
    z2_eval_8n.push(z2_eval_8n[3]);
    z2_eval_8n.push(z2_eval_8n[4]);
    z2_eval_8n.push(z2_eval_8n[5]);
    z2_eval_8n.push(z2_eval_8n[6]);
    z2_eval_8n.push(z2_eval_8n[7]);

    let f_eval_8n = domain_8n.coset_fft(f_poly);

    let mut table_eval_8n = domain_8n.coset_fft(table_poly);
    table_eval_8n.push(table_eval_8n[0]);
    table_eval_8n.push(table_eval_8n[1]);
    table_eval_8n.push(table_eval_8n[2]);
    table_eval_8n.push(table_eval_8n[3]);
    table_eval_8n.push(table_eval_8n[4]);
    table_eval_8n.push(table_eval_8n[5]);
    table_eval_8n.push(table_eval_8n[6]);
    table_eval_8n.push(table_eval_8n[7]);

    let mut h1_eval_8n = domain_8n.coset_fft(h1_poly);
    h1_eval_8n.push(h1_eval_8n[0]);
    h1_eval_8n.push(h1_eval_8n[1]);
    h1_eval_8n.push(h1_eval_8n[2]);
    h1_eval_8n.push(h1_eval_8n[3]);
    h1_eval_8n.push(h1_eval_8n[4]);
    h1_eval_8n.push(h1_eval_8n[5]);
    h1_eval_8n.push(h1_eval_8n[6]);
    h1_eval_8n.push(h1_eval_8n[7]);

    let h2_eval_8n = domain_8n.coset_fft(h2_poly);

    let gate_constraints = compute_gate_constraint_satisfiability::<F, P>(
        domain,
        *range_challenge,
        *logic_challenge,
        *fixed_base_challenge,
        *var_base_challenge,
        prover_key,
        &wl_eval_8n,
        &wr_eval_8n,
        &wo_eval_8n,
        &w4_eval_8n,
        public_inputs_poly,
    )?;

    let permutation = compute_permutation_checks::<F>(
        domain,
        prover_key,
        &wl_eval_8n,
        &wr_eval_8n,
        &wo_eval_8n,
        &w4_eval_8n,
        &z_eval_8n,
        *alpha,
        *beta,
        *gamma,
    )?;

    let lookup = prover_key.lookup.compute_lookup_quotient_term(
        domain,
        &wl_eval_8n,
        &wr_eval_8n,
        &wo_eval_8n,
        &w4_eval_8n,
        &f_eval_8n,
        &table_eval_8n,
        &h1_eval_8n,
        &h2_eval_8n,
        &z2_eval_8n,
        &l1_eval_8n,
        *delta,
        *epsilon,
        *zeta,
        *lookup_challenge,
    )?;

    let quotient = (0..domain_8n.size())
        .map(|i| {
            let numerator = gate_constraints[i] + permutation[i] + lookup[i];
            let denominator = prover_key.v_h_coset_8n()[i];
            numerator * denominator.inverse().unwrap()
        })
        .collect::<Vec<_>>();

    Ok(DensePolynomial::from_coefficients_vec(
        domain_8n.coset_ifft(&quotient),
    ))
}



// TODO 
// compute gate constraint satisfiability for both linearization poly and quotient poly


