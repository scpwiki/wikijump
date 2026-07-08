import { spawnSync } from 'node:child_process';

import pg from 'pg';

const { Client } = pg;

function sqlPreview(sql) {
  return sql.length > 2000 ? `${sql.slice(0, 2000)}\n... <truncated ${sql.length - 2000} bytes>` : sql;
}

function sqlErrorMessage(status, stdout, stderr, sql) {
  return `psql failed (${status})\nSTDOUT:\n${stdout}\nSTDERR:\n${stderr}\nSQL preview:\n${sqlPreview(sql)}`;
}

export function formatCaptureOutput(results) {
  const resultList = Array.isArray(results) ? results : [results];
  const lines = [];
  for (const result of resultList) {
    if (!Array.isArray(result?.rows) || result.rows.length === 0) continue;
    const fields = Array.isArray(result.fields) ? result.fields : [];
    for (const row of result.rows) {
      const values = Array.isArray(row)
        ? row
        : fields.map((field) => row[field.name]);
      lines.push(values.map((value) => value ?? '').join('|'));
    }
  }
  return lines.join('\n').trim();
}

export function createSqlExecutor({ dbUrl, dbContainer }) {
  if (dbUrl) {
    let client = null;

    async function ensureClient() {
      if (client !== null) return client;
      const newClient = new Client({
        connectionString: dbUrl,
        types: { getTypeParser: () => (value) => value },
      });
      await newClient.connect();
      client = newClient;
      return client;
    }

    return {
      mode: 'pg',
      async runSql(sql, { capture = false } = {}) {
        let activeClient = null;
        try {
          activeClient = await ensureClient();
          await activeClient.query('BEGIN');
          const results = await activeClient.query(sql);
          await activeClient.query('COMMIT');
          return capture ? formatCaptureOutput(results) : null;
        } catch (error) {
          if (activeClient !== null) {
            try {
              await activeClient.query('ROLLBACK');
            } catch {
              // Keep the original query error as the actionable failure.
            }
          }
          throw new Error(sqlErrorMessage('pg', '', error.stack || error.message, sql));
        }
      },
      async close() {
        if (client !== null) {
          await client.end();
          client = null;
        }
      },
    };
  }

  return {
    mode: 'docker-psql',
    async runSql(sql, { capture = false } = {}) {
      const result = spawnSync(
        'docker',
        ['exec', '-i', '-e', 'PGPASSWORD=wikijump', dbContainer, 'psql', '-h', 'localhost', '-U', 'wikijump', '-d', 'wikijump', '-1', '-v', 'ON_ERROR_STOP=1', '-q', '-t', '-A'],
        { input: sql, encoding: 'utf8' },
      );
      if (result.status !== 0) {
        throw new Error(sqlErrorMessage(result.status, result.stdout, result.stderr, sql));
      }
      return capture ? result.stdout.trim() : null;
    },
    async close() {},
  };
}
