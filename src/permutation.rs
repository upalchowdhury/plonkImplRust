   // Maps a set of [`Variable`]s (a,b,c) to a set of [`Wire`]
   //(left, right, out) with the corresponding gate index
    pub fn add_variables_to_map(
        &mut self,
        a: Variable,
        b: Variable,
        c: Variable,
        gate_index: usize,
    ) {
        let left: WireData = WireData::Left(gate_index);
        let right: WireData = WireData::Right(gate_index);
        let output: WireData = WireData::Output(gate_index);

        // Map each variable to the wire it is associated with
        self.add_variable_to_map(a, left);
        self.add_variable_to_map(b, right);
        self.add_variable_to_map(c, output);
    }

    pub fn add_variable_to_map(&mut self, var: Variable, wire_data: WireData) {
        assert!(self.valid_variables(&[var]));

        let vec_wire_data = self.variable_map.get_mut(&var).unwrap();
        vec_wire_data.push(wire_data);
    }