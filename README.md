# lore-forge 🦀

**Компилятор игрового мира из текста в структурированную JSON-базу.**

Инструмент превращает сырой текст истории (главы романа, лор, нарратив)
в структурированную базу знаний для RPG: персонажей, локации, города,
фракции, события, предметы — каждый объект в отдельном JSON-файле
со всеми источниками и связями.

Работает **локально** с моделями через [Ollama](https://ollama.com).
Никаких облаков, никакого API, никаких утечек данных.

---

## Зачем это

Пишешь историю мира постапокалиптической RPG глава за главой.

- Глава 1: знакомимся с персонажами и локациями
- Глава 2: добавляем фракции, события, артефакты
- Глава 3: раскрываем предыстории...

**Проблема:** ручная работа — таблицы, wiki, потерянные заметки.

**Решение:** `lore-forge` — автоматически извлекает всё из текста
и собирает единую базу. Добавил главу — база обогатилась.

Каждая крупица информации (цитата, связь, описание) помнит
**откуда она взята** и **когда обновлялась**.

---

## Что умеет

- 📖 **Читает** главы `.md` / `.txt` с порядковыми номерами (`001_...`, `002_...`)
- ✂️ **Режет** на сцены (умно: по абзацам или по предложениям)
- 🧠 **Извлекает** через LLM до 15 типов сущностей:
  персонажи, локации, города, деревни, базы, водоёмы, здания,
  квесты, фракции, кланы, альянсы, события, предметы, существа, ресурсы
- 🔗 **Строит** связи: родство, дружба, вражда, владение, участие
- 💾 **Кеширует** ответы LLM (по blake3-хешу) — повторные прогоны мгновенны
- 🔀 **Сливает** дубли — один объект из N упоминаний
- 📊 **Индексирует** — `_meta/index.json` с оглавлением всей базы
- 📦 **Портативна** — один `.exe`, работает из любой папки
- 🛡️ **Не теряет данные** — сохраняет все источники, даже нерелевантные (`unresolved`)

---

## Требования

- **Windows / Linux / macOS**
- **Rust 1.80+** (только для сборки из исходников, не для использования)
- **Ollama** — установлена и запущена ([ollama.com](https://ollama.com))
- Одна из моделей:
  - `qwen2.5-coder:14b` (рекомендуется)
  - `qwen2.5-coder:7b` (быстрее)
  - `qwen3:8b-64k` (для длинных текстов)

**Проверка Ollama:**

```bash
ollama list
curl http://localhost:11434/api/tags

Установка
Вариант 1 — portable .exe (проще)
Скачай loreforge.exe со страницы Releases

Создай папку под проект, например C:\my-world\

Положи loreforge.exe в эту папку

Запусти первичную настройку:

bash
cd C:\my-world
loreforge.exe init
Создастся структура:

text
my-world\
├── loreforge.exe
├── loreforge.toml          ← настройки LLM
├── data\
│   ├── input\              ← сюда главы
│   ├── output\             ← база генерируется
│   └── cache\              ← кеш LLM
└── prompts\
    └── extract_entities.md ← промпт (можно править)
Вариант 2 — из исходников
bash
git clone https://github.com/yourname/lore-forge
cd lore-forge
cargo build --release
# бинарник в target/release/loreforge.exe
Использование
Быстрый старт
Положи главу в data/input/, например 001_начало.md

Запусти:

bash
loreforge.exe run
Всё. Через 5–15 минут база готова: data/output/entities/.

Команды
bash
loreforge.exe init               # создать структуру (один раз)
loreforge.exe ingest             # только порезать на чанки
loreforge.exe extract            # только LLM-извлечение
loreforge.exe merge              # только сбор базы
loreforge.exe run                # полный цикл (рекомендуется)
loreforge.exe stats              # статистика
loreforge.exe ping               # проверить Ollama
Флаги:

bash
--force                # игнорировать кеш, пересчитать всё
--chapter 1            # только глава 1
--task extract_heavy   # выбор модели из loreforge.toml
--root C:\path         # явный корень (по умолчанию — папка рядом с .exe)
Примеры
bash
# Обработать только главу 5
loreforge.exe extract --chapter 5

# Пересобрать без LLM (мгновенно)
loreforge.exe merge --force

# Полный цикл с игнорированием кеша
loreforge.exe run --force
Формат главы
Файлы в data/input/:

text
001_пробуждение.md
002_встреча.md
003_дорога_на_север.md
Правила:

Начинается с порядкового номера (001, 002, ...)

Дальше подчёркивание и название

Расширение .md или .txt

Кодировка UTF-8 (VSCode сохраняет по умолчанию)

Внутри — обычный текст. Пустые строки между абзацами помогают
чанкеру, но не обязательны — есть fallback на разбивку по предложениям.

Что на выходе
text
data/output/
├── entities/                ← финальная база
│   ├── characters/
│   │   ├── char_aragorn.json
│   │   └── char_elena.json
│   ├── cities/
│   │   └── city_novgorod.json
│   ├── factions/
│   │   └── fac_dunedain.json
│   └── ...
├── entities.jsonl           ← все объекты по строке на объект (для отладки)
├── extracted/               ← сырые ответы LLM
│   └── 001.jsonl
└── _meta/
    ├── index.json           ← оглавление базы
    ├── unresolved.jsonl     ← неклассифицированное
    └── raw/                 ← чистые ответы LLM (для отладки)
Пример объекта
json
{
  "id": "char_knyaz_andrey",
  "type": "character",
  "subtype": "npc",
  "names": ["Князь Андрей", "Андрей", "князь Андрей"],
  "description": "Молодой князь, уходит на войну, женат на Лизе",
  "tags": ["военный", "сюжетный"],
  "attributes": {
    "gender": "male",
    "occupation": "офицер"
  },
  "relations": [
    {"target": "char_liza", "kind": "spouse"},
    {"target": "char_marya", "kind": "sister"}
  ],
  "sources": [
    {"chapter": 1, "scene": 0, "quote": "Ты постарел, Тихон"},
    {"chapter": 2, "scene": 1, "quote": "Даже завтра, — отвечал брат"}
  ],
  "meta": {
    "revision": 3,
    "confidence": 0.9,
    "extractor": "qwen2.5-coder:14b"
  }
}
Ключевые поля:

sources — все цитаты из всех глав, где упомянут персонаж

meta.revision — сколько раз объект обогащался

relations — связи с другими объектами базы

attributes — тип-специфичные поля (для персонажа — gender/occupation, для города — population)

Архитектура
text
crates/
├── core/       — типы: Entity, EntityId, Relation, Source
├── ingest/     — чтение глав, чанкинг, manifest
├── llm/        — HTTP-клиент Ollama, роутер моделей, кеш
├── extract/    — промпт, парсер JSON, валидатор
├── merge/      — слияние дублей, генерация ID, запись файлов
├── storage/    — JSONL, пути проекта
└── cli/        — бинарник loreforge
Пайплайн:

text
TXT главы
   ↓ [ingest] чтение → чанки
   ↓ [extract] каждый чанк → LLM → JSON сущностей
   ↓ [merge] все JSON → единые объекты → файлы
   ↓
JSON-база
Настройка
loreforge.toml
toml
[llm]
endpoint = "http://localhost:11434"
default_model = "qwen2.5-coder:14b"
timeout_secs = 600
retries = 3

[llm.tasks]
extract_heavy = "qwen2.5-coder:14b"          # точность (по умолчанию)
extract_light = "qwen2.5-coder:7b"           # скорость
extract_long  = "qwen3:8b-64k"               # длинные чанки
classify      = "qwen2.5-coder:7b"           # классификация
code          = "maternion/strand-rust-coder:14b-q4_K_M"  # кодогенерация
prompts/extract_entities.md
Промпт для LLM. Можно править без пересборки — loreforge.exe подхватит
изменения при следующем запуске. Если файла нет — используется встроенный.

Дорожная карта
✅ v0.1 — ingest + extract + merge, portable exe

🔜 v0.2 — alias-resolution (объединение «Андрей» и «Князь Андрей» в один объект)

🔜 v0.3 — export в форматы игровых движков (Godot, Unity, Bevy)

🔜 v0.4 — граф связей + визуализация

🔜 v0.5 — timeline событий с хронологией

🔜 v1.0 — интеграция с онлайн-сервером игры

Философия
Локально, не в облаке. Всё через Ollama, никаких OpenAI.

Не терять данные. Все цитаты и упоминания сохраняются.

Инкрементально. Добавил главу — база обогатилась. Никаких полных пересборок.

Портативно. Один .exe, одна папка, всё автономно.

Прозрачно. Все промежуточные файлы доступны для отладки.

Расширяемо. Промпт, конфиг, схема — всё редактируется.

Лицензия
MIT — используй как хочешь.

Вклад
Issues и PR приветствуются. Для крупных изменений — сначала открой Issue.

Благодарности
Ollama — за локальный LLM-инференс

Qwen — за модели

Сообществу Rust — за прекрасные крейты

text

**Сохрани** `Ctrl+S`.

---

## Шаг 3 — LICENSE (MIT)

**Файл `C:\projects\lore-forge\LICENSE`:**

```powershell
code C:\projects\lore-forge\LICENSE
text
MIT License

Copyright (c) 2026 No4ka

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.