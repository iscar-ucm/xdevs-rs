# xDEVS.rs

Version of the xDEVS simulator for Rust projects.
It allows you to build and simulate computational models following the DEVS formalism.
Its API is easy to use for DEVS practitioners. Currently, their main features are speed and parallelism.

## Blazingly fast 🚀

The Rust version of xDEVS is one of the fastests APIs currently available.
We will shortly publish some preliminary results to illustrate this.

## Unsafe but sound 🔐

We all love purely safe Rust crates. However, it is extremely difficult to provide a safe **AND**
fast implementation of DEVS event propagations. Thus, we decided to use `unsafe` Rust to model
DEVS ports and message propagation. In this way, we can provide the fastest DEVS implementation
we could think of. But don't worry! We have carefully studied the DEVS simulation algorithm
to come up with all the invariants that make our implementation safe even with `unsafe` code.
All the `unsafe` methods of ports come with proper documentation to let you know when it is safe
to use these `unsafe` methods!

**Spoiler alert:** if you don't try to *hack* the DEVS simulation workflow,
then you will always fufill the invariants to safely build your models.

## Fully configurable parallelism 🧶

We rely on the [`rayon`](https://github.com/rayon-rs/rayon) crate to provide parallelism for your simulations.
By default, all the simulation process is done sequentially. However, you can activate different features to
select where you want to take advantage of parallelism:

- `par_start`: it runs in parallel the start methods of your model before starting to simulate.
- `par_collection`: it executes the lambdas of your models in parallel.
- `par_eoc`: it propagates the EOCs in parallel (we **DO NOT** recommend this feature, it is likely to be removed).
- `par_xic`: it propagates the EICs and ICs in parallel (we **DO NOT** recommend this feature, it is likely to be removed).
- `par_transition`: it executes the deltas of your models in parallel (we **DO** recommend this feature).
- `par_stop`: it runs in parallel the stop methods of your model after the simulation.

### Useful combined features

We provide additional features to select handy combinations of features:

- `par_xxc`: alias for `par_eoc` and `par_xic` (we **DO NOT** recommend this feature, it is likely to be removed).
- `par_sim_no_xxc`: alias for `par_collection` and `par_transition`.
- `par_sim`: alias for `par_xxc` and `par_sim_no_xxc` (we **DO NOT** recommend this feature, it is likely to be removed).
- `par_all_no_xxc`: alias for `par_start`, `par_sim_no_xxc`, and `par_stop` (**THIS IS OUR FAVOURITE**).
- `par_all`: alias for `par_xxc` and `par_all_no_xcc` (we **DO NOT** recommend this feature, it is likely to be removed).

## Work in progress 👷‍♀️👷👷‍♂️

We are still working on this crate, and hope to add a plethora of cool features in the near future.
Stay tuned! If you want to contribute, feel free to open an issue on GitHub, we will reply ASAP.
