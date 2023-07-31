use crate::{nodes::*, out::Out};

pub(crate) struct Ambient;

impl Ambient {
    pub fn declare_group(binding: &mut Binding, o: &mut Out) -> u32 {
        let binding = binding.next();
        o.write(Var {
            binding,
            uniform: true,
            name: Name::Str("ambient"),
            ty: Type::VEC4,
        });

        binding.get()
    }
}
