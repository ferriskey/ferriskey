use sea_orm::{
    ColumnTrait, ConnectionTrait, DbErr, EntityTrait, Iterable, Order, PaginatorTrait,
    PrimaryKeyToColumn, QueryOrder, QuerySelect, Select,
    sea_query::{Expr, LikeExpr, SimpleExpr, extension::postgres::PgExpr},
};

use crate::domain::common::pagination::{PageRequest, SortOrder};

pub trait SortColumn<E: EntityTrait> {
    fn column(&self) -> E::Column;
}

pub trait SortExpr {
    fn expr(&self) -> SimpleExpr;
}

pub fn direction(order: SortOrder) -> Order {
    match order {
        SortOrder::Asc => Order::Asc,
        SortOrder::Desc => Order::Desc,
    }
}

fn window<Q: QuerySelect, F, S>(query: Q, request: &PageRequest<F, S>) -> Q {
    query
        .offset(request.offset())
        .limit(u64::from(request.limit.get()))
}

pub fn page_by_expr<Q, F, S>(query: Q, request: &PageRequest<F, S>, tie_breaker: SimpleExpr) -> Q
where
    Q: QueryOrder + QuerySelect,
    S: SortExpr,
{
    let order = direction(request.sort.order);
    window(
        query
            .order_by(request.sort.field.expr(), order.clone())
            .order_by(tie_breaker, order),
        request,
    )
}

pub async fn paginate_also<E, R, F, S, C>(
    db: &C,
    select: Select<E>,
    related: R,
    tie_breaker: SimpleExpr,
    request: &PageRequest<F, S>,
) -> Result<(Vec<(E::Model, Option<R::Model>)>, u64), DbErr>
where
    E: EntityTrait,
    E::Model: Sync,
    R: EntityTrait,
    R::Model: Sync,
    S: SortExpr,
    C: ConnectionTrait,
{
    let total = select.clone().count(db).await?;
    let rows = page_by_expr(select.select_also(related), request, tie_breaker)
        .all(db)
        .await?;
    Ok((rows, total))
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
    Expr::col((column.entity_name(), column)).ilike(LikeExpr::new(escape_like(value)))
}

pub fn order<E, F, S>(select: Select<E>, request: &PageRequest<F, S>) -> Select<E>
where
    E: EntityTrait,
    S: SortColumn<E>,
{
    let order = direction(request.sort.order);
    E::PrimaryKey::iter().fold(
        select.order_by(request.sort.field.column(), order.clone()),
        |select, key| select.order_by(key.into_column(), order.clone()),
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
    let models = window(order(select, request), request).all(db).await?;
    Ok((models, total))
}

#[cfg(test)]
mod tests {
    use sea_orm::{DbBackend, EntityTrait, QueryFilter, QueryTrait};

    use super::*;
    use crate::domain::common::pagination::{PageLimit, PageNumber, PageRequest, Sort, SortOrder};
    use crate::entity::{realms, users};

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

    impl SortExpr for TestSort {
        fn expr(&self) -> SimpleExpr {
            match self {
                TestSort::Name => Expr::col((users::Entity, users::Column::Username)).into(),
                TestSort::CreatedAt => {
                    Expr::col((realms::Entity, realms::Column::CreatedAt)).into()
                }
            }
        }
    }

    #[test]
    fn page_by_expr_orders_by_the_expression_then_the_tie_breaker_and_pages() {
        let request = PageRequest {
            page: PageNumber::try_from(3).expect("valid page"),
            limit: PageLimit::try_from(10).expect("valid limit"),
            sort: Sort {
                field: TestSort::Name,
                order: SortOrder::Asc,
            },
            filter: (),
        };
        let sql = page_by_expr(
            realms::Entity::find().select_also(users::Entity),
            &request,
            Expr::col((realms::Entity, realms::Column::Id)).into(),
        )
        .build(DbBackend::Postgres)
        .to_string();
        assert!(
            sql.contains(r#""users"."username" AS "B_username""#),
            "{sql}"
        );
        assert!(
            sql.ends_with(
                r#"ORDER BY "users"."username" ASC, "realms"."id" ASC LIMIT 10 OFFSET 20"#
            ),
            "{sql}"
        );

        let sql = page_by_expr(
            realms::Entity::find(),
            &self::request(TestSort::CreatedAt, SortOrder::Desc),
            Expr::col((realms::Entity, realms::Column::Id)).into(),
        )
        .build(DbBackend::Postgres)
        .to_string();
        assert!(
            sql.ends_with(
                r#"ORDER BY "realms"."created_at" DESC, "realms"."id" DESC LIMIT 20 OFFSET 0"#
            ),
            "{sql}"
        );
    }

    #[test]
    fn contains_is_a_qualified_case_insensitive_escaped_match() {
        let sql = realms::Entity::find()
            .filter(contains(realms::Column::Name, "a_b"))
            .build(DbBackend::Postgres)
            .to_string();
        assert!(sql.contains(r#""realms"."name" ILIKE E'%a\\_b%'"#), "{sql}");
        assert!(!sql.contains("ESCAPE"), "{sql}");
    }
}
