use sea_orm::{
    ColumnTrait, ConnectionTrait, DbErr, EntityTrait, Iterable, Order, PaginatorTrait,
    PrimaryKeyToColumn, QueryOrder, QuerySelect, Select,
    sea_query::{Expr, LikeExpr, SimpleExpr, extension::postgres::PgExpr},
};

use crate::domain::common::pagination::{PageRequest, SortOrder};

pub trait SortColumn<E: EntityTrait> {
    fn column(&self) -> E::Column;
}

pub fn escape_like(value: &str) -> String {
    let mut pattern = String::with_capacity(value.len() + 2);
    pattern.push('%');
    for c in value.chars() {
        if matches!(c, '%' | '_' | '\\') {
            pattern.push('\\');
        }
        pattern.push(c);
    }
    pattern.push('%');
    pattern
}

pub fn contains<C: ColumnTrait>(column: C, value: &str) -> SimpleExpr {
    Expr::col((column.entity_name(), column)).ilike(LikeExpr::new(escape_like(value)).escape('\\'))
}

pub fn order<E, F, S>(select: Select<E>, request: &PageRequest<F, S>) -> Select<E>
where
    E: EntityTrait,
    S: SortColumn<E>,
{
    let direction = match request.sort.order {
        SortOrder::Asc => Order::Asc,
        SortOrder::Desc => Order::Desc,
    };
    E::PrimaryKey::iter().fold(
        select.order_by(request.sort.field.column(), direction.clone()),
        |select, key| select.order_by(key.into_column(), direction.clone()),
    )
}

pub async fn paginate<E, F, S, C>(
    db: &C,
    select: Select<E>,
    request: &PageRequest<F, S>,
) -> Result<(Vec<E::Model>, u64), DbErr>
where
    E: EntityTrait,
    E::Model: Sync,
    S: SortColumn<E>,
    C: ConnectionTrait,
{
    let total = select.clone().count(db).await?;
    let models = order(select, request)
        .offset(request.offset())
        .limit(u64::from(request.limit.get()))
        .all(db)
        .await?;
    Ok((models, total))
}

#[cfg(test)]
mod tests {
    use sea_orm::{DbBackend, EntityTrait, QueryFilter, QueryTrait};

    use super::*;
    use crate::domain::common::pagination::{PageLimit, PageNumber, PageRequest, Sort, SortOrder};
    use crate::entity::realms;

    #[derive(Debug, Clone, Copy, Default)]
    enum TestSort {
        Name,
        #[default]
        CreatedAt,
    }

    impl SortColumn<realms::Entity> for TestSort {
        fn column(&self) -> realms::Column {
            match self {
                TestSort::Name => realms::Column::Name,
                TestSort::CreatedAt => realms::Column::CreatedAt,
            }
        }
    }

    fn request(field: TestSort, order: SortOrder) -> PageRequest<(), TestSort> {
        PageRequest {
            page: PageNumber::default(),
            limit: PageLimit::default(),
            sort: Sort { field, order },
            filter: (),
        }
    }

    #[test]
    fn escape_like_wraps_and_escapes() {
        assert_eq!(escape_like("jo"), "%jo%");
        assert_eq!(escape_like("50%_a\\b"), "%50\\%\\_a\\\\b%");
    }

    #[test]
    fn order_appends_primary_key_in_the_same_direction() {
        let sql = order(
            realms::Entity::find(),
            &request(TestSort::Name, SortOrder::Asc),
        )
        .build(DbBackend::Postgres)
        .to_string();
        assert!(
            sql.ends_with(r#"ORDER BY "realms"."name" ASC, "realms"."id" ASC"#),
            "{sql}"
        );
    }

    #[test]
    fn default_order_is_created_at_desc() {
        let sql = order(
            realms::Entity::find(),
            &request(TestSort::default(), SortOrder::default()),
        )
        .build(DbBackend::Postgres)
        .to_string();
        assert!(
            sql.ends_with(r#"ORDER BY "realms"."created_at" DESC, "realms"."id" DESC"#),
            "{sql}"
        );
    }

    #[test]
    fn contains_is_a_qualified_case_insensitive_escaped_match() {
        let sql = realms::Entity::find()
            .filter(contains(realms::Column::Name, "a_b"))
            .build(DbBackend::Postgres)
            .to_string();
        assert!(sql.contains(r#""realms"."name" ILIKE"#), "{sql}");
        assert!(sql.contains("ESCAPE"), "{sql}");
        assert!(sql.contains(r"E'%a\\_b%' ESCAPE E'\\'"), "{sql}");
    }
}
