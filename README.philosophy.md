# `rustshop` philosophy

## Small set of universal tools is better than large set of "best for the job" tools

At least in SWE. Larger scope software project are like the
Game of Yenga: block by block pushing the complexity, trying
to build taller and taller (more featureful and efficient)
tower.

Every part of software system requires maintenance and expertise
to be used efficiently and effectively.

I believe that the most important benefit of Rust language is that
it is universal. It might not be as good in a domain X as a certain
language Y, but it is usually not very far behind. It also beats Y at many
other domains. In any domain it is not far from the "best tool for the job".

You can build a web frontend with Rust, a backend system, embedded system,
desktop application, distributed system, you name it. It will compile
to bare metal binary, statically linked application to put in a scratch-image-based
docker container, wasm bundle, you name it. And you only need to know
well one programming language.

Similarly, Nix is extremely flexible and universal tool for system
automation, gluing things together, building AMIs, docker containers,
scripting, CI/CD, dev environment and any other glue-like thing.

Because of this rustshop sticks to this combo and will leverage it
to its full potential.

Among other technologies that are narrow yet universal I can name:
Postgres (for anything persistence), k8s (anything orchestration),
redis (anything in memory-database).

Obviously - as with anything - a pragmatic solution should not
be dogmatic. There are times when a new/different tool or technology
bring a huge improvement. But generally a strong preference
for keeping the tool-set lean is recommended.

## Large part of the "cultural fit" is shared love for same tools

"Speaking the same language" in SWE can taken very literally.
Different programming languages, platform, etc. tend to build
around them cohesive groups that share a technical culture.

One of the large inefficiencies in tech shops are cultural
sub-groups. One team likes to use Go, another Java, and another
Python. And now each team bring a different culture,
way of doing things, etc.

Now the teams have trouble sharing people, methodologies, APIs,
build tools, infrastructure, and so on.

It would have been easier to just hire people that want to do
things similar way to begin with.

His impossible without sticking to universal tools. 

## Engineers are too afraid of building their own tooling

IMO, SWE is primarily a craft. And what kind of craftsman would
build tools for others and when having a need for a tool
in their own domain of work always go to a big box store to
buy it? What kind of woodworker wouldn't have some furniture
built with their own hands?

Quite often I've experienced engineers dealing with huge,
general purpose bloated, off the shelf tooling that doesn't
fit their needs.

There are domains and problems where it's possible to build
a general purpose solution that suits quite well everyone.

But with more complex domain and problems, oftentimes
everyone generally have very custom and disjoint needs.
One software team needs to build large C++ codebases,
another bundle web app assets, and yet another build
Java. Trying to universally address all these use-cases
with a general purpose solution leads to tons of complexity
(tons of yamls or json files, custom DSLs, long lists of
configuration options, etc.) and sacrifices (costs,
inefficiencies, workarounds).

Oftentimes, what a team needs is hand-made, cut to size
solutions that is simplistic, easy to evolve and the
group is not afraid to develop it). This creates a
synergy with universal tools and cohesive technical culture:
If your CI/CD system is a small, easy to understand codebase
and written in Rust, and your whole team knows Rust,
it is trivial for anyone to maintain, debug and improve it.


## DSLs should be avoided whenever possible

Each DSLs act like another tool that needs to be learned
and maintained. This goes directly against "small set of of
universal tools" rule.

Programming languages are the best tool for building
custom logic and tools.

Lots of tools unnecessarily exposes their user to vast and gnarly yaml-based
configuration system, or other DSLs etc. because they are unable or are unwilling
(possibly for business reasons) to turn their software into a set
of modular, composable and reusable libraries, with well defined and type
checked APIs.

But oftentimes even developers (and their whole cultures) are
obsessed with DSLs. A good example is Java with their
XML-based configurations and ideas of "resources".

Again - as with everything - it's not a blind dogma rule. The SQL is a DSL
that is widely known, understood and serving its purpose very well.


## TBD.

There is more things that I plan to put here.
