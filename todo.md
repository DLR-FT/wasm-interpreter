# Todo

In Order:

- remove Ordering::Init. Add a comment for why it is missing. Idk why though
- Add a grow function to the FixedCapacityVec? It will make the transition easier.
- FixedCapacityVec everywhere. Start with the linear memory & tables. implement grow by allocating a new
  FixedCapacity vec. Then do it for the ModuleInst (i.e. idxvec, these might require a grow functionality), Data/Element segments and finally the addrvecs.
- make unwrap_validated unsafe. use debug asserts

Others:

- replace allow with expect
- remove grow methods from mem (and table?) inst types?
- C bindings
- requirement tracing?
- look globally for todos
- clean up threads proposal
- ub-checks feature like https://doc.rust-lang.org/nightly/unstable-book/compiler-flags/ub-checks.html
- multiple dispatch methods

Don't merge

- feat/threads-proposal (TODO: Find solution for spectests, wait/notify, fence?) (rename to feat/atomic-instructions, when shared-linear-memory gets merged)
