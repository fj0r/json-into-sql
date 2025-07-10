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

export def benchmark [] {
    mut args = [
        -c 50 -n 200000
        -m POST -T application/json
        http://localhost:5050/v1/upsert/public/test?var=data
        -d '{"a":1,"score":3424,"x":"j4","ff":{"a":1},"jj":["f"]}'
    ]
    oha ...$args
}
