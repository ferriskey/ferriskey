use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::sync::Arc;

use ferriskey_security::crypto::secret_cipher::SecretCipher;
use futures::TryStreamExt;
use serde_json::{Map, Value};
use sqlx::{PgConnection, PgPool, Row};
use uuid::Uuid;

use super::tables::{SEALED_COLUMNS, TABLES, TableSpec, spec_of};
use crate::domain::common::entities::app_errors::CoreError;
use crate::domain::realm::entities::RealmId;
use crate::domain::realm_transfer::bundle::{BundleHeader, BundleRow};
use crate::domain::realm_transfer::ports::{RealmArchiveRepository, RowSink, RowSource};
use crate::domain::realm_transfer::value_objects::ImportReport;
use crate::infrastructure::token_state::TokenState;

const BATCH_ROWS: usize = 500;

#[derive(Debug, Default)]
struct TableShape {
    columns: BTreeSet<String>,
    primary_key: Vec<String>,
}

type Shapes = HashMap<String, TableShape>;

#[derive(Clone)]
pub struct PostgresRealmArchiveRepository {
    pool: PgPool,
    cipher: SecretCipher,
    changes: Option<Arc<TokenState>>,
}

impl std::fmt::Debug for PostgresRealmArchiveRepository {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PostgresRealmArchiveRepository")
            .finish_non_exhaustive()
    }
}

fn database(error: sqlx::Error) -> CoreError {
    CoreError::Database(error.to_string())
}

fn quote(identifier: &str) -> String {
    format!("\"{}\"", identifier.replace('"', "\"\""))
}

fn row_id(row: &Map<String, Value>) -> Option<String> {
    row.get("id").and_then(Value::as_str).map(str::to_owned)
}

fn applies(row: &Map<String, Value>, only_when: Option<(&str, &str)>) -> bool {
    only_when.is_none_or(|(column, value)| row.get(column).and_then(Value::as_str) == Some(value))
}

fn find_sealed(value: &Value, cipher: &SecretCipher) -> bool {
    match value {
        Value::String(text) => cipher.sealed_by_this_ring(text),
        Value::Array(items) => items.iter().any(|item| find_sealed(item, cipher)),
        Value::Object(fields) => fields.values().any(|field| find_sealed(field, cipher)),
        _ => false,
    }
}

fn import_error(table: &str, error: sqlx::Error) -> CoreError {
    let sqlx::Error::Database(failure) = &error else {
        return database(error);
    };
    let constraint = failure.constraint().unwrap_or("unnamed").to_owned();

    match failure.code().as_deref() {
        Some("23505") => CoreError::RealmBundleConflict(format!(
            "{table}: a row clashes with an existing row on {constraint}"
        )),
        Some("23503") => CoreError::InvalidRealmBundle(format!(
            "{table}: a row points at a row the bundle does not hold ({constraint})"
        )),
        Some(code) if code.starts_with("22") || code.starts_with("23") => {
            CoreError::InvalidRealmBundle(format!("{table}: {}", failure.message()))
        }
        _ => database(error),
    }
}

impl PostgresRealmArchiveRepository {
    pub fn new(pool: PgPool, cipher: SecretCipher) -> Self {
        Self {
            pool,
            cipher,
            changes: None,
        }
    }

    pub fn announcing_changes(mut self, changes: Arc<TokenState>) -> Self {
        self.changes = Some(changes);
        self
    }

    async fn shapes(connection: &mut PgConnection) -> Result<Shapes, CoreError> {
        let mut shapes = Shapes::new();

        let columns = sqlx::query(
            "SELECT table_name::text, column_name::text FROM information_schema.columns \
             WHERE table_schema = current_schema()",
        )
        .fetch_all(&mut *connection)
        .await
        .map_err(database)?;
        for row in columns {
            let table: String = row.get(0);
            shapes.entry(table).or_default().columns.insert(row.get(1));
        }

        let keys = sqlx::query(
            "SELECT c.relname::text, a.attname::text FROM pg_index i \
             JOIN pg_class c ON c.oid = i.indrelid \
             JOIN pg_namespace n ON n.oid = c.relnamespace \
             JOIN LATERAL unnest(i.indkey) WITH ORDINALITY AS k(attnum, position) ON true \
             JOIN pg_attribute a ON a.attrelid = c.oid AND a.attnum = k.attnum \
             WHERE i.indisprimary AND n.nspname = current_schema() \
             ORDER BY c.relname, k.position",
        )
        .fetch_all(&mut *connection)
        .await
        .map_err(database)?;
        for row in keys {
            let table: String = row.get(0);
            shapes
                .entry(table)
                .or_default()
                .primary_key
                .push(row.get(1));
        }

        Ok(shapes)
    }

