use ark_ec::{models::TEModelParameters, ModelParameters};
use ark_ff::{PrimeField, ToConstraintField};
use crate::permutation::Permutation;



#[derive(derivative::Derivative)]
#[derivative(Debug)]
pub struct CircuitBuilder<F, P>
where
    F: PrimeField,
    P: ModelParameters<BaseField = F>,
{
    /// Number of arithmetic gates in the circuit
    pub(crate) n: usize,

    // Selector vectors
   
    pub(crate) q_m: Vec<F>,
    pub(crate) q_l: Vec<F>,
    pub(crate) q_r: Vec<F>,
    pub(crate) q_o: Vec<F>,
    pub(crate) q_4: Vec<F>,
    pub(crate) q_c: Vec<F>,

    // Witness vectors
 
    pub(crate) w_l: Vec<Variable>,
    pub(crate) w_r: Vec<Variable>,
    pub(crate) w_o: Vec<Variable>,

    // Public lookup table
    pub(crate) lookup_table: LookupTable<F>,


    // Permutation argument.
    pub(crate) perm: Permutation,

    // public inputs
    pub(crate) public_inputs: PublicInputs<F>,


    // zero var
    pub(crate) zero_var: Variable,


    // Type Parameter Marker
    __: PhantomData<P>,
}

