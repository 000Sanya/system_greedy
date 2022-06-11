use rand::{Rng, thread_rng};
use crate::{StateRegisterer, System};

pub fn metropolis_mc_step(system: &mut System, registerer: &impl StateRegisterer, temp: f64, steps: usize) {
    let mut rng = thread_rng();
    let size = system.size();

    registerer.register(system);

    for step in 0..steps {
        let e1= system.energy();
        let spin = rng.gen_range(0..size);

        system.reverse_spin(spin);
        registerer.register(system);

        let e2 = system.energy();

        if e2 >= e1 {
            let p = ((e2-e1) / temp).exp();
            if p < rng.gen() {
                system.reverse_spin(spin);
            }
        }
    }
}