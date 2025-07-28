import { sql } from "bun"

export const insert = async (x, trunc: boolean) => {
    for (let i of Object.entries(x)) {
        let [name, data] = i
        if (trunc) {
            let s = `TRUNCATE ${name} CASCADE;`
            console.log(s)
            await sql.unsafe(s)
            continue
        }
        let flow = -1
        for (let d of data) {
            if (['edge'].indexOf(name) >= 0) {
                let s = `INSERT INTO edge (flow_id, income, outgo, data)
                VALUES (
                    (select id from flow where name = $1 limit 1),
                    (select id from node where name = $2 limit 1),
                    (select id from node where name = $3 limit 1),
                    $4
                ) RETURNING flow_id;
                `
                let [{ flow_id }] = await sql.unsafe(s, [d.flow_id, d.income, d.outgo, d.data])
                if (flow_id != flow) {
                    flow = flow_id
                    let s = `UPDATE flow set head = (SELECT id from node where name = $1) where id = $2`
                    await sql.unsafe(s, [d.income, flow])
                }
            } else {
                let k = Object.keys(d)
                let val = k.map((_, ix) => `$${ix + 1}`).join(', ')
                let col = k.join(', ')
                let s = `INSERT INTO ${name} (${col}) VALUES (${val});`
                console.log(s)
                await sql.unsafe(s, Object.values(d))
            }
        }
    }
}
