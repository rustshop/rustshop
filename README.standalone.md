# Using `rustshop` as just a tool


While `rustshop` binary can do things, like bootstrapping
new accounts and clusters, it is possible to just use it a
drop-in wrapper for `aws`, `terraform`, `kubectl`, etc.


First one needs to setup a new flake or edit the existing
one to bring the `rustshop` binary in it. Here is a
diff that should ilustrate the changes:

```diff
diff --git a/flake.nix b/flake.nix
index fae587ecfe..ab9ae19251 100644
--- a/flake.nix
+++ b/flake.nix
@@ -17,9 +17,10 @@
       url = "github:edolstra/flake-compat";
       flake = false;
     };
+    rustshop.url = "github:rustshop/rustshop?dir=rustshop";
   };
 
-  outputs = { self, nixpkgs, flake-utils, flake-compat }:
+  outputs = { self, nixpkgs, flake-utils, flake-compat, rustshop }:
     flake-utils.lib.eachDefaultSystem
       (system:
         let
@@ -55,14 +56,13 @@
 
-            ];
+            ] ++ lib.attrsets.attrValues rustshop.packages."${system}";
 
             shellHook = 
             ''
               export PATH="$(pwd)/node_modules/.bin:$(pwd)/bin:$PATH"
               git config commit.template ./git_commit_template.txt
+            . ${rustshop.packages."${system}".rustshop}/usr/share/rustshop/shell-hook.sh
             '';
           };
         }
```

After that `nix develop` should bring the `rustshop` tooling into the shel.

## Configuring and using it


Using `shop add` one can add defintions for all the shop,
and the accounts&clusters.;

```
$ shop add shop myshop.com --domain myshop.com
$ shop add account dev
$ shop add account test
```

Then using `shop configure` set up the AWS profile
and kube ctx to use within account/cluster:

```
$ shop configure account test --profile test
$ shop configure account dev --profile dev 
$ shop switch account test
Context: shop=myshop.com (myshop.com); account=test (test)
$ shop configure cluster test --ctx test
$ shop switch account dev
Context: shop=myshop.com (myshop.com); account=dev (dev)
$ shop configure cluster dev --ctx dev
```

After that it's possible to use the tools 

```
$ shop switch account dev
$ shop switch namespace app
Context: shop=myshop.com (myshop.com); account=dev (dev); cluster=dev (dev); namespace=app
$ kc get pods
NAME                                                              READY   STATUS      RESTARTS   AGE
some-pod-in-dev-a8a9c849f5-b2bn1                                  1/1     Running     0          25m
[...]
$ shop s a test
Context: shop=myshop.com (myshop.com); account=test (test); cluster=test (test); namespace=app
$ kc get pods
NAME                                                              READY   STATUS        RESTARTS   AGE
some-pod-in-test-6849c849f5-h2cnm                                 1/1     Running       0          25m
[...]
```

As you can see, switching accounts automatically changes the pods `kc get pods` returns.
