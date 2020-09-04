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

To test it I run some local `nginx` on docker:

```sh
$ docker run --rm --name nginx-1 --publish 7892:80 nginx
```

```sh
$ docker run --rm --name nginx-2 --publish 9898:80 nginx
```

And run the load-balancer with `config.yaml`

```yaml
listen_on: "127.0.0.1:6767"
backends:
    - "127.0.0.1:7892"
    - "127.0.0.1:9898"
timeout: 5000
balancing: round-robin
```
