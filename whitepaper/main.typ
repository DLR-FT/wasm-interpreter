/* imports */
#import "@preview/acrostiche:0.4.0": *
#import "template.typ": setup_template

#show: setup_template.with(
  title: [WASM Interpreter for Safety -- White Paper],
  /* TODO make this multi-author capable */
  author: "Wanja Zaeske", keywords: ("WebAssembly", "Safety-Critical"),
  /* TODO set affiliation per author (or for multiple authors at once) */
  affiliation: [
    Department Safety Critical Systems & Systems Engineering \
    German Aerospace Center (DLR) \
    #link("mailto:wanja.zaeske@dlr.de")
  ],
)

#init-acronyms(
  (
    "AOT": ("Ahead-of-time"), "DAL": ("Design Assurance Level"), "DARPA": ("Defense Advanced Research Projects Agency"), "IR": ("Intermediate Representation"), "JIT": ("Just-in-time"), "SBOM": ("Software Bill of Materials"), "TQL": ("Tool Qualification Level"), "TRACTOR": ("Translating All C to Rust"), "WASM": ("WebAssembly"),
  ),
)

= Introduction
This white paper provides an overview over our WebAssembly interpreter
@our-repo. It Primarily serves to highlight and explain design decisions and the
goals we aim to achieve.

The development of this project currently takes place as a joint effort between
DLR-ft @dlr-ft-website and OxidOS @oxidos-website. Being truly Open Source, the
project already is willing to accept external contributions. As the project
matures, it might be worth considering moving the project to a vendor-agnostic
foundation, to assure for a fair process allowing contributions from multiple
commercial entities. However, for the time being the project is not mature
enough to justify the overhead doing so, thus for the time being the project
remains under control of DLR & OxidOS.

== Design Drivers
The implementation of our interpreter is driven by several design goals. After
all, there are dozens of WebAssembly interpreters around, why would one have to
implement yet another one?

=== Pure Interpretation
In fact, the prior sentence is somewhat imprecise. Most WebAssembly interpreters
are actually compilers; they compile the WebAssembly down to a different format
that seems more suitable for execution. A prominent example would be WasmTime
@wasmtime-website, which compiles WebAssembly down to object code, so called #acr("AOT") compilation.
Browsers commonly mix in #acr("JIT") compilation as well, starting with a
quick-to-compile but not so fast-to-run #acr("AOT") baseline build, which then
on demand is partially swapped for optimized sections obtained by #acr("JIT") compiling
code sections in the hot path (example: @mozilla-baseline-compiler).

But even if neither #acr("AOT") nor #acr("JIT") are at hand (no object code for
the physical processor is created), there still is a common pattern of rewriting
the #acr("WASM") bytecode into a custom #acr("IR") (example:
@wasmi-instruction-enum). There are multiple reasons for this, this allows for
gentle optimizations, simpler code, and foremost explicit jumps. In ordinary #acr("WASM") bytecode,
branches are indirect; e.g. a branch could be "jump out of the innermost three
scopes". In practical terms, a branch boils down to modifying the program
counter/instruction pointer, i.e. advance it 15 instructions forward. To avoid
costly linear search at runtime (for where the innermost three scopes end), many
interpreters rewrite the #acr("WASM") bytecode into their own #acr("IR"),
calculating the direct offsets on the instruction pointer for the branches once,
so that in their #acr("IR") branches are direct.

This is a good decision for many use-cases, but tricky when safety-certification
comes into play. Now, the currency in safety certification is evidence, and for
the special case of avionics, a significant amount of evidence has to be
provided for the object code @webassembly-in-avionics-paper. Thus, generating
object code at run-time (for example when #acr("JIT") compiling) is prohibitive
-- it simply does not fit together with the assumptions baked into current
certification guidelines such as DO-178C @do178c. Finally, DO-332 contains
specific language that allows for the directly interpreted format executed by a
virtual machine/bytecode interpreter to be treated as object code. Hence, when
compiling #acr("WASM") bytecode into a custom #acr("IR"), one would have to
provide certification evidence on that #acr("IR").

Our design alleviates all this; by directly interpreting the #acr("WASM") bytecode,
certification evidence has to be produced for the #acr("WASM") directly, and
thus is not tied to implementation details (as in particular #acr("IR")) used by
our interpreter. A comprehensive discussion of this can be found in our
publication @webassembly-in-avionics-paper.

To avoid the problem of indirect branching, we borrow ideas from Ben L. Titzer's
wizard @wizard-engine, a pre-computed side-table that for all branches stores
the direct offset on the instruction pointer. A detailed discussion of the
technique and further implementation details of wizard can be found in his paper
@fast-in-place-interpreter-for-webassembly.

=== Certification Friendly Source Code
Our implementation is written in Rust, which on its own poses a challenge: As of
now, we are not aware of any high-assurance deployment of Rust in the aviation
sector. Similarly, there is only little information on Rust being deployed in
automotive (such as @volvo-rust-assembly-line). At the same time, there is a
keen interest to push Rust in safety critical domains, and companies like
ferrous systems @ferrous-systems-website with their ISO 26262 ASIL D certified
Rust compiler @ferrocene-website and AdaCore with the GNAT Pro for Rust
@adacore-rust-website pave the way for Rust.

Now, there some techniques often found in Rust programs pose risk for a smooth
certification. One such thing would be macros. While we are not aware of a
precedence case, we assume that Rust's various flavors of macros might be
treated like tools, after all they are computer programs that generated code,
which in term is compiled in to the final application. As such, they might be
subject to tool qualification as per DO-330 @do330, and in since macros can
easily sneak in broken code, they are likely to be treated as criteria 1 tool
@do330. If these assumptions hold, macros would have to be tool qualified to the
matching #acr("TQL") of an application's #acr("DAL"). As testing macros is more
complicated than testing normal code, we restrict our usage of macros to the
bare minimum.

Another risk to certification is third party code. Thus, to keep our code
closure (and subsequently the #acr("SBOM") compact), we refrain from using
dependencies which get compiled into the code#footnote[Currently, two carefully selected run-time dependencies are allowed, however,
  there is a roadmap to phase them out, and each such exception is tracked in our
  requirements].

/*
TODO talk about Nix
=== Infrastructure as Code wherever possible
*/

#bibliography("refs.yaml")
