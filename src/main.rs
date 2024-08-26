use chrono::NaiveDateTime;
use std::env;
use std::fs;
use std::str::FromStr;


use chrono::TimeZone;
use serde_derive::Deserialize;
use serde_derive::Serialize;
use serde_json::Value;
use sqlx::sqlite::SqliteConnectOptions;
use sqlx::sqlite::SqliteJournalMode;
use sqlx::SqlitePool;
use v8;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LumaServiceRegions {
    pub regions: Vec<Region>,
    pub totals: Totals,
    pub timestamp: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Region {
    pub name: String,
    pub total_clients: i64,
    pub total_clients_without_service: i64,
    pub total_clients_with_service: i64,
    pub total_clients_affected_by_planned_outage: i64,
    pub percentage_clients_without_service: f64,
    pub percentage_clients_with_service: f64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Totals {
    pub total_clients_without_service: i64,
    pub total_clients: i64,
    pub total_clients_with_service: i64,
    pub total_percentage_without_service: f64,
    pub total_clients_affected_by_planned_outage: i64,
    pub total_percentage_with_service: f64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GenerationDataFrame {
    pub data_fecha_acualizado: String,
    pub data_fuel_cost: Vec<DataFuelCost>,
    pub data_by_fuel: Vec<DataByFuel>,
    pub data_metrics: Vec<DataMetric>,
    pub data_load_per_site: Vec<DataLoadPerSite>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DataFuelCost {
    pub place: String,
    pub value: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DataByFuel {
    pub fuel: String,
    pub value: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DataMetric {
    #[serde(rename(serialize = "index", deserialize = "Index"))]
    pub index: String,
    #[serde(rename(serialize = "desc", deserialize = "Desc"))]
    pub desc: String,
    pub value: Value,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DataLoadPerSite {
    #[serde(rename(serialize = "index", deserialize = "Index"))]
    pub index: String,
    #[serde(rename(serialize = "type_field", deserialize = "Type"))]
    pub type_field: String,
    #[serde(rename(serialize = "desc", deserialize = "Desc"))]
    pub desc: String,
    #[serde(rename(serialize = "site_total", deserialize = "SiteTotal"))]
    pub site_total: i64,

    pub units: Vec<Unit>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Unit {
    #[serde(default)]
    pub load_per_site_index: String,
    #[serde(rename(serialize = "index", deserialize = "Index"))]
    pub index: String,
    #[serde(rename(serialize = "unit", deserialize = "Unit"))]
    pub unit: String,
    #[serde(rename(serialize = "mw", deserialize = "MW"))]
    pub mw: i64,
    #[serde(rename(serialize = "mvar", deserialize = "MVar"))]
    pub mvar: String,
    #[serde(rename(serialize = "cost", deserialize = "Cost"))]
    pub cost: f64,
    #[serde(rename(serialize = "parent_id", deserialize = "ParentId"))]
    pub parent_id: String,
}

const JS_SNIPPET: &str = "
    JSON.stringify({
        dataFechaAcualizado,
        dataFuelCost,
        dataByFuel,
        dataMetrics,
        dataLoadPerSite
    });
";

async fn insert_luma_data(
    pool: &SqlitePool,
    lumadata: &LumaServiceRegions,
) -> Result<(), anyhow::Error> {
    let data_fecha_acualizado = NaiveDateTime::parse_from_str(
        lumadata.timestamp.as_str(),
        "%m/%d/%Y %I:%M %p",
    )
    .expect(&format!("Parsing date {}", lumadata.timestamp));
    let ast_tz = chrono_tz::America::Puerto_Rico; // Puerto Rico is in AST/ADT
    let local_dt = ast_tz
        .from_local_datetime(&data_fecha_acualizado)
        .single()
        .expect("Failed to convert to local time");

    let timestamp = local_dt.timestamp();
    for region in &lumadata.regions {
        sqlx::query!(
            "INSERT INTO RegionData (time, name, total_clients, total_clients_without_service, total_clients_with_service, total_clients_affected_by_planned_outage, percentage_clients_without_service, percentage_clients_with_service) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
            timestamp,
            region.name,
            region.total_clients,
            region.total_clients_without_service,
            region.total_clients_with_service,
            region.total_clients_affected_by_planned_outage,
            region.percentage_clients_without_service,
            region.percentage_clients_with_service
        )
        .execute(pool)
        .await
        .unwrap_or_default();
    }

    // Insert data into Totals
    sqlx::query!(
        "INSERT INTO Totals (time, total_clients_without_service, total_clients, total_clients_with_service, total_percentage_without_service, total_clients_affected_by_planned_outage, total_percentage_with_service) VALUES ($1, $2, $3, $4, $5, $6, $7)",
        timestamp,
        lumadata.totals.total_clients_without_service,
        lumadata.totals.total_clients,
        lumadata.totals.total_clients_with_service,
        lumadata.totals.total_percentage_without_service,
        lumadata.totals.total_clients_affected_by_planned_outage,
        lumadata.totals.total_percentage_with_service
    )
    .execute(pool)
    .await
    .unwrap_or_default();
    Ok(())
}

async fn insert_generation_data(
    pool: &SqlitePool,
    dataframe: &GenerationDataFrame,
) -> Result<(), anyhow::Error> {
    let data_fecha_acualizado = NaiveDateTime::parse_from_str(
        dataframe.data_fecha_acualizado.as_str(),
        "%m/%d/%Y %I:%M:%S %p",
    )
    .expect("Parsing date");
    let ast_tz = chrono_tz::America::Puerto_Rico; // Puerto Rico is in AST/ADT
    let local_dt = ast_tz
        .from_local_datetime(&data_fecha_acualizado)
        .single()
        .expect("Failed to convert to local time");

    let data_fecha_acualizado = local_dt.timestamp();
    sqlx::query!(
        "INSERT INTO GenerationData (data_fecha_acualizado) VALUES ($1)",
        data_fecha_acualizado,
    )
    .execute(pool)
    .await
    .expect("inserting generation data");

    for v in &dataframe.data_fuel_cost {
        sqlx::query!(
            "INSERT INTO FuelCost (data_fecha_acualizado, place, value) VALUES ($1, $2, $3)",
            data_fecha_acualizado,
            v.place,
            v.value
        )
        .execute(pool)
        .await
        .expect("inserting fuel cost");
    }

    for v in &dataframe.data_metrics {
        sqlx::query!(
            "INSERT INTO Metrics (data_fecha_acualizado, \"index\", Desc, value) VALUES ($1, $2, $3, $4)",
            data_fecha_acualizado, v.index, v.desc, v.value
        ).execute(pool).await.expect("inserting metrics");
    }

    for v in &dataframe.data_by_fuel {
        sqlx::query!(
            "INSERT INTO ByFuel (data_fecha_acualizado, fuel, value) VALUES ($1, $2, $3)",
            data_fecha_acualizado,
            v.fuel,
            v.value
        )
        .execute(pool)
        .await
        .expect("inserting by fuel");
    }

    for v in &dataframe.data_load_per_site {
        sqlx::query!(
            "INSERT INTO LoadPerSite (data_fecha_acualizado, \"index\", type_field, desc, site_total) VALUES ($1, $2, $3, $4, $5)",
            data_fecha_acualizado, v.index, v.type_field, v.desc, v.site_total
        ).execute(pool).await.expect("inserting load per site");
        for u in &v.units {
            sqlx::query!(
                    "INSERT INTO Units (data_fecha_acualizado, load_per_site_index, \"index\", unit, mw, mvar, cost, parent_id) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
                    data_fecha_acualizado, v.index, u.index, u.unit, u.mw, u.mvar, u.cost, u.parent_id
                ).execute(pool).await.expect("inserting units");
        }
    }
    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> { 
    match env::var("DOT_ENV") {
        Ok(fp) => {
            dotenv::from_filename(fp).expect("environment file not found");
        }
        Err(why) => {},
    };
    let conn =
        SqliteConnectOptions::from_str(&env::var("DATABASE_URL").expect("db file path exists"))
            .expect("Sqlite database opened")
            .journal_mode(SqliteJournalMode::Wal)
            .create_if_missing(true);
    let pool = SqlitePool::connect_with(conn).await.expect("Connected");
    
    if env::var("CREATE").unwrap_or_default() == "1" {
        let body = fs::read_to_string("tables.sql").expect("text read");
        let mut p = pool.acquire().await.expect("");
        for create in body.split(";") {
            sqlx::query(create)
                .execute(&mut *p)
                .await
                .expect("tables created");
        }
        return Ok(());
    }

    if env::var("UPDATE").unwrap_or_default() == "1" {
        let platform = v8::new_default_platform(0, false).make_shared();
        v8::V8::initialize_platform(platform);
        v8::V8::initialize();
        let isolate = &mut v8::Isolate::new(Default::default());
        let scope = &mut v8::HandleScope::new(isolate);
        let context = v8::Context::new(scope, Default::default());
        let scope = &mut v8::ContextScope::new(scope, context);
                
        let body: String = reqwest::get("https://api.miluma.lumapr.com/miluma-outage-api/outage/regionsWithoutService")
        .await
        .expect("javascript file downloaded")
        .text()
        .await
        .expect("text extracted");
        let luma_service_regions: LumaServiceRegions = serde_json::from_str(body.as_str()).expect("unmarshal luma");

        insert_luma_data(&pool, &luma_service_regions).await.expect("inserted luma data");
        let body = reqwest::get("https://operationdata.prepa.pr.gov/dataSource.js")
            .await
            .expect("javascript file downloaded")
            .text()
            .await
            .expect("text extracted") 
            .replace('\n', "")
            .replace('\t', "");
        let js = &format!("{} {}", body.as_str(), JS_SNIPPET);
        let code = v8::String::new(scope, js).unwrap();

        let script = v8::Script::compile(scope, code, None).unwrap();
        let result = script.run(scope).unwrap();
        let result = result.to_string(scope).unwrap();

        let json = result.to_rust_string_lossy(scope);
        let dataframe: GenerationDataFrame =
            serde_json::from_str(json.as_str()).expect("json parsed");
        insert_generation_data(&pool, &dataframe)
            .await
            .expect("inserted data");
        print!("Inserted {}\n", dataframe.data_fecha_acualizado);
        return Ok(());
    }
    println!("nothing ran\n");
    Ok(())
}
