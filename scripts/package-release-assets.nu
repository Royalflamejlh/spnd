#!/usr/bin/env nix
#! nix shell --inputs-from .. nixpkgs#nushell --command nu

# Packages the staged native binaries into the user-facing release assets:
# a tarball per Unix platform, a zip per Windows platform, .deb/.rpm built
# with nfpm, and a SHA256SUMS manifest, all written to dist/.
#
# Expects the packages/ccusage-<platform>-<arch>/bin binaries to be staged
# (in CI: the extracted native-package artifacts) and `zip` and `nfpm` on
# PATH.
def main [--version: string] {
    if ($version | is-empty) {
        error make {msg: 'pass --version <semver without the v prefix>'}
    }
    let repo_root = (
        $env.CURRENT_FILE
        | path dirname
        | path join ..
        | path expand
    )
    let dist = $repo_root | path join dist
    mkdir $dist

    let targets = [[platform, arch]; [linux, x64], [linux, arm64], [darwin, x64], [darwin, arm64], [win32, x64], [win32, arm64]]
    for target in $targets {
        let binary_name = (if $target.platform == 'win32' { 'spnd.exe' } else { 'spnd' })
        let binary = (
            $repo_root
            | path join packages $'ccusage-($target.platform)-($target.arch)' bin $binary_name
        )
        if not ($binary | path exists) {
            error make {msg: $'missing staged binary ($binary)'}
        }
        let stage = $repo_root | path join dist-stage $'($target.platform)-($target.arch)'
        mkdir $stage
        cp $binary ($stage | path join $binary_name)
        cp ($repo_root | path join LICENSE) $stage
        if $target.platform == 'win32' {
            let asset = $dist | path join $'spnd-($target.platform)-($target.arch).zip'
            ^zip --junk-paths $asset ($stage | path join $binary_name) ($stage | path join LICENSE)
        } else {
            let asset = $dist | path join $'spnd-($target.platform)-($target.arch).tar.gz'
            ^tar -czf $asset -C $stage $binary_name LICENSE
        }
    }

    for arch in [x64 arm64] {
        let config = $repo_root | path join dist-stage $'nfpm-linux-($arch).yaml'
        (
            open --raw ($repo_root | path join packaging nfpm.yaml)
            | str replace --all '@VERSION@' $version
            | str replace --all '@ARCH@' (if $arch == 'x64' { 'amd64' } else { 'arm64' })
            | str replace --all '@BINARY@' ($repo_root | path join dist-stage $'linux-($arch)' spnd)
            | save --force $config
        )
        for format in [deb rpm] {
            (^nfpm package
                --config $config
                --packager $format
                --target ($dist | path join $'spnd-linux-($arch).($format)'))
        }
    }

    let sums = (
        ls $dist
        | where type == file
        | get name
        | where {|file| ($file | path basename) != 'SHA256SUMS' }
        | each {|file| $'(open --raw $file | hash sha256)  ($file | path basename)' }
        | str join "\n"
    )
    $sums + "\n" | save --force ($dist | path join SHA256SUMS)
    print (
        ls $dist
        | where type == file
        | get name
        | each {|f| $f | path basename }
        | str join "\n"
    )
}
