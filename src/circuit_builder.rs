use ark_ec::{models::TEModelParameters, ModelParameters};
use ark_ff::{PrimeField, ToConstraintField};



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

    // Type Parameter Marker
    __: PhantomData<P>,
}

impl CircuitBuilder {
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
}


}







