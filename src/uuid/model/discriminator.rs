use serde::{Deserialize, Serialize};
use sqlx::database::HasArguments;
use sqlx::encode::IsNull;
use sqlx::mysql::MySqlTypeInfo;
use sqlx::MySql;

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Discriminator {
    Attachment,
    BlogPost,
    Comment,
    Entity,
    EntityRevision,
    Page,
    PageRevision,
    TaxonomyTerm,
    User,
}

impl std::str::FromStr for Discriminator {
    type Err = serde_json::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::from_value(serde_json::value::Value::String(s.to_string()))
    }
}

impl sqlx::Type<MySql> for Discriminator {
    fn type_info() -> MySqlTypeInfo {
        str::type_info()
    }
}
impl<'q> sqlx::Encode<'q, MySql> for Discriminator {
    fn encode_by_ref(&self, buf: &mut <MySql as HasArguments<'q>>::ArgumentBuffer) -> IsNull {
        let decoded = serde_json::to_value(self).unwrap();
        let decoded = decoded.as_str().unwrap();
        decoded.encode_by_ref(buf)
    }
}
