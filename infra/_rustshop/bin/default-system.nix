# See https://github.com/edolstra/flake-compat/issues/32#issuecomment-1169370930
system:
(import
  (
    let
      lock = builtins.fromJSON (builtins.readFile ./flake.lock);
    in
    fetchTarball {
      url = "https://github.com/edolstra/flake-compat/archive/${lock.nodes.flake-compat.locked.rev}.tar.gz";
      sha256 = lock.nodes.flake-compat.locked.narHash;
    }
  )
  {
    src = ./.;
    system = system;
  }).defaultNix
