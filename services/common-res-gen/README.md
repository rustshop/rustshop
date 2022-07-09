# Common resource generation code

Each service/application will want to generate bunch of
yaml files and likes for k8s.

Instead of screwing around with these stupid yaml files
everywhere: inventing templating languages, overlay systems
and alikes, we just use Rust to generate it.

To have one place to enforce certain rules and contain
common behavior and uniform layer, we use this library.
