export-env {
    if $env.FMOD_PATH? == null {
        $env.FMOD_PATH = (registry query --hkcu 'Software\FMOD Studio API Windows' | get 0.value)
    }
    plugin add nu_plugin_query.exe
}

const FMOD_RS_SRC = path self ./crates/fmod-rs/src

def 'html select' [
    query: string,
] {
    let html = $in
    $html | query web --as-html --query $query | str join
}

def clean-links [] {
    $in
    | str replace -ar '\<Debug_([[:word:]]+)\>' {|it| $'`debug::($it | str snake-case)`'}
    | str replace -ar '\<File_([[:word:]]+)\>' {|it| $'`file::($it | str snake-case)`'}
    | str replace -ar '\<FMOD_DEBUG_([[:word:]]+)\>' {|it| $'`DebugFlags::($it | str pascal-case)`'}
    | str replace -ar '\<FMOD_ERR_([[:word:]]+)\>' {|it| $'`Error::($it | str pascal-case)`'}
    | str replace -ar '\<Memory_([[:word:]]+)\>' {|it| $'`memory::($it | str snake-case)`'}
    | str replace -ar '\[`(.+?)`\]\(.*?\)\{.apilink\}' {|it| $'[`($it)`]'}
}

export def get-doc [
    page: string,
    id: string,
]: nothing -> string {
    open ([$env.FMOD_PATH 'doc' 'FMOD API User Manual' $'($page).html'] | path join)
    | html select ([
        $"#($id) ~ :is\(p, dd, ul)"         # after selected header / definition title
        $":not\(#($id) ~ :is\(h2, dt) ~ *)" # but before next header / definition title
        ':not(:has(> strong:first-child))'  # and not the one with `**See Also**`
    ] | str join)
    | pandoc --from html --to markdown
    | clean-links
}

export def docgen [] {
    const REPLACER_REGEX = '(?x: \< replace\(
        \s* r\#"(?<r_pat>(?: "[^"\#] | [^"] )*)"\#
        \s* ,
        \s* "(?<r_rep>[^"]*)"
        \s* ,?
    \s* \))'

    const REGEX = ['(?x) \< fmod_doc!\(
        \s* "(?<page>[[:word:]\-]+)"
        \s* ,
        \s* "(?<id>[[:word:]\-]+)" (?: \s* \+ \s* "(?<id_suffix>[[:word:]\-]+)" )?
        (?<replacers>(?:
            \s* ,
            \s* ' $REPLACER_REGEX '
        )*)?
        \s* ,?
    \s* \)'] | str join

    ls ...(glob -D 'crates/fmod-rs/src/**/*.rs')
    | get name
    | par-each {|src|
        let dir = $src | path dirname
        open $src
        | collect
        | parse --regex $REGEX
        | flatten
        | par-each {|it|
            mut doc = get-doc $it.page $it.id
            for replacer in ($it.replacers | parse --regex $REPLACER_REGEX) {
                $doc = $doc | str replace -ar $replacer.r_pat $replacer.r_rep
            }
            mkdir $'($dir)/doc'
            $doc | save --force $'($dir)/doc/($it.id)($it.id_suffix).md'
        }
    }

    cargo docs-rs --package fmod-rs
}
