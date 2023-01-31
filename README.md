# xDEVS.rs

Version of the xDEVS simulator for Rust projects.
It allows you to build and simulate computational models following the DEVS formalism.
Its API is easy to use for DEVS practitioners.
Currently, their main features are speed and parallelism.

## Blazingly fast 🚀

The Rust version of xDEVS is one of the fastests APIs currently available.
We will shortly publish some preliminary results to illustrate this.

## Fully configurable parallelism 🧶

We rely on the [`rayon`](https://github.com/rayon-rs/rayon) crate to provide parallelism for your simulations.
By default, all the simulation process is done sequentially. However, you can activate different features to
select where you want to take advantage of parallelism:

- `par_start`: it runs in parallel the start methods of your model before starting to simulate.
- `par_collection`: it executes the lambdas of your models in parallel.
- `par_eoc`: it propagates the EOCs in parallel (we do not recommend this feature, it is likely to be removed).
- `par_xic`: it propagates the EICs and ICs in parallel (we do not recommend this feature, it is likely to be removed).
- `par_transition`: it executes the deltas of your models in parallel (we **DO** recommend this feature).
- `par_stop`: it runs in parallel the stop methods of your model after the simulation.

### Useful combined features

We provide additional features to select handy combinations of features:

- `par_xxc`: alias for `par_eoc` and `par_xic` (we do not recommend this feature, it is likely to be removed).
- `par_sim_no_xcc`: alias for `par_collection` and `par_transition`.
- `par_sim`: alias for `par_xxc` and `par_sim_no_xcc` (we do not recommend this feature, it is likely to be removed).
- `par_all_no_xcc`: alias for `par_start`, `par_sim_no_xcc`, and `par_stop` (**this is our favourite**).
- `par_all`: alias for `par_xxc` and `par_all_no_xcc` (we do not recommend this feature, it is likely to be removed).

## Work in progress 👷‍♀️👷👷‍♂️

We are still working on this crate, and hope to add a plethora of cool features in the near future. Stay tuned!
If you want to contribute, feel free to open an issue on GitHub, we will reply ASAP.
