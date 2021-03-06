//
// optimisation.rs
// Copyright (C) 2019 Malcolm Ramsay <malramsay64@gmail.com>
// Distributed under terms of the MIT license.
//

use log::debug;
use rand::distributions::Uniform;
use rand::prelude::*;
use rand_pcg::Pcg64Mcg;
use structopt::StructOpt;

use crate::traits::*;

#[derive(StructOpt, Debug, Clone, Copy)]
pub struct BuildOptimiser {
    /// The number of steps to run the Monte-Carlo Optimisation.
    #[structopt(short, long, default_value = "100")]
    steps: u64,

    /// The initial value of the "temperature" for the simulation. This is an indicator of how bad
    /// a move can be yet still be accepted.
    #[structopt(long, default_value = "0.1")]
    kt_start: f64,

    /// The initial value of the "temperature" for the simulation. This is an indicator of how bad
    /// a move can be yet still be accepted.
    #[structopt(long)]
    kt_finish: Option<f64>,

    /// The ratio the temperature is reduced every inner_steps
    #[structopt(long)]
    kt_ratio: Option<f64>,

    /// The maximum size of a Monte-Carlo move
    #[structopt(long, default_value = "0.01")]
    max_step_size: f64,

    /// The number of steps to run before reducing the temperature. This also defines the number of
    /// steps before the step size is updated.
    #[structopt(long, default_value = "1000")]
    inner_steps: u64,

    /// This option is skipped on the command line and filled in when setting up the iterations.
    #[structopt(skip)]
    seed: Option<u64>,

    /// The minimum change of the score within an inner loop. This is the indicator of convergence
    /// which allows for an early exit.
    #[structopt(long)]
    convergence: Option<f64>,
}

impl Default for BuildOptimiser {
    fn default() -> Self {
        Self {
            kt_start: 0.1,
            kt_finish: Some(0.001),
            kt_ratio: None,
            max_step_size: 0.01,
            steps: 1000,
            inner_steps: 1000,
            seed: None,
            convergence: None,
        }
    }
}

impl BuildOptimiser {
    pub fn kt_start(&mut self, kt_start: f64) -> &mut Self {
        self.kt_start = kt_start;
        self
    }

    pub fn kt_finish(&mut self, kt_finish: f64) -> &mut Self {
        self.kt_finish = Some(kt_finish);
        self
    }

    pub fn kt_ratio(&mut self, kt_ratio: Option<f64>) -> &mut Self {
        self.kt_ratio = kt_ratio;
        self
    }

    pub fn max_step_size(&mut self, max_step_size: f64) -> &mut Self {
        self.max_step_size = max_step_size;
        self
    }

    pub fn steps(&mut self, steps: u64) -> &mut Self {
        self.steps = steps;
        self
    }

    pub fn inner_steps(&mut self, inner_steps: u64) -> &mut Self {
        self.inner_steps = inner_steps;
        self
    }

    pub fn seed(&mut self, seed: u64) -> &mut Self {
        self.seed = Some(seed);
        self
    }

    pub fn convergence(&mut self, converged: Option<f64>) -> &mut Self {
        self.convergence = converged;
        self
    }

    pub fn build(&self) -> MCOptimiser {
        let kt_ratio = match (self.kt_ratio, self.kt_finish) {
            (Some(ratio), _) => 1. - ratio,
            (None, Some(finish)) => f64::powf(finish / self.kt_start, 1. / self.steps as f64),
            (None, None) => 0.1,
        };
        debug!("Setting kt_ratio to: {}", kt_ratio);
        let seed = match self.seed {
            None => Pcg64Mcg::from_entropy().gen(),
            Some(x) => x,
        };

        MCOptimiser {
            kt_start: self.kt_start,
            kt_ratio,
            max_step_size: self.max_step_size,
            steps: self.steps,
            inner_steps: u64::min(self.inner_steps, self.steps),
            seed,
            convergence: self.convergence,
        }
    }
}

pub struct MCOptimiser {
    kt_start: f64,
    kt_ratio: f64,
    max_step_size: f64,
    steps: u64,
    inner_steps: u64,
    seed: u64,
    convergence: Option<f64>,
}

impl MCOptimiser {
    #[inline]
    fn energy_surface(&self, new: f64, old: f64, kt: f64) -> f64 {
        f64::min(f64::exp((new - old) / kt), 1.)
    }

    #[inline]
    fn test_acceptance(&self, threshold: f64, new: f64, old: f64, kt: f64) -> bool {
        threshold < self.energy_surface(new, old, kt)
    }

