export def post [
    data?
    --schema(-s): string = 'public'
    --table(-t): string = 'test'
] {
    let data = $data | default {{}}
    (
        http post
        --content-type application/json
        $"localhost:5050/v1/upsert/($schema)/($table)?var=x"
        $data
    )
}

export def schema [
    --schema(-s): string = 'public'
    --table(-t): string = 'test'
] {
    http get $"localhost:5050/v1/schema/($schema)/($table)"
}
