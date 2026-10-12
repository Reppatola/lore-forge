use serde::{Deserialize, Serialize};
use crate::id::EntityId;
use crate::source::Source;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RelationKind {
    // === Семья ===
    Father,       // отец
    Mother,       // мать
    Son,          // сын
    Daughter,     // дочь
    Brother,      // брат
    Sister,       // сестра
    Spouse,       // супруг(а)
    Child,        // ребёнок
    Parent,       // родитель
    Grandparent,  // дед/бабушка
    Grandchild,   // внук
    Relative,     // родственник (общее)
    Family,       // семья (без уточнения)

    // === Межличностные ===
    Friend,       // друг
    Enemy,        // враг
    Ally,         // союзник
    Love,         // любовь
    Rival,        // соперник
    Mentor,       // наставник
    Student,      // ученик

    // === Работа / служба ===
    Employer,     // работодатель
    Employee,     // работник
    Servant,      // слуга
    Master,       // хозяин
    Colleague,    // коллега

    // === Членство ===
    Member,       // состоит в
    Leader,       // возглавляет
    Founder,      // основатель
    Follower,     // последователь

    // === Персонаж ↔ локация ===
    LivesIn,      // живёт в
    BornIn,       // рождён в
    Visits,       // посещает
    Owns,         // владеет

    // === Группа ↔ группа ===
    AlliedWith,   // в союзе с
    AtWarWith,    // воюет с
    Controls,     // контролирует
    VassalOf,     // вассал
    Overlord,     // сюзерен

    // === Квесты ===
    GivesQuest,   // выдаёт квест
    TakesQuest,   // берёт квест
    QuestIn,      // квест происходит в

    // === Прочее ===
    LocatedIn,    // находится в
    Contains,     // содержит
    Knows,        // знает
    Uses,         // использует
    Created,      // создал
    Participates, // участвует в

    // === Свободное ===
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relation {
    pub target: EntityId,
    pub kind: RelationKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub since: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub until: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sources: Vec<Source>,
}