    fn accept_score<R: Rng + ?Sized>(
        &self,
        new: Option<f64>,
        old: f64,
        kt: f64,
        rng: &mut R,
    ) -> Option<f64> {
        let threshold: f64 = rng.gen();

        match new {
            // New score is better, keep updated state
            Some(new_score) if new_score > old => Some(new_score),
            // When the score increases, there is a probability of accepting the new
            // score, which is evaluated based on the difference between the new score,
            // and the old score along with the temperature value (kt).
            // The acceptance occurs when a random number smaller than the acceptance
            // probability is drawn.
            Some(new_score) if self.test_acceptance(threshold, new_score, old, kt) => {
                Some(new_score)
            }
            // If the first two tests fail, then the score is rejected.
            _ => None,
        }
    }

    pub fn optimise_state(&self, state: impl State) -> impl State {
        let mut score_current = match state.score() {
            Some(score) => score,
            _ => panic!("Invalid configuration passed to function, exiting."),
        };

        let mut rng = Pcg64Mcg::seed_from_u64(self.seed);
        let mut rejections: u64 = 0;

        let mut kt: f64 = self.kt_start;

        let mut basis = state.generate_basis();
        let basis_distribution = Uniform::new(0, basis.len() as usize);

        let mut step_ratio = 1.;
        let mut convergence_count = 0;

        for loop_counter in 1..=(self.steps / self.inner_steps) {
            let score_start = score_current;
            let mut loop_rejections: u64 = 0;
            for _ in 0..self.inner_steps {
                // Choose a basis at random to modify
                // This is needed later if we need to undo the change
                let basis_index: usize = basis_distribution.sample(&mut rng);

                // Make a random modification to the selected basis
                basis
                    .get_mut(basis_index)
                    // There was some error in accessing the basis,
                    // This should never occur in normal operation so panic and exit
                    .expect("Trying to access basis which doesn't exist")
                    .set_sampled(&mut rng, self.max_step_size * step_ratio);

                // Check if modification was good
                score_current = match self.accept_score(state.score(), score_current, kt, &mut rng)
                {
                    Some(score) => score,
                    // Score was rejected so we have to undo the change
                    None => {
                        basis
                            .get(basis_index)
                            // There was some error in accessing the basis,
                            // This should never occur in normal operation so panic and exit
                            .expect("Trying to access basis which doesn't exist.")
                            .reset_value();
                        // Increment counter of rejections
                        loop_rejections += 1;
                        score_current
                    }
                };
            }
            rejections += loop_rejections;
            kt *= self.kt_ratio;

            // Where the score has converged to the precision of the convergence we can exit early
            if let Some(precision) = self.convergence {
                // The current score should be larger than the original score -> optimising to
                // larger numbers
                if score_current - score_start < precision {
                    convergence_count += 1;
                    if convergence_count > 5 {
                        debug!(
                            "Found convergence of score after {} steps, difference of {}",
                            loop_counter * self.inner_steps,
                            score_current - score_start,
                        );
                        return state;
                    }
                } else {
                    // Reset to zero, convergence has to be consecutive loops
                    convergence_count = 0;
                }
            }

            // Scale step ratio with goal of 75% rejections
            // Taking shinking the cell as an example, 50% of steps will  increase the cell, so
            // we want 50% of the steps which can improve the performance to be accepted.
            // There is a limit to the usefulness though and 1e-4 has been good.
            if step_ratio > 1e-4 {
                step_ratio *= self.inner_steps as f64 / (loop_rejections as f64 + 1.);
            }
        }
        debug!(
            "Score: {:.4}, Rejected Fraction: {:.2}%",
            score_current,
            100. * rejections as f64 / self.steps as f64,
        );

        assert!(
            state.score().is_some(),
            "Final score is invalid, this shouldn't occur in normal operation"
        );
        state
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use approx::abs_diff_eq;
    use quickcheck_macros::quickcheck;

    static OPT: MCOptimiser = MCOptimiser {
        kt_start: 0.,
        kt_ratio: 0.,
        max_step_size: 0.,
        steps: 0,
        inner_steps: 0,
        seed: 0,
        convergence: None,
    };

    #[quickcheck]
    fn test_energy_surface(new: f64, old: f64) -> bool {
        let result = OPT.energy_surface(new, old, 0.);
        if new < old {
            abs_diff_eq!(result, 0.)
        } else if new >= old {
            abs_diff_eq!(result, 1.)
        } else {
            false
        }
    }

    #[quickcheck]
    fn test_energy_surface_temperature(new: f64, old: f64) -> bool {
        let result = OPT.energy_surface(new, old, 0.5);
        if new < old {
            0. < result && result < 1.
        } else if new >= old {
            abs_diff_eq!(result, 1.)
        } else {
            false
        }
    }
}