impl<F,P> CircuitBuilder<F,P> 
where
    F: PrimeField,
    P: TEModelParameters<BaseField:F>
{

    // Creates a new circuit with an expected circuit size.
    pub fn new(circuit_size: usize) -> Self {
        let mut builder = Self {
            n: 0,
            q_m: Vec::with_capacity(circuit_size),
            q_l: Vec::with_capacity(circuit_size),
            q_r: Vec::with_capacity(circuit_size),
            q_o: Vec::with_capacity(circuit_size),
            q_c: Vec::with_capacity(circuit_size),
            q_lookup: Vec::with_capacity(circuit_size),
            public_inputs: PublicInputs::new(),
            w_l: Vec::with_capacity(circuit_size),
            w_r: Vec::with_capacity(circuit_size),
            w_o: Vec::with_capacity(circuit_size),
            lookup_table: LookupTable::new(),
            perm: Permutation::new(),
            __: PhantomData::<P>,
        };

        builder

  
    }














     /*  The final constraint added will force the following:
    `(a * b) * q_m + a * q_l + b * q_r + q_c + PI + q_o * c = 0`.*/
    pub fn poly_gate(
        &mut self,
        a: Variable,
        b: Variable,
        c: Variable,
        q_m: F,
        q_l: F,
        q_r: F,
        q_o: F,
        q_c: F,
        pi: Option<F>,
    ) -> (Variable, Variable, Variable) {
        self.w_l.push(a);
        self.w_r.push(b);
        self.w_o.push(c);

        // Add selector vectors
   
        self.q_l.push(q_l);
        self.q_r.push(q_r);

        
        self.q_m.push(q_m);
        self.q_o.push(q_o);
        self.q_c.push(q_c);
  


        self.q_lookup.push(F::zero());



        if let Some(pi) = pi {
            self.add_pi(self.n, &pi).unwrap_or_else(|_| {
                panic!("Could not insert PI {:?} at {}", pi, self.n)
            });
        };

        self.perm
            .add_variables_to_map(a, b, c, self.zero_var, self.n);
        self.n += 1;

        (a, b, c)
    }

    // Constrain a Variable to be a constant
    pub fn constrain_to_constant(
        &mut self,
        a: Variable,
        constant: F,
        pi: Option<F>,
    ) {
        self.poly_gate(
            a,
            a,
            a,
            F::zero(),
            F::one(),
            F::zero(),
            F::zero(),
            -constant,
            pi,
        );
    }

        // assert two variable to be equal
        pub fn assert_equal(&mut self, a: Variable, b: Variable) {
        self.poly_gate(
            a,
            b,
            self.zero_var,
            F::zero(),
            F::one(),
            -F::one(),
            F::zero(),
            F::zero(),
            None,
        );
    }


    pub fn zero_var(&self) -> Variable {
        self.zero_var
    }


    pub fn add_input(&mut self, s: F) -> Variable {
        // Get a new Variable from the permutation
        let var = self.perm.new_variable();
        // The composer now links the Variable returned from
        // the Permutation to the value F.
        self.variables.insert(var, s);

        var
    }


     //Insert data in the PI starting at the given position and stores the occupied positions as intended for public inputs.
    pub(crate) fn add_pi<T>(
        &mut self,
        pos: usize,
        item: &T,
    ) -> Result<(), Error>
    where
        T: ToConstraintField<F>,
    {
        let n_positions = self.public_inputs.add_input(pos, item)?;
        self.intended_pi_pos.extend(pos..(pos + n_positions));
        Ok(())
    }


    // output 1 if the input is 0 otherwise 1

    pub fn is_zero_with_output(&mut self, a:Variable) -> Variable {
        let a_value = self.variabels.get(&a).unwrap();
        let y_value = a_value.inverse().unwrap_or_else(F::one);


        let b_value = F::one() - *a_value * y_value;

        let y = self.add_input(y_value);

        let b = self.add_input(b_value);

        let zero = self.zero_var();


        // Enforce constraints. The constraint system being used here is
        // a * y + b - 1 = 0
        // a * b = 0
        // where y is auxiliary and b is the boolean (a == 0).
        let _a_times_b = self.arithmetic_gate(|gate| {
            gate.witness(a, b, Some(zero)).mul(F::one())
        });

        let _first_constraint = self.arithmetic_gate(|gate| {
            gate.witness(a, y, Some(zero))
                .mul(F::one())
                .fan_in_3(F::one(), b)
                .constant(-F::one())
        });

        b
    }

    pub fn is_eq_with_output(&mut self, a: Variable, b: Variable) -> Variable {
        let difference = self.arithmetic_gate(|gate| {
            gate.witness(a, b, None).add(F::one(), -F::one())
        });
        self.is_zero_with_output(difference)
    }

    // Conditionally selects a [`Variable`] based on an input bit.
    
    // If:
    // bit == 1 => choice_a,
    // bit == 0 => choice_b,

    pub fn conditional_select(
        &mut self,
        bit: Variable,
        choice_a: Variable,
        choice_b: Variable,
    ) -> Variable {
        let zero = self.zero_var;
        // bit * choice_a
        let bit_times_a = self.arithmetic_gate(|gate| {
            gate.witness(bit, choice_a, None).mul(F::one())
        });

        // 1 - bit
        let one_min_bit = self.arithmetic_gate(|gate| {
            gate.witness(bit, zero, None)
                .add(-F::one(), F::zero())
                .constant(F::one())
        });

        // (1 - bit) * b
        let one_min_bit_choice_b = self.arithmetic_gate(|gate| {
            gate.witness(one_min_bit, choice_b, None).mul(F::one())
        });

        // [ (1 - bit) * b ] + [ bit * a ]
        self.arithmetic_gate(|gate| {
            gate.witness(one_min_bit_choice_b, bit_times_a, None)
                .add(F::one(), F::one())
        })
    }


    // This function adds two dummy gates to the circuit description which are guaranteed to always satisfy the gate equation.

    pub fn add_dummy_constraints(&mut self) {
        let var_six = self.add_input(F::from(6u64));
        let var_one = self.add_input(F::one());
        let var_seven = self.add_input(F::from(7u64));
        let var_min_twenty = self.add_input(-F::from(20u64));

        self.q_m.push(F::from(1u64));
        self.q_l.push(F::from(2u64));
        self.q_r.push(F::from(3u64));
        self.q_o.push(F::from(4u64));
        self.q_c.push(F::from(4u64));


        self.q_lookup.push(F::one());
     
        self.w_l.push(var_six);
        self.w_r.push(var_seven);
        self.w_o.push(var_min_twenty);

        self.perm.add_variables_to_map(
            var_six,
            var_seven,
            var_min_twenty,
            var_one,
            self.n,
        );
        self.n += 1;

        self.q_m.push(F::one());
        self.q_l.push(F::one());
        self.q_r.push(F::one());
        self.q_o.push(F::one());
        self.q_c.push(F::from(127u64));
        self.q_4.push(F::zero());

        self.q_lookup.push(F::one());

        self.w_l.push(var_min_twenty);
        self.w_r.push(var_six);
        self.w_o.push(var_seven);
    
        self.perm.add_variables_to_map(
            var_min_twenty,
            var_six,
            var_seven,
            self.zero_var,
            self.n,
        );
        self.n += 1;
    }

    // Adds 3 dummy rows to the lookup table

    pub fn add_dummy_lookup_table(&mut self) {
        self.lookup_table.insert_row(
            F::from(6u64),
            F::from(7u64),
            -F::from(20u64),
            F::one(),
        );

        self.lookup_table.insert_row(
            -F::from(20u64),
            F::from(6u64),
            F::from(7u64),
            F::zero(),
        );

        self.lookup_table.insert_row(
            F::from(3u64),
            F::one(),
            F::from(4u64),
            F::from(9u64),
        );
    }



    //This function is used to add a blinding factors to the witness and permutation polynomials.
    pub fn add_blinding_factors<R>(&mut self, rng: &mut R)
    where
        R: CryptoRng + RngCore + ?Sized,
    {
        let mut rand_var_1 = self.zero_var();
        let mut rand_var_2 = self.zero_var();
        // Blinding wires
        for _ in 0..2 {
            rand_var_1 = self.add_input(F::rand(rng));
            rand_var_2 = self.add_input(F::rand(rng));
            let rand_var_3 = self.add_input(F::rand(rng));
            let rand_var_4 = self.add_input(F::rand(rng));

            self.w_l.push(rand_var_1);
            self.w_r.push(rand_var_2);
            self.w_o.push(rand_var_3);


            // All selectors fixed to 0 so that the constraints are satisfied
            self.q_m.push(F::zero());
            self.q_l.push(F::zero());
            self.q_r.push(F::zero());
            self.q_o.push(F::zero());
            self.q_c.push(F::zero());
         
            self.q_lookup.push(F::zero());


            self.perm.add_variables_to_map(
                rand_var_1, rand_var_2, rand_var_3, rand_var_4, self.n,
            );
            self.n += 1;
        }

        // Blinding Z
        // We add 2 pairs of equal random points

        self.w_l.push(rand_var_1);
        self.w_r.push(rand_var_2);
        self.w_o.push(self.zero_var());


        // All selectors fixed to 0 so that the constraints are satisfied
        self.q_m.push(F::zero());
        self.q_l.push(F::zero());
        self.q_r.push(F::zero());
        self.q_o.push(F::zero());
        self.q_c.push(F::zero());


        self.q_lookup.push(F::zero());


        self.perm.add_variables_to_map(
            rand_var_1,
            rand_var_2,
            self.zero_var(),
            self.zero_var(),
            self.n,
        );
        self.n += 1;
    }






 









}










