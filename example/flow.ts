import { sql } from "bun"

export const get_flow = async (name: string) => {
    let r = await sql`
    WITH RECURSIVE f AS (
        SELECT name, id, head FROM flow WHERE name = ${name}
    ), e AS (
        SELECT e0.income, e0.outgo, e0.flow_id, e0.data
            , f.id AS fid
            , false AS is_cycle
            , array[ROW(e0.income, e0.outgo, e0.flow_id)] AS path
        FROM edge AS e0
        JOIN f ON e0.income = f.head
        WHERE e0.flow_id = f.id
        UNION ALL
        SELECT e1.income, e1.outgo, e1.flow_id, e1.data
            , fid
            , ROW(e1.income, e1.outgo, e1.flow_id) = ANY(path)
            , path || ROW(e1.income, e1.outgo, e1.flow_id)
        FROM e
        JOIN edge AS e1 ON e.outgo = e1.income
        WHERE e1.flow_id = fid
        AND NOT is_cycle
    ), e2 AS (
        SELECT DISTINCT i.name AS income, o.name AS outgo
            , e.data
        FROM e
        JOIN node AS i ON i.id = e.income
        JOIN node AS o ON o.id = e.outgo
    ), n1 AS (
        SELECT n.name, n.kind, n.data
        FROM e
        JOIN node AS n ON e.income = n.id
        UNION
        SELECT n.name, n.kind, n.data
        FROM e
        JOIN node AS n ON e.outgo = n.id
    ), eo AS (
        SELECT JSONB_AGG(
            JSONB_BUILD_OBJECT('in', e2.income, 'out', e2.outgo, 'data', e2.data)
        ) AS d FROM e2
    ), no AS (
        SELECT JSONB_OBJECT_AGG(
            n1.name,
            JSONB_BUILD_OBJECT('kind', n1.kind, 'data', n1.data)
        ) AS d FROM n1
    ), so AS (
        SELECT node.name FROM f
        JOIN node ON f.head = node.id
    ) SELECT JSONB_BUILD_OBJECT(
            'name', f.name,
            'head', so.name,
            'node', no.d,
            'edge', eo.d
        ) AS data
        FROM eo, no, so, f;
    `
    return r['0'].data
}

export const draw_flow = async (name: string) => {
    let x = await get_flow(name)
    let r = ["flowchart LR"]
    r.push(`    ${x.head}("${x.head}")`)
    for (let i of x.edge) {
        let _in = i.in
        let _out = i.out
        if (i.data != null) {
            r.push(`    ${_in} --${i.data.condition}--> ${_out}["${_out}"]`)
        } else {
            r.push(`    ${_in} --> ${_out}["${_out}"]`)
        }
    }
    return `<html>
  <body>
    <pre class="mermaid">
${r.join("\n")}
    </pre>
    <script type="module">
      import mermaid from 'https://cdn.jsdelivr.net/npm/mermaid@11/dist/mermaid.esm.min.mjs';
      mermaid.initialize({ startOnLoad: true });
    </script>
  </body>
</html>`
}


export type Edge = {
    in: string
    out: string
    data: object
    ready: boolean
}

export type Node = {
    kind: string
    data: Object
    ready: boolean
    positive: boolean
}

export type Flow = {
    name: string
    head: string
    edge: Edge[]
    node: Record<string, Node>
}

const run_node = async (
    name: string,
    node: Node,
    context: any,
    info: any
) => {
    console.log({ name, node, context, info })
    return {}
}

export const run_flow = async (flow: Flow, step = run_node, context = {}) => {
    let imap: Record<string, Edge[]> = {}
    let omap: Record<string, string[]> = {}
    for (let i of flow.edge) {
        if (!(i.out in imap)) {
            imap[i.out] = []
        }
        if (!(i.in in omap)) {
            omap[i.in] = []
        }
        imap[i.out].push(i)
        omap[i.in].push(i.out)
    }

    //console.log(flow)
    //console.log(imap)
    //console.log(omap)

    let _tid = await sql`
        INSERT INTO tasks (flow, context)
        SELECT id, ${context} FROM flow WHERE name = ${flow.name}
        RETURNING id
    `
    let tid = _tid['0'].id
    let nodes = flow.node
    let queue = [flow.head]
    let count = 0
    while (queue.length > 0) {
        let curr = queue.shift()
        if (curr == null) { break }

        let cn = nodes[curr]
        if (cn.ready) { continue }

        let no_ready = (imap[curr] ?? []).filter(x => !nodes[x.in].ready)
        if (no_ready.length > 0 && curr != flow.head) {
            if (false) {
                console.log(`[push back]task: ${tid}, curr: ${curr}`)
                queue.push(curr)
            }
            continue
        }
        // console.log(`task: ${tid}, curr: ${curr}`)

        await sql`
            INSERT INTO steps (task_id, node_id)
            SELECT ${tid}, id FROM node WHERE name = ${curr}
        `
        let d, err
        try {
            d = await step(curr, cn, context, {count, task_id: tid})
            cn.positive = true
        } catch (e: any) {
            err = { name: e.name, message: e.message, cause: e.cause }
            cn.positive = false
        }
        count += 1
        await sql`
            WITH step AS (
                UPDATE steps
                SET stop = now(), data = ${d}, error = ${err} FROM node
                WHERE node.NAME = ${curr} AND task_id = ${tid} AND node_id = node.id
            ) UPDATE tasks SET context = ${context} WHERE id = ${tid}
        `
        if (err != null) { break }

        cn.ready = true
        queue.push(...(omap[curr] ?? []))
    }
    await sql`UPDATE tasks SET stop = now() WHERE id = ${tid}`
}

export const list_tasks = async (q) => {
    let c = []
    if (q.incomplete === 'true') {
        c.push('t.stop IS NULL')
    }
    let w = c.length > 0 ? `WHERE ${c.join(' AND ')}` : ''
    let s = `
        select t.id, f.name, t.start, t.stop, t.context
        from tasks as t join flow as f on t.flow = f.id
        ${w}
        order by t.start desc
        limit 20
        `
    let d = await sql.unsafe(s)
    return d
}

export const get_task = async (id) => {
    let d = await sql`
        select n.name, n.kind, s.start, s.stop, s.data, s.error
        from steps as s join node as n on s.node_id = n.id
        where s.task_id = ${id}`
    return d
}
