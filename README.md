rlb
===

(**R**)ust(**L**)oad(**B**)alancer, simple reverse-proxy written to learn the
language.

PoC ported from [llb](https://github.com/codepr/llb.git) as a learning and
comparative exercise, evaluation of Rust language and dabbling with a solid
type-system.

Features:

- Basic healthcheck for backends
- Round-robin, hash-balancing, random-balancing, leasttraffic