    fn select_sql(spec: &TableSpec, shape: &TableShape) -> String {
        let scope = spec.scope_on("t", "$1");
        let order = shape
            .primary_key
            .iter()
            .map(|column| format!("t.{}", quote(column)))
            .collect::<Vec<_>>()
            .join(", ");

        match spec.tree_parent {
            None => format!(
                "SELECT row_to_json(t)::text FROM {table} t WHERE {scope} ORDER BY {order}",
                table = spec.name,
            ),
            Some(parent) => format!(
                "WITH RECURSIVE tree AS ( \
                   SELECT t.id, 0 AS depth FROM {table} t WHERE {scope} AND t.{parent} IS NULL \
                   UNION ALL \
                   SELECT c.id, tree.depth + 1 FROM {table} c JOIN tree ON c.{parent} = tree.id \
                 ) \
                 SELECT row_to_json(t)::text FROM {table} t JOIN tree ON tree.id = t.id \
                 ORDER BY tree.depth, {order}",
                table = spec.name,
                parent = quote(parent),
            ),
        }
    }

    fn open_secrets(&self, table: &str, row: &mut Map<String, Value>) -> Result<(), CoreError> {
        for sealed in SEALED_COLUMNS.iter().filter(|sealed| sealed.table == table) {
            if !applies(row, sealed.only_when) {
                continue;
            }
            let (Some(id), Some(Value::String(stored))) = (row_id(row), row.get(sealed.column))
            else {
                continue;
            };
            let opened = self
                .cipher
                .open(sealed.purpose, &id, stored)
                .map_err(|error| {
                    CoreError::InvalidRealmBundle(format!(
                        "{table}.{} could not be opened: {error}",
                        sealed.column
                    ))
                })?;
            row.insert(sealed.column.to_owned(), Value::String(opened));
        }

        if let Some((column, _)) = row
            .iter()
            .find(|(_, value)| find_sealed(value, &self.cipher))
        {
            return Err(CoreError::InvalidRealmBundle(format!(
                "{table}.{column} holds a sealed value the export does not know how to open"
            )));
        }
        Ok(())
    }

    fn seal_secrets(&self, table: &str, row: &mut Map<String, Value>) -> Result<(), CoreError> {
        for sealed in SEALED_COLUMNS.iter().filter(|sealed| sealed.table == table) {
            if !applies(row, sealed.only_when) {
                continue;
            }
            let (Some(id), Some(Value::String(plain))) = (row_id(row), row.get(sealed.column))
            else {
                continue;
            };
            let stored = self
                .cipher
                .seal(sealed.purpose, &id, plain)
                .map_err(|error| CoreError::Database(error.to_string()))?;
            row.insert(sealed.column.to_owned(), Value::String(stored));
        }
        Ok(())
    }
}

struct Import<'a> {
    archive: &'a PostgresRealmArchiveRepository,
    realm_id: Uuid,
    shapes: Shapes,
    table: Option<&'static TableSpec>,
    batch: Vec<Map<String, Value>>,
    rows: BTreeMap<String, u64>,
    ignored: BTreeSet<String>,
}

impl Import<'_> {
    fn spec_for(&self, row: &BundleRow) -> Result<&'static TableSpec, CoreError> {
        spec_of(&row.table).ok_or_else(|| {
            CoreError::InvalidRealmBundle(format!("table {:?} cannot be imported", row.table))
        })
    }

    fn stage(&mut self, spec: &'static TableSpec, row: BundleRow) -> Result<(), CoreError> {
        let shape = self.shapes.get(spec.name).ok_or_else(|| {
            CoreError::InvalidRealmBundle(format!("table {} does not exist here", spec.name))
        })?;

        let mut values = row.row;
        values.retain(|column, _| {
            if spec.dropped.contains(&column.as_str()) {
                return false;
            }
            let known = shape.columns.contains(column);
            if !known {
                self.ignored.insert(format!("{}.{column}", spec.name));
            }
            known
        });
        self.archive.seal_secrets(spec.name, &mut values)?;

        self.table = Some(spec);
        self.batch.push(values);
        Ok(())
    }

    async fn flush_batch(&mut self, connection: &mut PgConnection) -> Result<(), CoreError> {
        let Some(spec) = self.table else {
            return Ok(());
        };
        let rows = std::mem::replace(&mut self.batch, Vec::with_capacity(BATCH_ROWS));
        self.flush(spec, rows, connection).await
    }

    async fn flush(
        &mut self,
        spec: &TableSpec,
        rows: Vec<Map<String, Value>>,
        connection: &mut PgConnection,
    ) -> Result<(), CoreError> {
        if rows.is_empty() {
            return Ok(());
        }
        let shape = &self.shapes[spec.name];
        let columns: BTreeSet<&str> = rows
            .iter()
            .flat_map(|row| row.keys().map(String::as_str))
            .collect();
        let conflict: Vec<&str> = match spec.conflict {
            Some(columns) => columns.to_vec(),
            None => shape.primary_key.iter().map(String::as_str).collect(),
        };
        let listed = columns
            .iter()
            .map(|column| quote(column))
            .collect::<Vec<_>>()
            .join(", ");
        let mut updates: Vec<String> = columns
            .iter()
            .filter(|column| !conflict.contains(column))
            .map(|column| format!("{0} = EXCLUDED.{0}", quote(column)))
            .collect();
        if updates.is_empty() {
            updates.push(format!("{0} = EXCLUDED.{0}", quote(conflict[0])));
        }

        let sql = format!(
            "INSERT INTO {table} AS t ({listed}) \
             SELECT {listed} FROM jsonb_populate_recordset(NULL::{table}, $1::jsonb) AS r \
             WHERE {incoming} \
             ON CONFLICT ({target}) DO UPDATE SET {updates} \
             WHERE {existing} \
             RETURNING 1",
            table = spec.name,
            incoming = spec.scope_on("r", "$2"),
            target = conflict
                .iter()
                .map(|c| quote(c))
                .collect::<Vec<_>>()
                .join(", "),
            updates = updates.join(", "),
            existing = spec.scope_on("t", "$2"),
        );

        let expected = rows.len();
        let written = sqlx::query(&sql)
            .bind(sqlx::types::Json(rows))
            .bind(self.realm_id)
            .fetch_all(&mut *connection)
            .await
            .map_err(|error| import_error(spec.name, error))?
            .len();
        if written != expected {
            return Err(CoreError::RealmBundleConflict(format!(
                "{}: {} rows belong to another realm or point outside this one",
                spec.name,
                expected - written
            )));
        }

        *self.rows.entry(spec.name.to_owned()).or_default() += expected as u64;
        Ok(())
    }
}

