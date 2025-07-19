import { sql } from "bun"
import { load } from 'js-toml'
import { parseArgs } from 'util'


export const schema = async () => {
    let x = await sql`
        WITH ct AS (
            SELECT ccu.table_schema, ccu.table_name, ccu.column_name, tc.constraint_type IS NOT NULL AS pk
            FROM information_schema.table_constraints AS tc
            JOIN information_schema.constraint_column_usage AS ccu
            ON tc.constraint_schema = ccu.constraint_schema
                AND tc.constraint_name = ccu.constraint_name
            WHERE tc.constraint_type = 'PRIMARY KEY'
        ) SELECT co.table_schema, co.table_name, co.column_name, co.is_nullable, co.data_type, COALESCE(ct.pk, false) AS pk
        FROM information_schema.columns AS co
        LEFT OUTER JOIN ct
        ON co.table_schema = ct.table_schema
          AND co.table_name = ct.table_name
          AND co.column_name = ct.column_name
        WHERE co.table_schema not in ('information_schema', 'pg_catalog')
    `
    let r: any = {}
    for (let i of x) {
        let tn = i.table_schema == 'public' ? i.table_name : [i.table_schema, i.table_name].join('.')
        if (!(tn in r)) {
            r[tn] = {
                column: {},
                primary: [],
                index: {}
            }
        }
        if (!(i.column_name in r[tn].column)) {
            r[tn].column[i.column_name] = {}
        }
        r[tn].column[i.column_name].type = i.data_type
        r[tn].column[i.column_name].nullable = i.is_nullable == 'YES'
        if (i.pk) {
            r[tn].primary.push(i.column_name)
        }
    }
    return r
}

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
            'jsonb': `'${defs.default}'::JSONB`,
        }
        if (defs.type in m) {
            r.push(`DEFAULT ${m[defs.type]}`)
        } else {
            r.push(`DEFAULT ${defs.default}`)
        }

    }
    if (defs.uniq) {
        r.push('UNIQUE')
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
        for (let i of (v.index ?? [])) {
            let n = [`idx_${k}_`, ...i.column].join('_')
            let c = i.type == null ? [] : [`USING ${i.type.toUpperCase()}`]
            c.push(`(${i.column.join(', ')})`)
            if (i.include != null) {
                c.push(`INCLUDE (${i.include.join(', ')})`)
            }
            if (i.with != null) {
                let o = Object.entries(i.with).reduce((a, x) => {a.push(`${x[0]} = ${x[1]}`); return a}, []).join(', ')
                c.push(`WITH (${o})`)
            }
            if (i.where != null) {
                c.push(`WHERE ${i.where}`)
            }
            stmt.push(`CREATE INDEX ${n} ON ${k} ${c.join(' ')};`)

        }
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
} else if (args.values.run) {
    for (let s of (await gen())) {
        console.log(s)
        await sql.unsafe(s)
    }
} else if (args.values.gen) {
    let f = args.positionals[0]
    let tables = Bun.file(f)
    let t = load(await tables.text())
    let stmt = (await gen(t)).join("\n\n")
    console.log(stmt)
} else {
    console.log('require parameters')
}

