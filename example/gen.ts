import { sql } from "bun"
import { load } from 'js-toml'
import { parseArgs } from 'util'


const dereferences = (tbl, defs) => {
    let r = []
    if ('type' in defs) {
        r[0] = defs.type.toUpperCase()
    }
    if (defs.notnull) {
        r.push('NOT NULL')
    }
    if (defs.default) {
        let m = {
            'jsonb': `'${defs.default}'::jsonb`,
        }
        if (defs.type in m) {
            r.push(`DEFAULT ${m[defs.type]}`)
        } else {
            r.push(`DEFAULT ${defs.default}`)
        }

    }
    if (defs.uniq) {
        r.push('UNIQ')
    }
    if ('references' in defs) {
        let refs = defs.references
        let c = tbl[refs[0]].column[refs[1]]
        let m = {
            'serial': 'integer',
            'bigserial': 'biginteger'
        }
        let t = c.type in m ? m[c.type] : c.type
        r[0] = t.toUpperCase()
        r.push(`REFERENCES ${refs[0]} (${refs[1]})`)
    }
    return r
}

export const gen = async (x) => {
    let stmt = []
    for (let [k, v] of Object.entries(x)) {
        let f = []
        for (let [j, w] of Object.entries(v.column ?? {})) {
            let t = dereferences(x, w)
            if ('enum' in w) {
                let n = `${k}_${j}`
                let c = w.enum.map(x => `'${x}'`).join(', ')
                stmt.push(`CREATE TYPE ${n} AS ENUM (${c});`)
                t[0] = n
            }
            f.push(`    ${j} ${t.join(' ')}`)
        }
        if (v.primary != null) {
            f.push(`    PRIMARY KEY (${v.primary.join(", ")})`)
        }
        let fs = f.join(",\n")
        let s = `CREATE TABLE ${k} (\n${fs}\n);`
        stmt.push(s)
    }
    return stmt
}


const argx = {
    options: {
        gen: { type: 'boolean' },
        run: { type: 'boolean' },
        help: { type: 'boolean' }
    },
    allowPositionals: true
}
const args = parseArgs(argx)

if (args.values.help) {
    console.log(JSON.stringify(argx, null, 2))
    console.log(args)
} else if (args.values.gen) {
    let f = args.positionals[0]
    let tables = Bun.file(f)
    let t = load(await tables.text())
    let stmt = (await gen(t)).join("\n\n")
    console.log(stmt)
} else {
    console.log('require parameters')
}
