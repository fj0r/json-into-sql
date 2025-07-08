export def post [
    data?
    --schema(-s): string = 'public'
    --table(-t): string = 'test'
] {
    let data = $data | default {{}}
    (
        http post -e
        --content-type application/json
        $"localhost:5050/v1/upsert/($schema)/($table)?var=data"
        $data
    )
}

export def schema [
    --schema(-s): string = 'public'
    --table(-t): string = 'test'
    --force(-f)
] {
    let f = if $force { "?force_update=true" } else { "" }
    http get -e $"localhost:5050/v1/schema/($schema)/($table)($f)"
}

export def list [] {
    http get -e localhost:5050/v1/list
}

export def git-hooks [act ctx] {
    if $act == 'pre-commit' and $ctx.branch == 'main' {
        cargo fmt
        git add .
    }
}
