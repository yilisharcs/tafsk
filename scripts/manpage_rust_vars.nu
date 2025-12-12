#!/usr/bin/env nu

const root = (path self | path dirname --num-levels 2)
let mansection = 1 # Executables or shell commands

open Cargo.toml
| do {
        let version = ($in | get package | get version)
        let name    = ($in | get package | get name)
        let bin = (
                $in
                | if bin in $in {
                        $in.bin.name.0
                } else {
                        $in.package.name
                }
        )

        nu $"($root)/scripts/manpage.nu" --root $root --ver $version --name $name --bin $bin --mansection $mansection
        # man $"([$root "doc/man"] | path join)/($bin).($mansection).zst"
}
