use rustsat::instances::SatInstance;
use rustsat::types::constraints::PbConstraint;
use rustsat::types::{IWLitIter, Lit};

pub struct SatLut {
    enabled: Lit,
    out_enables: Vec<Lit>,
    entries: Vec<SatLutEntry>,
}

impl SatLut {
    pub fn new(max_in: usize, max_out: usize, instance: &mut SatInstance) -> Self {
        assert_ne!(max_in, 0);
        assert_ne!(max_out, 0);

        let enabled = instance.new_lit();
        let mut out_enables = Vec::new();
        for _ in 0..max_out {
            let lit = instance.new_lit();
            instance.add_lit_impl_lit(lit, enabled);

            if let Some(&prev) = out_enables.last() {
                instance.add_lit_impl_lit(lit, prev);
            }

            out_enables.push(lit);
        }

        let entries = (0..2_usize.pow(max_out as u32))
            .map(|idx| SatLutEntry::new(enabled, idx, max_in, &out_enables, instance))
            .collect();

        Self {
            enabled,
            out_enables,
            entries,
        }
    }
}

pub struct SatLutEntry {
    idx_bits: Vec<bool>,
    output_bits: Vec<Lit>,
}

impl SatLutEntry {
    pub fn new(
        lut_enable: Lit,
        idx: usize,
        max_in: usize,
        out_enables: &[Lit],
        instance: &mut SatInstance,
    ) -> Self {
        assert_ne!(max_in, 0);
        assert!(!out_enables.is_empty());

        Self {
            idx_bits: (0..max_in)
                .map(|bit_idx| 1_usize << bit_idx)
                .map(|mask| idx & mask != 0)
                .collect(),
            output_bits: out_enables
                .iter()
                .map(|&out_enable| {
                    let lit = instance.new_lit();
                    instance.add_lit_impl_cube(lit, &[lut_enable, out_enable]);
                    lit
                })
                .collect(),
        }
    }
}
