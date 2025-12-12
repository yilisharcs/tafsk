#!/usr/bin/env nu

def main [
  --root: string
  --ver: string
  --name: string
  --bin: string
  --mansection: int
] {
        let date = (date now | format date "%d %b %Y" )
        let mandir = ([$root "doc/man"] | path join)
        if not ($mandir | path exists) {
                mkdir $mandir
        } else {
                ls $mandir
                | where name =~ ".zst"
                | each { rm $in.name }
                | ignore
        }
        open README.md
        | lines
        | insert 0 $"% ($bin)\(($mansection)\) ($bin) ($ver) | ($name | str title-case) Manual"
        | insert 1 "%"
        | insert 2 $"% ($date)"
        | insert 3 ""
        | each {
                if ($in | str starts-with "# ") {
                        prepend (char hamburger)
                } else {
                        return $in
                }
        }
        | flatten
        | to text
        | split row (char hamburger)
        | do {
                let synopsis_index = (
                        $in
                        | enumerate
                        | where $it.item =~ "# SYNOPSIS"
                        | first
                        | get index
                )
                $in
                | update $synopsis_index {
                        lines
                        | skip 1
                        | drop
                        | each {
                                if not ($in | str starts-with "# ") {
                                        str replace --regex "^" "| "
                                } else {
                                        return $in
                                }
                        }
                        | to text
                }
        }
        | to text
        | pandoc --standalone --from markdown-smart-tex_math_dollars --to man
        | zstd --compress --force -19 -o $"($mandir)/($bin).($mansection).zst"
}
