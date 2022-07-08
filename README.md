# RustShop (meta)

`rustshop` is an attempt at building a template and utilities
to help quickly set up and manage a production grade cloud-based system.

The core target audience are Rust developers that are familiar or willing to learn Nix and
have some DevOps interest. However it might be useful for wide group of people:

* `rustshop` project is a bit like a tutorial for setting up AWS/k8s stack.
* `rustshop` binary is well integrated tool to make working with 
  like `terraform`, `kubectl`, `kops`, `helm` and other utitli much more convenient
  (in particular when dealing with multiple accounts/profiles/clusters).
* `rustshop` is trying to build a well oiled, flexible and extremely powerful IaaC,
   gitops based cloud software shop template that can be a source of inspiration.
   A system like this must:
   * Store both all infra and application code under revision control, gitops style.
   * Utilize a customizable (also stored under revision control) merge queue bot,
     and a CI/CD pipeline.
   * Implement well integrated solutions for all cutting edge best practices.

While the project aim is large in scope, the core technical philosophy is about
minimalism and efficiency:

* Pick few powerful and universal tools and stick to them (Rust + Nix).
* Integrate things well, but keep them extremely modular to enable change.
* Don't be afraid to implement smaller, easier to customize and right-sized solutions
  from scratch, when the mainstream does not fit your goals.

[Read more about our technical philosophy](./README.philosophy.md).

The basic technologies used are:

* Rust (for all tools, and eventually example applications)
* Nix (for all things build & dev-env & glue)
* AWS (for cloud hosting)
* Terraform (for infrastructure automation)
* Kubernetes (for orchestration)

See [Tech Stack](./README.techstack.md) for more details.
Subscribe to [Status Updates](https://github.com/rustshop/rustshop/discussions/6)
to track project progress.

### Participating

For fun and to have real-like general direction, RustShop (capitalized) is a pretend (fake)
business. Hopefully it will keep the goal and the direction be focused on solving
real-like problems.

See [Welcome to Rust Shop](https://github.com/rustshop/rustshop/discussions/1)
for more information about the idea and the project.

# Welcome to RustShop!

Tired of not being able to find a Rust job? Join RustShop -
our ambitious innovative fake company. Pretend you are delivering
business value using Rust in your free time for no pay or benefits!

## Links

* [Onboarding](./README.onboarding.md) - setting up dev env used in rustshop
* [Infra Bootstrapping](README.bootstrapping.md) - how to set up
  initial infrastructure.
* [Tech Stack](./README.techstack.md) - general information about technologies used.
* [Technical philosophy](./README.philosophy.md)