impl RealmArchiveRepository for PostgresRealmArchiveRepository {
    async fn schema_version(&self) -> Result<i64, CoreError> {
        sqlx::query_scalar("SELECT COALESCE(MAX(version), 0) FROM _sqlx_migrations WHERE success")
            .fetch_one(&self.pool)
            .await
            .map_err(database)
    }

    async fn export_rows<K: RowSink>(
        &self,
        realm_id: RealmId,
        sink: &mut K,
    ) -> Result<(), CoreError> {
        let realm_id = Uuid::from(realm_id);
        let mut tx = self.pool.begin().await.map_err(database)?;
        sqlx::query("SET TRANSACTION ISOLATION LEVEL REPEATABLE READ, READ ONLY")
            .execute(&mut *tx)
            .await
            .map_err(database)?;
        let shapes = Self::shapes(&mut tx).await?;

        for spec in TABLES {
            let shape = shapes.get(spec.name).ok_or_else(|| {
                CoreError::Database(format!("table {} is missing from the schema", spec.name))
            })?;
            let sql = Self::select_sql(spec, shape);
            let mut rows = sqlx::query_scalar::<_, String>(&sql)
                .bind(realm_id)
                .fetch(&mut *tx);

            while let Some(text) = rows.try_next().await.map_err(database)? {
                let mut row: Map<String, Value> = serde_json::from_str(&text)
                    .map_err(|error| CoreError::Database(error.to_string()))?;
                for dropped in spec.dropped {
                    row.remove(*dropped);
                }
                self.open_secrets(spec.name, &mut row)?;
                sink.accept(BundleRow {
                    table: spec.name.to_owned(),
                    row,
                })
                .await?;
            }
        }

        tx.commit().await.map_err(database)
    }

    async fn import_rows<S: RowSource>(
        &self,
        header: &BundleHeader,
        source: &mut S,
    ) -> Result<ImportReport, CoreError> {
        let realm_id = header.realm.id;
        let mut tx = self.pool.begin().await.map_err(database)?;
        sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended($1::text, 1609))")
            .bind(realm_id)
            .execute(&mut *tx)
            .await
            .map_err(database)?;
        let existed: bool =
            sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM realms WHERE id = $1)")
                .bind(realm_id)
                .fetch_one(&mut *tx)
                .await
                .map_err(database)?;

        let mut import = Import {
            archive: self,
            realm_id,
            shapes: Self::shapes(&mut tx).await?,
            table: None,
            batch: Vec::with_capacity(BATCH_ROWS),
            rows: BTreeMap::new(),
            ignored: BTreeSet::new(),
        };

        while let Some(row) = source.next_row().await? {
            let spec = import.spec_for(&row)?;
            let switching = import
                .table
                .is_some_and(|current| current.name != spec.name);
            if switching || import.batch.len() == BATCH_ROWS {
                import.flush_batch(&mut tx).await?;
            }
            import.stage(spec, row)?;
        }
        import.flush_batch(&mut tx).await?;

        let active_keys: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM jwt_keys \
             WHERE realm_id = $1 AND rotated_at IS NULL AND revoked_at IS NULL",
        )
        .bind(realm_id)
        .fetch_one(&mut *tx)
        .await
        .map_err(database)?;
        if active_keys != 1 {
            return Err(CoreError::RealmBundleWithoutKeys);
        }

        tx.commit().await.map_err(database)?;
        if let Some(changes) = &self.changes {
            changes.changed();
        }

        Ok(ImportReport {
            realm_id,
            realm_name: header.realm.name.clone(),
            created: !existed,
            rows: import.rows,
            ignored_columns: import.ignored.into_iter().collect(),
        })
    }
}
