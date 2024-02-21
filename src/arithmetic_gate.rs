// implement arithmetization of the different circuits

use crate::constraint_system::{StandardComposer, Variable};
use ark_ec::TEModelParameters;
use ark_ff::PrimeField;

#[derive(Debug, Clone, Copy)]
pub struct ArithmeticGate<F>
where
    F: PrimeField,
{
    pub(crate) witness: Option<(Variable, Variable, Option<Variable>)>,
    pub(crate) fan_in_3: Option<(F, Variable)>,
    pub(crate) mul_selector: F,
    pub(crate) add_selectors: (F, F),
    pub(crate) out_selector: F,
    pub(crate) const_selector: F,
    pub(crate) pi: Option<F>,
}

impl<F> Default for ArithmeticGate<F>
where
    F: PrimeField,
{
    fn default() -> Self {
        Self {
            witness: None,
            fan_in_3: None,
            mul_selector: F::zero(),
            add_selectors: (F::zero(), F::zero()),
            out_selector: -F::one(),
            const_selector: F::zero(),
            pi: None,
        }
    }
}

impl<F> ArithmeticGate<F>
where
    F: PrimeField,
{
    pub fn new() -> Self {
        Self::default()
    }

    pub fn witness(
        &mut self,
        w_l: Variable,
        w_r: Variable,
        w_o: Option<Variable>,
    ) -> &mut Self {
        self.witness = Some((w_l, w_r, w_o));
        self
    }

    pub fn fan_in_3(&mut self, q_4: F, w_4: Variable) -> &mut Self {
        self.fan_in_3 = Some((q_4, w_4));
        self
    }

    pub fn mul(&mut self, q_m: F) -> &mut Self {
        self.mul_selector = q_m;
        self
    }

    pub fn add(&mut self, q_l: F, q_r: F) -> &mut Self {
        self.add_selectors = (q_l, q_r);
        self
    }

    pub fn out(&mut self, q_o: F) -> &mut Self {
        self.out_selector = q_o;
        self
    }

    pub fn constant(&mut self, q_c: F) -> &mut Self {
        self.const_selector = q_c;
        self
    }

    pub fn pi(&mut self, pi: F) -> &mut Self {
        self.pi = Some(pi);
        self
    }

    pub fn build(&mut self) -> Self {
        *self
    }
}
